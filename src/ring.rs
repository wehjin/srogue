#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use crate::prelude::*;
use crate::prelude::item_usage::{ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::prelude::object_what::PackFilter::Rings;
use crate::prelude::ring_kind::{RingKind, RINGS};
use crate::prelude::stat_const::STAT_STRENGTH;


pub static left_or_right: &'static str = "left or right hand?";
pub static no_ring: &'static str = "there's no ring on that hand";
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

pub unsafe fn put_on_ring(depth: &RogueDepth, level: &Level) {
	if r_rings == 2 {
		message("wearing two rings already", 0);
		return;
	}
	let ch = pack_letter("put on what?", Rings);
	if ch == CANCEL {
		return;
	}
	let ring = get_letter_object(ch);
	if ring.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*ring).what_is != Ring {
		message("that's not a ring", 0);
		return;
	}
	if (*ring).in_use_flags & (ON_LEFT_HAND | ON_RIGHT_HAND) != 0 {
		message("that ring is already being worn", 0);
		return;
	}
	let mut ch = char::default();
	if r_rings == 1 {
		ch = if !rogue.left_ring.is_null() { 'r' } else { 'l' };
	} else {
		ch = ask_left_or_right();
	}
	if ch != 'l' && ch != 'r' {
		check_message();
		return;
	}
	if ch == 'l' && !rogue.left_ring.is_null() || ch == 'r' && !rogue.right_ring.is_null()
	{
		check_message();
		message("there's already a ring on that hand", 0);
		return;
	}
	if ch == 'l' {
		do_put_on(&mut *ring, true);
	} else {
		do_put_on(&mut *ring, false);
	}
	ring_stats(true, depth.cur, level);
	check_message();
	message(&get_desc(&*ring), 0);
	reg_move(depth, level);
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

pub unsafe fn do_put_on(ring: &mut obj, on_left: bool) {
	if on_left {
		ring.in_use_flags |= ON_LEFT_HAND;
		rogue.left_ring = ring;
	} else {
		ring.in_use_flags |= ON_RIGHT_HAND;
		rogue.right_ring = ring;
	}
}

pub unsafe fn remove_ring(depth: &RogueDepth, level: &Level) {
	let mut left = false;
	let mut right = false;
	if r_rings == 0 {
		inv_rings();
	} else if !rogue.left_ring.is_null() && rogue.right_ring.is_null() {
		left = true;
	} else if rogue.left_ring.is_null() && !rogue.right_ring.is_null() {
		right = true;
	} else {
		let ch = ask_left_or_right();
		left = ch == 'l';
		right = ch == 'r';
		check_message();
	}
	if left || right {
		let mut ring: *mut obj = 0 as *mut obj;
		if left {
			if !rogue.left_ring.is_null() {
				ring = rogue.left_ring;
			} else {
				message(no_ring, 0);
			}
		} else if !rogue.right_ring.is_null() {
			ring = rogue.right_ring;
		} else {
			message(no_ring, 0);
		}
		if (*ring).is_cursed != 0 {
			message(CURSE_MESSAGE, 0);
		} else {
			un_put_on(ring, depth.cur, level);
			message(&format!("removed {}", get_desc(&*ring)), 0);
			reg_move(depth, level);
		}
	}
}

pub unsafe fn un_put_on(ring: *mut obj, level_depth: usize, level: &Level) {
	if !ring.is_null() && ((*ring).in_use_flags & ON_LEFT_HAND != 0) {
		(*ring).in_use_flags &= !ON_LEFT_HAND;
		rogue.left_ring = 0 as *mut object;
	} else if !ring.is_null() && ((*ring).in_use_flags & ON_RIGHT_HAND != 0) {
		(*ring).in_use_flags &= !ON_RIGHT_HAND;
		rogue.right_ring = 0 as *mut object;
	}
	ring_stats(true, level_depth, level);
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

pub unsafe fn inv_rings() {
	if r_rings == 0 {
		message("not wearing any rings", 0);
	} else {
		if !rogue.left_ring.is_null() {
			message(&get_desc(&*rogue.left_ring), 0);
		}
		if !rogue.right_ring.is_null() {
			message(&get_desc(&*rogue.right_ring), 0);
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


enum RingHand {
	Left,
	Right,
}

impl RingHand {
	pub const ALL: &'static [RingHand; 2] = &[RingHand::Left, RingHand::Right];
}

pub unsafe fn ring_stats(print: bool, level_depth: usize, level: &Level) {
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

	for ring_hand in RingHand::ALL {
		let ring = match ring_hand {
			RingHand::Left => rogue.left_ring,
			RingHand::Right => rogue.right_ring,
		};
		if ring.is_null() {
			continue;
		}
		let ring = &*ring;
		r_rings += 1;
		e_rings += 1;
		match RingKind::from_index((*ring).which_kind as usize) {
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
	if print {
		print_stats(STAT_STRENGTH, level_depth);
		relight(level);
	}
}