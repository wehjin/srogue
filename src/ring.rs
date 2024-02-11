#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments)]

use crate::player::{Player};
use crate::player::rings::HandUsage;
use crate::prelude::*;
use crate::prelude::item_usage::{ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::prelude::object_what::PackFilter::Rings;
use crate::prelude::ring_kind::{RingKind, RINGS};
use crate::prelude::stat_const::STAT_STRENGTH;


pub static left_or_right: &'static str = "left or right hand?";
#[no_mangle]
pub static mut stealthy: libc::c_short = 0;
pub static mut r_rings: isize = 0;
pub static mut add_strength: isize = 0;
#[no_mangle]
pub static mut e_rings: libc::c_short = 0;
pub static mut regeneration: isize = 0;
pub static mut ring_exp: isize = 0;
#[no_mangle]
pub static mut auto_search: libc::c_short = 0;
pub static mut r_teleport: bool = false;
pub static mut r_see_invisible: bool = false;
pub static mut sustain_strength: bool = false;
pub static mut maintain_armor: bool = false;

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
	let mut ch = char::default();
	message(left_or_right, 0);
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
		ring_kind::R_TELEPORT => {
			ring.is_cursed = 1;
		}
		ring_kind::ADD_STRENGTH | ring_kind::DEXTERITY => {
			loop {
				ring.class = get_rand(0, 4) - 2;
				if ring.class != 0 {
					break;
				}
			}
			ring.is_cursed = if ring.class < 0 { 1 } else { 0 };
		}
		ring_kind::ADORNMENT => {
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
			         stealthy, r_rings, e_rings, r_teleport, sustain_strength,
			         add_strength, regeneration, ring_exp, r_see_invisible,
			         maintain_armor, auto_search),
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

pub unsafe fn ring_stats(print: bool, player: &mut Player, level: &mut Level) {
	r_rings = 0;
	e_rings = 0;
	r_teleport = false;
	sustain_strength = false;
	add_strength = 0;
	regeneration = 0;
	ring_exp = 0;
	r_see_invisible = false;
	maintain_armor = false;
	auto_search = 0;

	for ring_hand in PlayerHand::ALL_HANDS {
		match player.ring(ring_hand) {
			None => {
				continue;
			}
			Some(ring) => {
				r_rings += 1;
				e_rings += 1;
				match RingKind::from_index(ring.which_kind as usize) {
					RingKind::Stealth => { stealthy += 1; }
					RingKind::RTeleport => { r_teleport = true; }
					RingKind::Regeneration => { regeneration += 1; }
					RingKind::SlowDigest => { e_rings -= 2; }
					RingKind::AddStrength => { add_strength += ring.class; }
					RingKind::SustainStrength => { sustain_strength = true; }
					RingKind::Dexterity => { ring_exp += ring.class; }
					RingKind::Adornment => {}
					RingKind::RSeeInvisible => { r_see_invisible = true; }
					RingKind::MaintainArmor => { maintain_armor = true; }
					RingKind::Searching => { auto_search += 2; }
				}
			}
		}
	}
	if print {
		print_stats(STAT_STRENGTH, player);
		relight(player, level);
	}
}