#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_short, strcpy, strlen};
use scroll_kind::SCARE_MONSTER;
use crate::{get_input_line, message, mv_aquatars, print_stats};
use crate::objects::IdStatus::Identified;
use crate::objects::place_at;

extern "C" {
	fn reg_move() -> libc::c_char;
	fn alloc_object() -> *mut object;
}

use crate::prelude::*;
use crate::prelude::IdStatus::Unidentified;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::{PackFilter};
use crate::prelude::object_what::ObjectWhat::{Armor, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Amulets, AnyFrom, Armors, Foods, Potions, Rings, Scrolls, Wands, Weapons};
use crate::prelude::stat_const::STAT_GOLD;


pub static mut curse_message: &'static str = "you can't, it appears to be cursed";
pub const MAX_PACK_COUNT: usize = 24;

#[no_mangle]
pub unsafe extern "C" fn add_to_pack(
	mut obj: *mut object,
	mut pack: *mut object,
	mut condense: i64,
) -> *mut object {
	let mut op: *mut object = 0 as *mut object;
	if condense != 0 {
		op = check_duplicate(obj, pack);
		if !op.is_null() {
			free_object(obj);
			return op;
		} else {
			(*obj).ichar = next_avail_ichar();
		}
	}
	if ((*pack).next_object).is_null() {
		(*pack).next_object = obj;
	} else {
		op = (*pack).next_object;
		while !((*op).next_object).is_null() {
			op = (*op).next_object;
		}
		(*op).next_object = obj;
	}
	(*obj).next_object = 0 as *mut obj;
	return obj;
}

#[no_mangle]
pub unsafe extern "C" fn take_from_pack(
	mut obj: *mut object,
	mut pack: *mut object,
) -> i64 {
	while (*pack).next_object != obj {
		pack = (*pack).next_object;
	}
	(*pack).next_object = (*(*pack).next_object).next_object;
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn pick_up(row: i64, col: i64, mut status: *mut c_short) -> *mut object {
	let obj = object_at(&mut level_objects, row, col);
	*status = 1;
	if (*obj).what_is == Scroll
		&& (*obj).which_kind == SCARE_MONSTER
		&& (*obj).picked_up != 0 {
		message("the scroll turns to dust as you pick it up", 0);
		dungeon[row as usize][col as usize] = dungeon[row as usize][col as usize] & !SpotFlag::Object.code();
		vanish(&mut *obj, false, &mut level_objects);
		*status = 0;
		if id_scrolls[SCARE_MONSTER as usize].id_status == Unidentified {
			id_scrolls[SCARE_MONSTER as usize].id_status = Identified
		}
		return 0 as *mut object;
	}
	if (*obj).what_is == Gold {
		rogue.gold += (*obj).quantity as isize;
		dungeon[row as usize][col as usize] = dungeon[row as usize][col as usize] & !SpotFlag::Object.code();
		take_from_pack(obj, &mut level_objects);
		print_stats(STAT_GOLD);
		return obj;
	}
	if pack_count(obj) >= MAX_PACK_COUNT {
		message("pack too full", 1);
		return 0 as *mut object;
	}
	dungeon[row as usize][col as usize] = dungeon[row as usize][col as usize] & !SpotFlag::Object.code();
	take_from_pack(obj, &mut level_objects);

	let obj = add_to_pack(obj, &mut rogue.pack, 1);
	(*obj).picked_up = 1;
	return obj;
}

#[export_name = "drop"]
pub unsafe extern "C" fn drop_0() {
	let mut obj: *mut object = 0 as *mut object;
	let mut new: *mut object = 0 as *mut object;
	let mut ch: libc::c_short = 0;
	let mut desc: [libc::c_char; 80] = [0; 80];
	if dungeon[rogue.row as usize][rogue.col as usize] as i64
		& (0o1 as libc::c_ushort as i64
		| 0o4 as i64 as libc::c_ushort as i64
		| 0o400 as i64 as libc::c_ushort as i64) != 0
	{
		message("there's already something there", 0 as i64);
		return;
	}
	if (rogue.pack.next_object).is_null() {
		message("you have nothing to drop", 0 as i64);
		return;
	}
	ch = pack_letter("drop what?", AllObjects) as libc::c_short;
	if ch as i64 == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as i64);
	if obj.is_null() {
		message("no such item.", 0 as i64);
		return;
	}
	if (*obj).in_use_flags as i64
		& 0o1 as libc::c_ushort as i64 != 0
	{
		if (*obj).is_cursed != 0 {
			message(curse_message, 0 as i64);
			return;
		}
		unwield(rogue.weapon);
	} else if (*obj).in_use_flags as i64
		& 0o2 as i64 as libc::c_ushort as i64 != 0
	{
		if (*obj).is_cursed != 0 {
			message(curse_message, 0 as i64);
			return;
		}
		mv_aquatars();
		unwear(rogue.armor);
		print_stats(0o20 as i64);
	} else if (*obj).in_use_flags as i64
		& 0o14 as i64 as libc::c_ushort as i64 != 0
	{
		if (*obj).is_cursed != 0 {
			message(curse_message, 0 as i64);
			return;
		}
		un_put_on(obj);
	}
	(*obj).row = rogue.row;
	(*obj).col = rogue.col;
	if (*obj).quantity as i64 > 1
		&& (*obj).what_is as i64
		!= 0o2 as i64 as libc::c_ushort as i64
	{
		(*obj).quantity -= 1;
		(*obj).quantity;
		new = alloc_object();
		*new = *obj;
		(*new).quantity = 1 as libc::c_short;
		obj = new;
	} else {
		(*obj).ichar = 'L' as i32 as libc::c_short;
		take_from_pack(obj, &mut rogue.pack);
	}
	place_at(obj, rogue.row as i64, rogue.col as i64);
	let desc = "dropped ";
	let full_desc = get_desc(obj, desc.as_mut_ptr().offset(8 as i64 as isize));
	message(&full_desc, 0 as i64);
	reg_move();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn check_duplicate(
	mut obj: *mut object,
	mut pack: *mut object,
) -> *mut object {
	let mut op: *mut object = 0 as *mut object;
	if (*obj).what_is as i64
		& (0o2 as i64 as libc::c_ushort as i64
		| 0o40 as i64 as libc::c_ushort as i64
		| 0o4 as i64 as libc::c_ushort as i64
		| 0o10 as i64 as libc::c_ushort as i64) == 0
	{
		return 0 as *mut object;
	}
	if (*obj).what_is as i64
		== 0o40 as i64 as libc::c_ushort as i64
		&& (*obj).which_kind as i64 == 1
	{
		return 0 as *mut object;
	}
	op = (*pack).next_object;
	while !op.is_null() {
		if (*op).what_is as i64 == (*obj).what_is as i64
			&& (*op).which_kind as i64 == (*obj).which_kind as i64
		{
			if (*obj).what_is as i64
				!= 0o2 as i64 as libc::c_ushort as i64
				|| (*obj).what_is as i64
				== 0o2 as i64 as libc::c_ushort as i64
				&& ((*obj).which_kind as i64 == 2 as i64
				|| (*obj).which_kind as i64 == 3 as i64
				|| (*obj).which_kind as i64 == 1
				|| (*obj).which_kind as i64 == 4 as i64)
				&& (*obj).quiver as i64 == (*op).quiver as i64
			{
				(*op)
					.quantity = ((*op).quantity as i64
					+ (*obj).quantity as i64) as libc::c_short;
				return op;
			}
		}
		op = (*op).next_object;
	}
	return 0 as *mut object;
}

pub unsafe fn next_avail_ichar() -> char {
	let mut used = [false; 26];
	{
		let mut obj = rogue.pack.next_object;
		while !obj.is_null() {
			used[(*obj).ichar as u8 - 'a' as u8] = true;
			obj = (*obj).next_object;
		}
	}
	let unused = used.into_iter().position(|used| used == false);
	if let Some(unused) = unused {
		(unused as u8 + 'a' as u8) as char
	} else {
		'?'
	}
}

pub unsafe fn wait_for_ack() {
	while rgetchar() != ' ' {}
}

pub unsafe fn pack_letter(prompt: &str, filter: PackFilter) -> char {
	if !mask_pack(&*rogue.pack, filter.clone()) {
		message("nothing appropriate", 0);
		return CANCEL;
	}

	loop {
		message(prompt, 0);
		let pack_op = {
			let mut pack_op = None;
			loop {
				let ch = rgetchar() as u8 as char;
				pack_op = get_pack_op(ch, filter.clone());
				if pack_op.is_none() {
					sound_bell();
				} else {
					break;
				}
			}
			pack_op.expect("some pack operation")
		};
		check_message();
		match pack_op {
			PackOp::List(filter) => {
				inventory(&*rogue.pack, filter);
			}
			PackOp::Cancel => {
				return CANCEL;
			}
			PackOp::Select(letter) => {
				return letter;
			}
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn take_off() -> i64 {
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut obj: *mut object = 0 as *mut object;
	if !(rogue.armor).is_null() {
		if (*rogue.armor).is_cursed != 0 {
			message(curse_message, 0 as i64);
		} else {
			mv_aquatars();
			obj = rogue.armor;
			unwear(rogue.armor);
			strcpy(
				desc.as_mut_ptr(),
				b"was wearing \0" as *const u8 as *const libc::c_char,
			);
			get_desc(obj, desc.as_mut_ptr().offset(12 as i64 as isize));
			message(desc.as_mut_ptr(), 0 as i64);
			print_stats(0o20 as i64);
			reg_move();
		}
	} else {
		message(
			b"not wearing any\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn wear() -> i64 {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut desc: [libc::c_char; 80] = [0; 80];
	if !(rogue.armor).is_null() {
		message(
			b"your already wearing some\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	ch = pack_letter("wear what?", Armors) as libc::c_short;
	if ch as i64 == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as i64);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*obj).what_is as i64
		!= 0o1 as libc::c_ushort as i64
	{
		message(
			b"you can't wear that\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	(*obj).identified = 1 as libc::c_short;
	strcpy(desc.as_mut_ptr(), b"wearing \0" as *const u8 as *const libc::c_char);
	get_desc(obj, desc.as_mut_ptr().offset(8 as i64 as isize));
	message(desc.as_mut_ptr(), 0 as i64);
	do_wear(&mut *obj);
	print_stats(0o20 as i64);
	reg_move();
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn do_wear(obj: &mut obj) {
	rogue.armor = obj;
	obj.in_use_flags |= BEING_WORN;
	obj.identified = 1;
}

#[no_mangle]
pub unsafe extern "C" fn wield() {
	if !rogue.weapon.is_null() && (*rogue.weapon).is_cursed != 0 {
		message(curse_message, 0);
		return;
	}
	let ch = pack_letter("wield what?", Weapons);

	if ch == CANCEL {
		return;
	}
	let obj = get_letter_object(ch as i64);
	if obj.is_null() {
		message("No such item.", 0);
		return;
	}
	let what = (*obj).what_is;
	if what == Armor || what == Ring {
		let desc = format!("you can't wield {}",
		                   if what == Armor { "armor" } else { "rings" });
		message(&desc,
		        0);
		return;
	}
	if ((*obj).in_use_flags & BEING_WIELDED) != 0 {
		message("in use", 0);
	} else {
		unwield(rogue.weapon);
		let desc = "wielding ";
		let full_desc = get_desc(obj, desc.as_mut_ptr().offset(9 as i64 as isize));
		message(full_desc, 0);
		do_wield(&mut *obj);
		reg_move();
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn do_wield(obj: &mut obj) {
	rogue.weapon = obj;
	obj.in_use_flags |= BEING_WIELDED;
}

#[no_mangle]
pub unsafe extern "C" fn call_it() -> i64 {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut id_table: *mut id = 0 as *mut id;
	let mut buf: [libc::c_char; 32] = [0; 32];
	ch = pack_letter("call what?", AnyFrom(vec![Scroll, Potion, Wand, Ring])) as libc::c_short;
	if ch as i64 == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as i64);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*obj).what_is as i64
		& (0o4 as i64 as libc::c_ushort as i64
		| 0o10 as i64 as libc::c_ushort as i64
		| 0o100 as i64 as libc::c_ushort as i64
		| 0o200 as i64 as libc::c_ushort as i64) == 0
	{
		message(
			b"surely you already know what that's called\0" as *const u8
				as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	id_table = get_id_table(obj);
	if get_input_line(
		b"call it:\0" as *const u8 as *const libc::c_char,
		b"\0" as *const u8 as *const libc::c_char,
		buf.as_mut_ptr(),
		((*id_table.offset((*obj).which_kind as isize)).title).as_mut_ptr(),
		1,
		1,
	) != 0
	{
		(*id_table.offset((*obj).which_kind as isize))
			.id_status = 0o2 as i64 as libc::c_ushort;
		strcpy(
			((*id_table.offset((*obj).which_kind as isize)).title).as_mut_ptr(),
			buf.as_mut_ptr(),
		);
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn pack_count(new_obj: *const obj) -> usize {
	let mut count = 0;
	let mut obj = rogue.pack.next_object;
	while !obj.is_null() {
		if (*obj).what_is != Weapon {
			count += (*obj).quantity;
		} else if new_obj.is_null() {
			count += 1;
		} else if ((*new_obj).what_is != Weapon)
			|| (((*obj).which_kind != weapon_kind::ARROW) && ((*obj).which_kind != weapon_kind::DAGGER) && ((*obj).which_kind != weapon_kind::DART) && ((*obj).which_kind != weapon_kind::SHURIKEN))
			|| ((*new_obj).which_kind != (*obj).which_kind)
			|| ((*obj).quiver != (*new_obj).quiver) {
			count += 1;
		}
		obj = (*obj).next_object;
	}
	count as usize
}


pub unsafe fn mask_pack(pack: *const object, mask: PackFilter) -> bool {
	let mut next = (*pack).next_object;
	while !next.is_null() {
		let what = (*next).what_is;
		if mask.includes(what) {
			return true;
		}
		next = (*next).next_object;
	}
	return false;
}

pub enum PackOp {
	Cancel,
	Select(char),
	List(PackFilter),
}

pub fn get_pack_op(c: char, default_filter: PackFilter) -> Option<PackOp> {
	match c {
		LIST => Some(PackOp::List(default_filter)),
		CANCEL => Some(PackOp::Cancel),
		'?' => Some(PackOp::List(Scrolls)),
		'!' => Some(PackOp::List(Potions)),
		':' => Some(PackOp::List(Foods)),
		')' => Some(PackOp::List(Weapons)),
		']' => Some(PackOp::List(Armors)),
		'/' => Some(PackOp::List(Wands)),
		'=' => Some(PackOp::List(Rings)),
		',' => Some(PackOp::List(Amulets)),
		'a'..='z' => Some(PackOp::Select(c)),
		_ => None
	}
}

pub unsafe fn has_amulet() -> bool {
	mask_pack(&rogue.pack, Amulets)
}

#[no_mangle]
pub unsafe extern "C" fn kick_into_pack() -> i64 {
	let mut obj: *mut object = 0 as *mut object;
	let mut n: libc::c_short = 0;
	let mut stat: libc::c_short = 0;
	if dungeon[rogue.row as usize][rogue.col as usize] as i64
		& 0o1 as libc::c_ushort as i64 == 0
	{
		message("nothing here", 0 as libc::c_int);
	} else {
		obj = pick_up(rogue.row as libc::c_int, rogue.col as libc::c_int, &mut stat);
		if !obj.is_null() {
			let desc = get_desc(obj, "");
			if (*obj).what_is as libc::c_int
				== 0o20 as libc::c_int as libc::c_ushort as libc::c_int
			{
				message(desc, 0 as libc::c_int);
				free_object(obj);
			} else {
				n = strlen(desc.as_mut_ptr()) as libc::c_short;
				desc[n as usize] = '(' as i32 as libc::c_char;
				desc[(n as libc::c_int + 1 as libc::c_int)
					as usize] = (*obj).ichar as libc::c_char;
				desc[(n as libc::c_int + 2 as libc::c_int)
					as usize] = ')' as i32 as libc::c_char;
				desc[(n as libc::c_int + 3 as libc::c_int)
					as usize] = 0 as libc::c_int as libc::c_char;
				message(desc, 0 as libc::c_int);
			}
		}
		if !obj.is_null() || stat == 0 {
			reg_move();
		}
	}
	panic!("Reached end of non-void function without returning");
}
