use constants::RINGS;
use ring_kind::RingKind;

use crate::init::GameState;
use crate::objects::{Object, ObjectId};
use crate::prelude::item_usage::{ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::r#use::relight;
use crate::random::{coin_toss, get_rand};
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};

pub(crate) mod constants;
pub(crate) mod ring_kind;
pub(crate) mod ring_gem;


pub(crate) fn ask_ring_hand(game: &mut GameState) -> Option<PlayerHand> {
	match ask_left_or_right(game) {
		'l' => Some(PlayerHand::Left),
		'r' => Some(PlayerHand::Right),
		_ => None,
	}
}

fn ask_left_or_right(game: &mut GameState) -> char {
	diary::show_prompt("left or right hand?", &mut game.diary);
	let mut ch;
	loop {
		ch = rgetchar();
		let good_ch = ch == CANCEL_CHAR || ch == 'l' || ch == 'r' || ch == '\n' || ch == '\r';
		if good_ch {
			break;
		}
	}
	ch
}

pub fn un_put_hand(hand: PlayerHand, game: &mut GameState) -> Option<ObjectId> {
	let un_put_id = game.player.un_put_ring(hand);
	ring_stats(true, game);
	un_put_id
}

pub fn gr_ring(ring: &mut Object, assign_wk: bool) {
	ring.what_is = Ring;
	if assign_wk {
		ring.which_kind = get_rand(0, (RINGS - 1) as u16);
	}
	ring.class = 0;
	match RingKind::from_index(ring.which_kind as usize) {
		RingKind::RTeleport => {
			ring.is_cursed = 1;
		}
		RingKind::AddStrength | RingKind::Dexterity => {
			loop {
				ring.class = get_rand(0, 4) - 2;
				if ring.class != 0 {
					break;
				}
			}
			ring.is_cursed = if ring.class < 0 { 1 } else { 0 };
		}
		RingKind::Adornment => {
			ring.is_cursed = if coin_toss() { 1 } else { 0 };
		}
		_ => ()
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PlayerHand { Left, Right }

impl PlayerHand {
	pub fn use_flag(&self) -> u16 {
		match self {
			PlayerHand::Left => ON_LEFT_HAND,
			PlayerHand::Right => ON_RIGHT_HAND,
		}
	}
	pub fn invert(&self) -> Self {
		match self {
			PlayerHand::Left => PlayerHand::Right,
			PlayerHand::Right => PlayerHand::Left
		}
	}
	pub const ALL_HANDS: [PlayerHand; 2] = [PlayerHand::Left, PlayerHand::Right];
}

pub(crate) mod effects;


pub fn ring_stats(print: bool, game: &mut GameState) {
	game.player.ring_effects.clear_stealthy();
	game.player.ring_effects.clear_calorie_burn();
	game.player.ring_effects.set_teleport(false);
	game.player.ring_effects.set_sustain_strength(false);
	game.player.ring_effects.clear_add_strength();
	game.player.ring_effects.clear_regeneration();
	game.player.ring_effects.clear_dexterity();
	game.player.ring_effects.set_see_invisible(false);
	game.player.ring_effects.set_maintain_armor(false);
	game.player.ring_effects.clear_auto_search();

	for ring_hand in PlayerHand::ALL_HANDS {
		match game.player.ring_id(ring_hand) {
			None => {
				continue;
			}
			Some(ring_id) => {
				game.player.ring_effects.incr_calorie_burn();
				match game.player.expect_object(ring_id).ring_kind().expect("ring kind") {
					RingKind::Stealth => {
						game.player.ring_effects.incr_stealthy();
					}
					RingKind::RTeleport => {
						game.player.ring_effects.set_teleport(true);
					}
					RingKind::Regeneration => {
						game.player.ring_effects.incr_regeneration();
					}
					RingKind::SlowDigest => {
						game.player.ring_effects.slow_calorie_burn();
					}
					RingKind::AddStrength => {
						let player_class = game.player.expect_object(ring_id).class;
						game.player.ring_effects.increase_add_strength(player_class);
					}
					RingKind::SustainStrength => {
						game.player.ring_effects.set_sustain_strength(true);
					}
					RingKind::Dexterity => {
						let player_class = game.player.expect_object(ring_id).class;
						game.player.ring_effects.increase_dexterity(player_class);
					}
					RingKind::Adornment => {
						// Do nothing
					}
					RingKind::RSeeInvisible => {
						game.player.ring_effects.set_see_invisible(true);
					}
					RingKind::MaintainArmor => {
						game.player.ring_effects.set_maintain_armor(true);
					}
					RingKind::Searching => {
						game.player.ring_effects.increase_auto_search(2);
					}
				}
			}
		}
	}
	if print {
		game.stats_changed = true;
		relight(game);
	}
}