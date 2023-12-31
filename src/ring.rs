#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	fn reg_move() -> libc::c_char;
	fn get_letter_object() -> *mut object;
}

use crate::prelude::*;
use crate::prelude::object_what::PackFilter::Rings;
use crate::prelude::ring_kind::RingKind;
use crate::prelude::stat_const::STAT_STRENGTH;


#[no_mangle]
pub static left_or_right: &'static str = "left or right hand?";
#[no_mangle]
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
#[no_mangle]
pub static mut sustain_strength: libc::c_char = 0;
#[no_mangle]
pub static mut maintain_armor: libc::c_char = 0;

#[no_mangle]
pub unsafe extern "C" fn put_on_ring() -> i64 {
	let mut ch: libc::c_short = 0;
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut ring: *mut object = 0 as *mut object;
	if r_rings as i64 == 2 as i64 {
		message("wearing two rings already", 0);
		return;
	}
	ch = pack_letter("put on what?", Rings) as libc::c_short;
	if ch as i64 == '\u{1b}' as i32 {
		return;
	}
	ring = get_letter_object(ch as i64);
	if ring.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*ring).what_is as i64
		& 0o200 as i64 as libc::c_ushort as i64 == 0
	{
		message(
			b"that's not a ring\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*ring).in_use_flags as i64
		& (0o4 as i64 as libc::c_ushort as i64
		| 0o10 as i64 as libc::c_ushort as i64) != 0
	{
		message(
			b"that ring is already being worn\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if r_rings as i64 == 1 {
		ch = (if !(rogue.left_ring).is_null() { 'r' as i32 } else { 'l' as i32 })
			as libc::c_short;
	} else {
		message(left_or_right, 0 as i64);
		loop {
			ch = rgetchar() as libc::c_short;
			if !(ch as i64 != '\u{1b}' as i32 && ch as i64 != 'l' as i32
				&& ch as i64 != 'r' as i32 && ch as i64 != '\n' as i32
				&& ch as i64 != '\r' as i32)
			{
				break;
			}
		}
	}
	if ch as i64 != 'l' as i32 && ch as i64 != 'r' as i32 {
		check_message();
		return;
	}
	if ch as i64 == 'l' as i32 && !(rogue.left_ring).is_null()
		|| ch as i64 == 'r' as i32 && !(rogue.right_ring).is_null()
	{
		check_message();
		message(
			b"there's already a ring on that hand\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if ch as i64 == 'l' as i32 {
		do_put_on(ring, 1);
	} else {
		do_put_on(ring, 0 as i64);
	}
	ring_stats(1);
	check_message();
	get_desc(ring, desc.as_mut_ptr());
	message(desc.as_mut_ptr(), 0 as i64);
	reg_move();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn remove_ring() -> i64 {
	let mut left: libc::c_char = 0 as i64 as libc::c_char;
	let mut right: libc::c_char = 0 as i64 as libc::c_char;
	let mut ch: libc::c_short = 0;
	let mut ring: *mut object = 0 as *mut object;
	if r_rings as i64 == 0 as i64 {
		inv_rings();
	} else if !(rogue.left_ring).is_null() && (rogue.right_ring).is_null() {
		left = 1 as libc::c_char;
	} else if (rogue.left_ring).is_null() && !(rogue.right_ring).is_null() {
		right = 1 as libc::c_char;
	} else {
		message(left_or_right, 0 as i64);
		loop {
			ch = rgetchar() as libc::c_short;
			if !(ch as i64 != '\u{1b}' as i32 && ch as i64 != 'l' as i32
				&& ch as i64 != 'r' as i32 && ch as i64 != '\n' as i32
				&& ch as i64 != '\r' as i32)
			{
				break;
			}
		}
		left = (ch as i64 == 'l' as i32) as i64 as libc::c_char;
		right = (ch as i64 == 'r' as i32) as i64 as libc::c_char;
		check_message();
	}
	if left as i64 != 0 || right as i64 != 0 {
		if left != 0 {
			if !(rogue.left_ring).is_null() {
				ring = rogue.left_ring;
			} else {
				message(no_ring, 0 as i64);
			}
		} else if !(rogue.right_ring).is_null() {
			ring = rogue.right_ring;
		} else {
			message(no_ring, 0 as i64);
		}
		if (*ring).is_cursed != 0 {
			message(curse_message, 0 as i64);
		} else {
			un_put_on(ring);
			message(&format!("removed {}", get_desc(ring)), 0 as i64);
			reg_move();
		}
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn gr_ring(
	mut ring: *mut object,
	mut assign_wk: libc::c_char,
) -> i64 {
	(*ring).what_is = 0o200 as i64 as libc::c_ushort;
	if assign_wk != 0 {
		(*ring)
			.which_kind = get_rand(
			0 as i64,
			11 - 1,
		) as libc::c_ushort;
	}
	(*ring).class = 0;
	match (*ring).which_kind as i64 {
		1 => {
			(*ring).is_cursed = 1 as libc::c_short;
		}
		4 | 6 => {
			loop {
				(*ring)
					.class = (get_rand(0 as i64, 4 as i64)
					- 2 as i64) as libc::c_short;
				if !((*ring).class as i64 == 0 as i64) {
					break;
				}
			}
			(*ring)
				.is_cursed = (((*ring).class as i64) < 0 as i64)
				as i64 as libc::c_short;
		}
		7 => {
			(*ring).is_cursed = coin_toss() as libc::c_short;
		}
		_ => {}
	}
	panic!("Reached end of non-void function without returning");
}

enum RingHand {
	Left,
	Right,
}

impl RingHand {
	pub const ALL: &'static [RingHand; 2] = &[RingHand::Left, RingHand::Right];
}

pub unsafe fn ring_stats(print: bool) {
	r_rings = 0;
	e_rings = 0;
	r_teleport = 0;
	sustain_strength = 0;
	add_strength = 0;
	regeneration = 0;
	ring_exp = 0;
	r_see_invisible = 0;
	maintain_armor = 0;
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
		match RingKind::from_code((*ring).which_kind) {
			RingKind::Stealth => { stealthy += 1; }
			RingKind::RTeleport => { r_teleport = 1; }
			RingKind::Regeneration => { regeneration += 1; }
			RingKind::SlowDigest => { e_rings -= 2; }
			RingKind::AddStrength => { add_strength += ring.class; }
			RingKind::SustainStrength => { sustain_strength = 1; }
			RingKind::Dexterity => { ring_exp += ring.class; }
			RingKind::Adornment => {}
			RingKind::RSeeInvisible => { r_see_invisible = 1; }
			RingKind::MaintainArmor => { maintain_armor = 1; }
			RingKind::Searching => { auto_search += 2; }
		}
	}
	if print {
		print_stats(STAT_STRENGTH);
		relight();
	}
}