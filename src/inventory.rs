#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{strcpy, strlen};
use ncurses::{clrtoeol, refresh, waddnstr};
use crate::message;
use crate::pack::wait_for_ack;
use crate::random::get_rand;

extern "C" {
	pub type ldat;

	static mut id_weapons: [id; 0];
	static mut id_armors: [id; 0];
	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn name_of() -> *mut libc::c_char;
	fn get_letter_object() -> *mut object;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> i64;
}

use crate::prelude::*;
use crate::prelude::wand_kind::WANDS;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: libc::c_short,
	pub _pad_x: libc::c_short,
	pub _pad_top: libc::c_short,
	pub _pad_left: libc::c_short,
	pub _pad_bottom: libc::c_short,
	pub _pad_right: libc::c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = ncurses::chtype;


pub static mut is_wood: [bool; WANDS] = [false; WANDS];
#[no_mangle]
pub static mut wand_materials: [*mut libc::c_char; 30] = [
	b"steel \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"bronze \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"gold \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"silver \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"copper \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"nickel \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"cobalt \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"tin \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"iron \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"magnesium \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"chrome \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"carbon \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"platinum \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"silicon \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"titanium \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"teak \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"oak \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"cherry \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"birch \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"pine \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"cedar \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"redwood \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"balsa \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ivory \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"walnut \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"maple \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"mahogany \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"elm \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"palm \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"wooden \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
];
#[no_mangle]
pub static mut gems: [*mut libc::c_char; 14] = [
	b"diamond \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"stibotantalite \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"lapi-lazuli \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ruby \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"emerald \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"sapphire \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"amethyst \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"quartz \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"tiger-eye \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"opal \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"agate \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"turquoise \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"pearl \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"garnet \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
];
#[no_mangle]
pub static mut syllables: [*mut libc::c_char; 40] = [
	b"blech \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"foo \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"barf \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"rech \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"bar \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"blech \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"quo \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"bloto \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"woh \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"caca \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"blorp \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"erp \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"festr \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"rot \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"slie \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"snorf \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"iky \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"yuky \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ooze \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ah \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"bahl \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"zep \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"druhl \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"flem \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"behil \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"arek \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"mep \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"zihr \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"grit \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"kona \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"kini \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ichi \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"niah \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ogr \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ooh \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ighr \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"coph \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"swerr \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"mihln \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"poxi \0" as *const u8 as *const libc::c_char as *mut libc::c_char,
];

#[no_mangle]
pub unsafe extern "C" fn inventory(
	mut pack: *mut object,
	mut mask: libc::c_ushort,
) -> i64 {
	let mut obj: *mut object = 0 as *mut object;
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut maxlen: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut descs: [[libc::c_char; 80]; 25] = [[0; 80]; 25];
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	obj = (*pack).next_object;
	if obj.is_null() {
		message(
			b"your pack is empty\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	while !obj.is_null() {
		if (*obj).what_is as i64 & mask as i64 != 0 {
			descs[i as usize][0 as i64 as usize] = ' ' as i32 as libc::c_char;
			descs[i as usize][1 as usize] = (*obj).ichar as libc::c_char;
			descs[i
				as usize][2 as i64
				as usize] = (if (*obj).what_is as i64
				& 0o1 as libc::c_ushort as i64 != 0
				&& (*obj).is_protected as i64 != 0
			{
				'}' as i32
			} else {
				')' as i32
			}) as libc::c_char;
			descs[i as usize][3 as i64 as usize] = ' ' as i32 as libc::c_char;
			get_desc(
				obj,
				(descs[i as usize]).as_mut_ptr().offset(4 as i64 as isize),
			);
			n = strlen(descs[i as usize].as_mut_ptr()) as libc::c_short;
			if n as i64 > maxlen as i64 {
				maxlen = n;
			}
			i += 1;
			i;
		}
		obj = (*obj).next_object;
	}
	let fresh0 = i;
	i = i + 1;
	strcpy(
		(descs[fresh0 as usize]).as_mut_ptr(),
		b" --press space to continue--\0" as *const u8 as *const libc::c_char,
	);
	if (maxlen as i64) < 27 as i64 {
		maxlen = 27 as i64 as libc::c_short;
	}
	col = (80 as i64 - (maxlen as i64 + 2 as i64))
		as libc::c_short;
	row = 0;
	while (row as i64) < i as i64
		&& (row as i64) < 24 as i64
	{
		if row as i64 > 0 as i64 {
			j = col;
			while (j as i64) < 80 as i64 {
				descs[(row as i64 - 1)
					as usize][(j as i64 - col as i64)
					as usize] = (if ncurses::wmove(ncurses::stdscr(), row as i64, j as i64)
					== -(1)
				{
					-(1) as ncurses::chtype
				} else {
					ncurses::winch(ncurses::stdscr())
				}) as libc::c_char;
				j += 1;
				j;
			}
			descs[(row as i64 - 1)
				as usize][(j as i64 - col as i64)
				as usize] = 0 as i64 as libc::c_char;
		}
		if ncurses::wmove(ncurses::stdscr(), row as i64, col as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), (descs[row as usize]).as_mut_ptr(), -(1));
		};
		clrtoeol();
		row += 1;
		row;
	}
	refresh();
	wait_for_ack();
	ncurses::wmove(ncurses::stdscr(), 0 as i64, 0 as i64);
	clrtoeol();
	j = 1 as libc::c_short;
	while (j as i64) < i as i64 && (j as i64) < 24 as i64
	{
		if ncurses::wmove(ncurses::stdscr(), j as i64, col as i64) == -(1) {
			-(1);
		} else {
			waddnstr(
				ncurses::stdscr(),
				(descs[(j as i64 - 1) as usize]).as_mut_ptr(),
				-(1),
			);
		};
		j += 1;
		j;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn mix_colors() -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut k: libc::c_short = 0;
	let mut t: [libc::c_char; 128] = [0; 128];
	i = 0;
	while i as i64 <= 32 as i64 {
		j = get_rand(0 as i64, 14 as i64 - 1)
			as libc::c_short;
		k = get_rand(0 as i64, 14 as i64 - 1)
			as libc::c_short;
		strcpy(
			t.as_mut_ptr(),
			((*id_potions.as_mut_ptr().offset(j as isize)).title).as_mut_ptr(),
		);
		strcpy(
			((*id_potions.as_mut_ptr().offset(j as isize)).title).as_mut_ptr(),
			((*id_potions.as_mut_ptr().offset(k as isize)).title).as_mut_ptr(),
		);
		strcpy(
			((*id_potions.as_mut_ptr().offset(k as isize)).title).as_mut_ptr(),
			t.as_mut_ptr(),
		);
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn make_scroll_titles() -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut sylls: libc::c_short = 0;
	let mut s: libc::c_short = 0;
	i = 0;
	while (i as i64) < 12 as i64 {
		sylls = get_rand(2 as i64, 5 as i64) as libc::c_short;
		strcpy(
			((*id_scrolls.as_mut_ptr().offset(i as isize)).title).as_mut_ptr(),
			b"'\0" as *const u8 as *const libc::c_char,
		);
		j = 0;
		while (j as i64) < sylls as i64 {
			s = get_rand(1, 40 as i64 - 1)
				as libc::c_short;
			strcat(
				((*id_scrolls.as_mut_ptr().offset(i as isize)).title).as_mut_ptr(),
				syllables[s as usize],
			);
			j += 1;
			j;
		}
		n = strlen(((*id_scrolls.as_mut_ptr().offset(i as isize)).title).as_mut_ptr())
			as libc::c_short;
		strcpy(
			((*id_scrolls.as_mut_ptr().offset(i as isize)).title)
				.as_mut_ptr()
				.offset((n as i64 - 1) as isize),
			b"' \0" as *const u8 as *const libc::c_char,
		);
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn get_wand_and_ring_materials() -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut used: [libc::c_char; 30] = [0; 30];
	i = 0;
	while (i as i64) < 30 as i64 {
		used[i as usize] = 0 as i64 as libc::c_char;
		i += 1;
		i;
	}
	i = 0;
	while (i as i64) < 10 as i64 {
		loop {
			j = get_rand(0 as i64, 30 as i64 - 1)
				as libc::c_short;
			if !(used[j as usize] != 0) {
				break;
			}
		}
		used[j as usize] = 1 as libc::c_char;
		strcpy(
			((*id_wands.as_mut_ptr().offset(i as isize)).title).as_mut_ptr(),
			wand_materials[j as usize],
		);
		is_wood[i
			as usize] = (j as i64 > 14 as i64) as i64
			as libc::c_char;
		i += 1;
		i;
	}
	i = 0;
	while (i as i64) < 14 as i64 {
		used[i as usize] = 0 as i64 as libc::c_char;
		i += 1;
		i;
	}
	i = 0;
	while (i as i64) < 11 {
		loop {
			j = get_rand(0 as i64, 14 as i64 - 1)
				as libc::c_short;
			if !(used[j as usize] != 0) {
				break;
			}
		}
		used[j as usize] = 1 as libc::c_char;
		strcpy(
			((*id_rings.as_mut_ptr().offset(i as isize)).title).as_mut_ptr(),
			gems[j as usize],
		);
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn single_inv(mut ichar: libc::c_short) -> i64 {
	let mut ch: libc::c_short = 0;
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut obj: *mut object = 0 as *mut object;
	ch = (if ichar as i64 != 0 {
		ichar as i64
	} else {
		pack_letter(
			b"inventory what?\0" as *const u8 as *const libc::c_char,
			0o777 as i64 as libc::c_ushort as i64,
		)
	}) as libc::c_short;
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
	desc[0 as i64 as usize] = ch as libc::c_char;
	desc[1
		as usize] = (if (*obj).what_is as i64
		& 0o1 as libc::c_ushort as i64 != 0
		&& (*obj).is_protected as i64 != 0
	{
		'}' as i32
	} else {
		')' as i32
	}) as libc::c_char;
	desc[2 as libc::c_int as usize] = ' ' as i32 as libc::c_char;
	desc[3 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
	get_desc(obj, desc.as_mut_ptr().offset(3 as libc::c_int as isize));
	message(desc.as_mut_ptr(), 0 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn get_id_table(mut obj: *mut object) -> *mut id {
	match (*obj).what_is as libc::c_int {
		4 => return id_scrolls.as_mut_ptr(),
		8 => return id_potions.as_mut_ptr(),
		64 => return id_wands.as_mut_ptr(),
		128 => return id_rings.as_mut_ptr(),
		2 => return id_weapons.as_mut_ptr(),
		1 => return id_armors.as_mut_ptr(),
		_ => {}
	}
	return 0 as *mut id;
}

#[no_mangle]
pub unsafe extern "C" fn inv_armor_weapon(mut is_weapon: libc::c_char) -> libc::c_int {
	if is_weapon != 0 {
		if !(rogue.weapon).is_null() {
			single_inv((*rogue.weapon).ichar as libc::c_int);
		} else {
			message(
				b"not wielding anything\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
		}
	} else if !(rogue.armor).is_null() {
		single_inv((*rogue.armor).ichar as libc::c_int);
	} else {
		message(
			b"not wearing anything\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
	}
	panic!("Reached end of non-void function without returning");
}
