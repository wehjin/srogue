#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use constants::{ADD_STRENGTH, ADORNMENT, DEXTERITY, R_TELEPORT, RINGS};
use ring_kind::RingKind;
use crate::level::Level;
use crate::message::{CANCEL, message, print_stats, rgetchar};
use crate::monster::MonsterMash;
use crate::objects::{Object, ObjectId};
use crate::player::Player;
use crate::player::rings::HandUsage;
use crate::prelude::item_usage::{ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::prelude::stat_const::STAT_STRENGTH;
use crate::r#use::relight;
use crate::random::{coin_toss, get_rand};
use crate::zap::wizard;

pub(crate) mod constants;
pub(crate) mod ring_kind;
pub(crate) mod ring_gem;


pub(crate) unsafe fn ask_ring_hand() -> Option<PlayerHand> {
	match ask_left_or_right() {
		'l' => Some(PlayerHand::Left),
		'r' => Some(PlayerHand::Right),
		_ => None,
	}
}

unsafe fn ask_left_or_right() -> char {
	message("left or right hand?", 0);
	let mut ch;
	loop {
		ch = rgetchar();
		let good_ch = ch == CANCEL || ch == 'l' || ch == 'r' || ch == '\n' || ch == '\r';
		if good_ch {
			break;
		}
	}
	ch
}

pub unsafe fn un_put_hand(hand: PlayerHand, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> Option<ObjectId> {
	let un_put_id = player.un_put_ring(hand);
	ring_stats(true, mash, player, level);
	un_put_id
}

pub fn gr_ring(ring: &mut Object, assign_wk: bool) {
	ring.what_is = Ring;
	if assign_wk {
		ring.which_kind = get_rand(0, (RINGS - 1) as u16);
	}
	ring.class = 0;
	match ring.which_kind {
		R_TELEPORT => {
			ring.is_cursed = 1;
		}
		ADD_STRENGTH | DEXTERITY => {
			loop {
				ring.class = get_rand(0, 4) - 2;
				if ring.class != 0 {
					break;
				}
			}
			ring.is_cursed = if ring.class < 0 { 1 } else { 0 };
		}
		ADORNMENT => {
			ring.is_cursed = if coin_toss() { 1 } else { 0 };
		}
		_ => {}
	}
}

pub unsafe fn inv_rings(player: &Player) {
	let hand_usage = player.hand_usage();
	if hand_usage == HandUsage::None {
		message("not wearing any rings", 0);
	} else {
		for ring_hand in PlayerHand::ALL_HANDS {
			if let Some(ring_id) = player.ring_id(ring_hand) {
				let msg = player.get_obj_desc(ring_id);
				message(&msg, 0);
			}
		}
	}
	if wizard {
		message(
			&format!("ste {}, r_r {}, e_r {}, r_t {}, s_s {}, a_s {}, reg {}, r_e {}, s_i {}, m_a {}, aus {}",
			         player.ring_effects.stealthy(), hand_usage.count_hands(),
			         player.ring_effects.calorie_burn(), player.ring_effects.has_teleport(),
			         player.ring_effects.has_sustain_strength(), player.ring_effects.add_strength(),
			         player.ring_effects.regeneration(), player.ring_effects.dexterity(),
			         player.ring_effects.has_see_invisible(), player.ring_effects.has_maintain_armor(),
			         player.ring_effects.auto_search()),
			0,
		);
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


pub unsafe fn ring_stats(print: bool, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	player.ring_effects.clear_stealthy();
	player.ring_effects.clear_calorie_burn();
	player.ring_effects.set_teleport(false);
	player.ring_effects.set_sustain_strength(false);
	player.ring_effects.clear_add_strength();
	player.ring_effects.clear_regeneration();
	player.ring_effects.clear_dexterity();
	player.ring_effects.set_see_invisible(false);
	player.ring_effects.set_maintain_armor(false);
	player.ring_effects.clear_auto_search();

	for ring_hand in PlayerHand::ALL_HANDS {
		match player.ring_id(ring_hand) {
			None => {
				continue;
			}
			Some(ring_id) => {
				player.ring_effects.incr_calorie_burn();
				match player.expect_object(ring_id).ring_kind().expect("ring kind") {
					RingKind::Stealth => {
						player.ring_effects.incr_stealthy();
					}
					RingKind::RTeleport => {
						player.ring_effects.set_teleport(true);
					}
					RingKind::Regeneration => {
						player.ring_effects.incr_regeneration();
					}
					RingKind::SlowDigest => {
						player.ring_effects.slow_calorie_burn();
					}
					RingKind::AddStrength => {
						let player_class = player.expect_object(ring_id).class;
						player.ring_effects.increase_add_strength(player_class);
					}
					RingKind::SustainStrength => {
						player.ring_effects.set_sustain_strength(true);
					}
					RingKind::Dexterity => {
						let player_class = player.expect_object(ring_id).class;
						player.ring_effects.increase_dexterity(player_class);
					}
					RingKind::Adornment => {
						// Do nothing
					}
					RingKind::RSeeInvisible => {
						player.ring_effects.set_see_invisible(true);
					}
					RingKind::MaintainArmor => {
						player.ring_effects.set_maintain_armor(true);
					}
					RingKind::Searching => {
						player.ring_effects.increase_auto_search(2);
					}
				}
			}
		}
	}
	if print {
		print_stats(STAT_STRENGTH, player);
		relight(mash, player, level);
	}
}