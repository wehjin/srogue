#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use crate::{get_input_line, message, mv_aquatars, print_stats};
use crate::objects::place_at;

extern "C" {
	fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
	static mut rogue: fighter;
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	static mut id_scrolls: [id; 0];
	fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn reg_move() -> libc::c_char;
	fn alloc_object() -> *mut object;
	fn get_letter_object() -> *mut object;
	fn object_at() -> *mut object;
	fn get_id_table() -> *mut id;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
}

use crate::prelude::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct id {
	pub value: libc::c_short,
	pub title: [libc::c_char; 128],
	pub real: [libc::c_char; 128],
	pub id_status: libc::c_ushort,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: libc::c_short,
	pub hp_max: libc::c_short,
	pub str_current: libc::c_short,
	pub str_max: libc::c_short,
	pub pack: object,
	pub gold: libc::c_long,
	pub exp: libc::c_short,
	pub exp_points: libc::c_long,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub fchar: libc::c_short,
	pub moves_left: libc::c_short,
}

pub type fighter = fight;

#[no_mangle]
pub static mut curse_message: *mut libc::c_char = b"you can't, it appears to be cursed\0"
	as *const u8 as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn add_to_pack(
	mut obj: *mut object,
	mut pack: *mut object,
	mut condense: libc::c_int,
) -> *mut object {
	let mut op: *mut object = 0 as *mut object;
	if condense != 0 {
		op = check_duplicate(obj, pack);
		if !op.is_null() {
			free_object(obj);
			return op;
		} else {
			(*obj).ichar = next_avail_ichar() as libc::c_short;
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
) -> libc::c_int {
	while (*pack).next_object != obj {
		pack = (*pack).next_object;
	}
	(*pack).next_object = (*(*pack).next_object).next_object;
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn pick_up(
	mut row: libc::c_int,
	mut col: libc::c_int,
	mut status: *mut libc::c_short,
) -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = object_at(&mut level_objects, row, col);
	*status = 1 as libc::c_int as libc::c_short;
	if (*obj).what_is as libc::c_int
		== 0o4 as libc::c_int as libc::c_ushort as libc::c_int
		&& (*obj).which_kind as libc::c_int == 7 as libc::c_int
		&& (*obj).picked_up as libc::c_int != 0
	{
		message(
			b"the scroll turns to dust as you pick it up\0" as *const u8
				as *const libc::c_char,
			0 as libc::c_int,
		);
		dungeon[row
			as usize][col
			as usize] = (dungeon[row as usize][col as usize] as libc::c_int
			& !(0o1 as libc::c_int as libc::c_ushort as libc::c_int)) as libc::c_ushort;
		vanish(obj, 0 as libc::c_int, &mut level_objects);
		*status = 0 as libc::c_int as libc::c_short;
		if (*id_scrolls.as_mut_ptr().offset(7 as libc::c_int as isize)).id_status
			as libc::c_int == 0 as libc::c_int as libc::c_ushort as libc::c_int
		{
			(*id_scrolls.as_mut_ptr().offset(7 as libc::c_int as isize))
				.id_status = 0o1 as libc::c_int as libc::c_ushort;
		}
		return 0 as *mut object;
	}
	if (*obj).what_is as libc::c_int
		== 0o20 as libc::c_int as libc::c_ushort as libc::c_int
	{
		rogue.gold += (*obj).quantity as libc::c_long;
		dungeon[row
			as usize][col
			as usize] = (dungeon[row as usize][col as usize] as libc::c_int
			& !(0o1 as libc::c_int as libc::c_ushort as libc::c_int)) as libc::c_ushort;
		take_from_pack(obj, &mut level_objects);
		print_stats(0o2 as libc::c_int);
		return obj;
	}
	if pack_count(obj) >= 24 as libc::c_int {
		message(
			b"pack too full\0" as *const u8 as *const libc::c_char,
			1 as libc::c_int,
		);
		return 0 as *mut object;
	}
	dungeon[row
		as usize][col
		as usize] = (dungeon[row as usize][col as usize] as libc::c_int
		& !(0o1 as libc::c_int as libc::c_ushort as libc::c_int)) as libc::c_ushort;
	take_from_pack(obj, &mut level_objects);
	obj = add_to_pack(obj, &mut rogue.pack, 1 as libc::c_int);
	(*obj).picked_up = 1 as libc::c_int as libc::c_short;
	return obj;
}

#[export_name = "drop"]
pub unsafe extern "C" fn drop_0() -> libc::c_int {
	let mut obj: *mut object = 0 as *mut object;
	let mut new: *mut object = 0 as *mut object;
	let mut ch: libc::c_short = 0;
	let mut desc: [libc::c_char; 80] = [0; 80];
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& (0o1 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o400 as libc::c_int as libc::c_ushort as libc::c_int) != 0
	{
		message(
			b"there's already something there\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (rogue.pack.next_object).is_null() {
		message(
			b"you have nothing to drop\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	ch = pack_letter(
		b"drop what?\0" as *const u8 as *const libc::c_char,
		0o777 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).in_use_flags as libc::c_int
		& 0o1 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		if (*obj).is_cursed != 0 {
			message(curse_message, 0 as libc::c_int);
			return;
		}
		unwield(rogue.weapon);
	} else if (*obj).in_use_flags as libc::c_int
		& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		if (*obj).is_cursed != 0 {
			message(curse_message, 0 as libc::c_int);
			return;
		}
		mv_aquatars();
		unwear(rogue.armor);
		print_stats(0o20 as libc::c_int);
	} else if (*obj).in_use_flags as libc::c_int
		& 0o14 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		if (*obj).is_cursed != 0 {
			message(curse_message, 0 as libc::c_int);
			return;
		}
		un_put_on(obj);
	}
	(*obj).row = rogue.row;
	(*obj).col = rogue.col;
	if (*obj).quantity as libc::c_int > 1 as libc::c_int
		&& (*obj).what_is as libc::c_int
		!= 0o2 as libc::c_int as libc::c_ushort as libc::c_int
	{
		(*obj).quantity -= 1;
		(*obj).quantity;
		new = alloc_object();
		*new = *obj;
		(*new).quantity = 1 as libc::c_int as libc::c_short;
		obj = new;
	} else {
		(*obj).ichar = 'L' as i32 as libc::c_short;
		take_from_pack(obj, &mut rogue.pack);
	}
	place_at(obj, rogue.row as libc::c_int, rogue.col as libc::c_int);
	strcpy(desc.as_mut_ptr(), b"dropped \0" as *const u8 as *const libc::c_char);
	get_desc(obj, desc.as_mut_ptr().offset(8 as libc::c_int as isize));
	message(desc.as_mut_ptr(), 0 as libc::c_int);
	reg_move();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn check_duplicate(
	mut obj: *mut object,
	mut pack: *mut object,
) -> *mut object {
	let mut op: *mut object = 0 as *mut object;
	if (*obj).what_is as libc::c_int
		& (0o2 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o40 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o10 as libc::c_int as libc::c_ushort as libc::c_int) == 0
	{
		return 0 as *mut object;
	}
	if (*obj).what_is as libc::c_int
		== 0o40 as libc::c_int as libc::c_ushort as libc::c_int
		&& (*obj).which_kind as libc::c_int == 1 as libc::c_int
	{
		return 0 as *mut object;
	}
	op = (*pack).next_object;
	while !op.is_null() {
		if (*op).what_is as libc::c_int == (*obj).what_is as libc::c_int
			&& (*op).which_kind as libc::c_int == (*obj).which_kind as libc::c_int
		{
			if (*obj).what_is as libc::c_int
				!= 0o2 as libc::c_int as libc::c_ushort as libc::c_int
				|| (*obj).what_is as libc::c_int
				== 0o2 as libc::c_int as libc::c_ushort as libc::c_int
				&& ((*obj).which_kind as libc::c_int == 2 as libc::c_int
				|| (*obj).which_kind as libc::c_int == 3 as libc::c_int
				|| (*obj).which_kind as libc::c_int == 1 as libc::c_int
				|| (*obj).which_kind as libc::c_int == 4 as libc::c_int)
				&& (*obj).quiver as libc::c_int == (*op).quiver as libc::c_int
			{
				(*op)
					.quantity = ((*op).quantity as libc::c_int
					+ (*obj).quantity as libc::c_int) as libc::c_short;
				return op;
			}
		}
		op = (*op).next_object;
	}
	return 0 as *mut object;
}

#[no_mangle]
pub unsafe extern "C" fn wait_for_ack() -> libc::c_int {
	while rgetchar() != ' ' as i32 {}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn take_off() -> libc::c_int {
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut obj: *mut object = 0 as *mut object;
	if !(rogue.armor).is_null() {
		if (*rogue.armor).is_cursed != 0 {
			message(curse_message, 0 as libc::c_int);
		} else {
			mv_aquatars();
			obj = rogue.armor;
			unwear(rogue.armor);
			strcpy(
				desc.as_mut_ptr(),
				b"was wearing \0" as *const u8 as *const libc::c_char,
			);
			get_desc(obj, desc.as_mut_ptr().offset(12 as libc::c_int as isize));
			message(desc.as_mut_ptr(), 0 as libc::c_int);
			print_stats(0o20 as libc::c_int);
			reg_move();
		}
	} else {
		message(
			b"not wearing any\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn wear() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut desc: [libc::c_char; 80] = [0; 80];
	if !(rogue.armor).is_null() {
		message(
			b"your already wearing some\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	ch = pack_letter(
		b"wear what?\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
		0o1 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).what_is as libc::c_int
		!= 0o1 as libc::c_int as libc::c_ushort as libc::c_int
	{
		message(
			b"you can't wear that\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	(*obj).identified = 1 as libc::c_int as libc::c_short;
	strcpy(desc.as_mut_ptr(), b"wearing \0" as *const u8 as *const libc::c_char);
	get_desc(obj, desc.as_mut_ptr().offset(8 as libc::c_int as isize));
	message(desc.as_mut_ptr(), 0 as libc::c_int);
	do_wear(obj);
	print_stats(0o20 as libc::c_int);
	reg_move();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn wield() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut desc: [libc::c_char; 80] = [0; 80];
	if !(rogue.weapon).is_null() && (*rogue.weapon).is_cursed as libc::c_int != 0 {
		message(curse_message, 0 as libc::c_int);
		return;
	}
	ch = pack_letter(
		b"wield what?\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
		0o2 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"No such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).what_is as libc::c_int
		& (0o1 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o200 as libc::c_int as libc::c_ushort as libc::c_int) != 0
	{
		sprintf(
			desc.as_mut_ptr(),
			b"you can't wield %s\0" as *const u8 as *const libc::c_char,
			if (*obj).what_is as libc::c_int
				== 0o1 as libc::c_int as libc::c_ushort as libc::c_int
			{
				b"armor\0" as *const u8 as *const libc::c_char
			} else {
				b"rings\0" as *const u8 as *const libc::c_char
			},
		);
		message(desc.as_mut_ptr(), 0 as libc::c_int);
		return;
	}
	if (*obj).in_use_flags as libc::c_int
		& 0o1 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		message(b"in use\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
	} else {
		unwield(rogue.weapon);
		strcpy(desc.as_mut_ptr(), b"wielding \0" as *const u8 as *const libc::c_char);
		get_desc(obj, desc.as_mut_ptr().offset(9 as libc::c_int as isize));
		message(desc.as_mut_ptr(), 0 as libc::c_int);
		do_wield(obj);
		reg_move();
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn call_it() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut id_table: *mut id = 0 as *mut id;
	let mut buf: [libc::c_char; 32] = [0; 32];
	ch = pack_letter(
		b"call what?\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
		0o4 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o10 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o100 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o200 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).what_is as libc::c_int
		& (0o4 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o10 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o100 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o200 as libc::c_int as libc::c_ushort as libc::c_int) == 0
	{
		message(
			b"surely you already know what that's called\0" as *const u8
				as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	id_table = get_id_table(obj);
	if get_input_line(
		b"call it:\0" as *const u8 as *const libc::c_char,
		b"\0" as *const u8 as *const libc::c_char,
		buf.as_mut_ptr(),
		((*id_table.offset((*obj).which_kind as isize)).title).as_mut_ptr(),
		1 as libc::c_int,
		1 as libc::c_int,
	) != 0
	{
		(*id_table.offset((*obj).which_kind as isize))
			.id_status = 0o2 as libc::c_int as libc::c_ushort;
		strcpy(
			((*id_table.offset((*obj).which_kind as isize)).title).as_mut_ptr(),
			buf.as_mut_ptr(),
		);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn mask_pack(
	mut pack: *mut object,
	mut mask: libc::c_ushort,
) -> libc::c_char {
	while !((*pack).next_object).is_null() {
		pack = (*pack).next_object;
		if (*pack).what_is as libc::c_int & mask as libc::c_int != 0 {
			return 1 as libc::c_int as libc::c_char;
		}
	}
	return 0 as libc::c_int as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn has_amulet() -> libc::c_int {
	return mask_pack(
		&mut rogue.pack,
		0o400 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn kick_into_pack() -> libc::c_int {
	let mut obj: *mut object = 0 as *mut object;
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut n: libc::c_short = 0;
	let mut stat: libc::c_short = 0;
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o1 as libc::c_int as libc::c_ushort as libc::c_int == 0
	{
		message(b"nothing here\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
	} else {
		obj = pick_up(rogue.row as libc::c_int, rogue.col as libc::c_int, &mut stat);
		if !obj.is_null() {
			get_desc(obj, desc.as_mut_ptr());
			if (*obj).what_is as libc::c_int
				== 0o20 as libc::c_int as libc::c_ushort as libc::c_int
			{
				message(desc.as_mut_ptr(), 0 as libc::c_int);
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
				message(desc.as_mut_ptr(), 0 as libc::c_int);
			}
		}
		if !obj.is_null() || stat == 0 {
			reg_move();
		}
	}
	panic!("Reached end of non-void function without returning");
}
