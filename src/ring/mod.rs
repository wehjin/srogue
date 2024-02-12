#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use constants::{ADD_STRENGTH, ADORNMENT, DEXTERITY, R_TELEPORT, RINGS};
use crate::player::Player;
use crate::player::rings::HandUsage;
use crate::prelude::*;
use crate::prelude::item_usage::{ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::prelude::object_what::PackFilter::Rings;
use ring_kind::RingKind;
use crate::prelude::stat_const::STAT_STRENGTH;
use crate::ring::effects::*;

pub(crate) mod constants;
pub(crate) mod ring_kind;

pub static mut r_rings: isize = 0;

pub unsafe fn put_on_ring(player: &mut Player, level: &mut Level) {
	if player.hand_usage() == HandUsage::Both {
		message("wearing two rings already", 0);
		return;
	}
	let ch = pack_letter("put on what?", Rings, player);
	if ch == CANCEL {
		return;
	}
	match player.object_id_with_letter(ch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			if player.object_what(obj_id) != Ring {
				message("that's not a ring", 0);
				return;
			}
			let ring_id = obj_id;
			if player.check_object(ring_id, obj::is_on_either_hand) {
				message("that ring is already being worn", 0);
				return;
			}
			let hand = match player.hand_usage() {
				HandUsage::None => match ask_ring_hand() {
					None => {
						check_message();
						return;
					}
					Some(ring_hand) => ring_hand,
				},
				HandUsage::Left => PlayerHand::Right,
				HandUsage::Right => PlayerHand::Left,
				HandUsage::Both => unreachable!("both hands checked at top of put_on_ring")
			};
			if !player.hand_is_free(hand) {
				check_message();
				message("there's already a ring on that hand", 0);
				return;
			}
			player.put_ring(ring_id, hand);
			ring_stats(true, player, level);
			check_message();
			{
				let ring = player.object(ring_id).expect("ring in pack");
				let msg = get_obj_desc(ring);
				message(&msg, 0);
			}
			reg_move(player, level);
		}
	}
}

unsafe fn ask_ring_hand() -> Option<PlayerHand> {
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

pub unsafe fn remove_ring(player: &mut Player, level: &mut Level) {
	let hand = match player.hand_usage() {
		HandUsage::None => {
			inv_rings(player);
			return;
		}
		HandUsage::Left => PlayerHand::Left,
		HandUsage::Right => PlayerHand::Right,
		HandUsage::Both => {
			let asked = ask_ring_hand();
			check_message();
			match asked {
				None => { return; }
				Some(hand) => hand
			}
		}
	};
	if player.ring_id(hand).is_none() {
		message("there's no ring on that hand", 0);
		return;
	}
	if player.check_ring(hand, obj::is_cursed) {
		message(CURSE_MESSAGE, 0);
		return;
	}
	let removed_id = un_put_hand(hand, player, level).expect("some removed_id");
	{
		let removed_obj = player.object(removed_id).expect("some removed_obj");
		let msg = format!("removed {}", get_obj_desc(removed_obj));
		message(&msg, 0);
	}
	reg_move(player, level);
}

pub unsafe fn un_put_hand(hand: PlayerHand, player: &mut Player, level: &mut Level) -> Option<ObjectId> {
	let un_put_id = player.un_put_ring(hand);
	ring_stats(true, player, level);
	un_put_id
}

pub fn gr_ring(ring: &mut object, assign_wk: bool) {
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
	if r_rings == 0 {
		message("not wearing any rings", 0);
	} else {
		for ring_hand in PlayerHand::ALL_HANDS {
			if let Some(ring) = player.ring(ring_hand) {
				let msg = get_obj_desc(ring);
				message(&msg, 0);
			}
		}
	}
	if wizard {
		message(
			&format!("ste {}, r_r {}, e_r {}, r_t {}, s_s {}, a_s {}, reg {}, r_e {}, s_i {}, m_a {}, aus {}",
			         player.ring_effects.stealthy(), r_rings,
			         player.ring_effects.calorie_burn(), player.ring_effects.has_teleport(),
			         player.ring_effects.has_sustain_strength(), player.ring_effects.add_strength(),
			         player.ring_effects.regeneration(), player.ring_effects.dexterity(),
			         player.ring_effects.has_see_invisible(), player.ring_effects.has_maintain_armor(),
			         auto_search),
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


pub unsafe fn ring_stats(print: bool, player: &mut Player, level: &mut Level) {
	player.ring_effects.clear_stealthy();
	r_rings = 0;
	player.ring_effects.clear_calorie_burn();
	player.ring_effects.set_teleport(false);
	player.ring_effects.set_sustain_strength(false);
	player.ring_effects.clear_add_strength();
	player.ring_effects.clear_regeneration();
	player.ring_effects.clear_dexterity();
	player.ring_effects.set_see_invisible(false);
	player.ring_effects.set_maintain_armor(false);
	auto_search = 0;

	for ring_hand in PlayerHand::ALL_HANDS {
		match player.ring_id(ring_hand) {
			None => {
				continue;
			}
			Some(ring_id) => {
				r_rings += 1;
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
						auto_search += 2;
					}
				}
			}
		}
	}
	if print {
		print_stats(STAT_STRENGTH, player);
		relight(player, level);
	}
}