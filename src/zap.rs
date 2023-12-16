#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_monsters: object;
	fn is_direction() -> libc::c_char;
	fn reg_move() -> libc::c_char;
	fn get_letter_object() -> *mut object;
	fn gr_monster() -> *mut object;
	fn object_at() -> *mut object;
	fn xxx() -> libc::c_long;
	static mut being_held: libc::c_char;
	static mut score_only: libc::c_char;
	static mut detect_monster: libc::c_char;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> libc::c_int;
}

use crate::prelude::*;

pub type chtype = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: libc::c_short,
	pub _curx: libc::c_short,
	pub _maxy: libc::c_short,
	pub _maxx: libc::c_short,
	pub _begy: libc::c_short,
	pub _begx: libc::c_short,
	pub _flags: libc::c_short,
	pub _attrs: attr_t,
	pub _bkgd: chtype,
	pub _notimeout: libc::c_int,
	pub _clear: libc::c_int,
	pub _leaveok: libc::c_int,
	pub _scroll: libc::c_int,
	pub _idlok: libc::c_int,
	pub _idcok: libc::c_int,
	pub _immed: libc::c_int,
	pub _sync: libc::c_int,
	pub _use_keypad: libc::c_int,
	pub _delay: libc::c_int,
	pub _line: *mut ldat,
	pub _regtop: libc::c_short,
	pub _regbottom: libc::c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: libc::c_short,
}

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
pub type attr_t = chtype;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct obj {
	pub m_flags: libc::c_ulong,
	pub damage: *mut libc::c_char,
	pub quantity: libc::c_short,
	pub ichar: libc::c_short,
	pub kill_exp: libc::c_short,
	pub is_protected: libc::c_short,
	pub is_cursed: libc::c_short,
	pub class: libc::c_short,
	pub identified: libc::c_short,
	pub which_kind: libc::c_ushort,
	pub o_row: libc::c_short,
	pub o_col: libc::c_short,
	pub o: libc::c_short,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub d_enchant: libc::c_short,
	pub quiver: libc::c_short,
	pub trow: libc::c_short,
	pub tcol: libc::c_short,
	pub hit_enchant: libc::c_short,
	pub what_is: libc::c_ushort,
	pub picked_up: libc::c_short,
	pub in_use_flags: libc::c_ushort,
	pub next_object: *mut obj,
}

pub type object = obj;

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
pub static mut wizard: libc::c_char = 0 as libc::c_int as libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn zapp() -> libc::c_int {
	let mut wch: libc::c_short = 0;
	let mut first_miss: libc::c_char = 1 as libc::c_int as libc::c_char;
	let mut wand: *mut object = 0 as *mut object;
	let mut dir: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut monster: *mut object = 0 as *mut object;
	loop {
		dir = rgetchar() as libc::c_short;
		if !(is_direction(dir as libc::c_int) == 0) {
			break;
		}
		sound_bell();
		if first_miss != 0 {
			message(
				b"direction? \0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			first_miss = 0 as libc::c_int as libc::c_char;
		}
	}
	check_message();
	if dir as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	wch = pack_letter(
		b"zap with what?\0" as *const u8 as *const libc::c_char,
		0o100 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if wch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	check_message();
	wand = get_letter_object(wch as libc::c_int);
	if wand.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*wand).what_is as libc::c_int
		!= 0o100 as libc::c_int as libc::c_ushort as libc::c_int
	{
		message(
			b"you can't zap with that\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*wand).class as libc::c_int <= 0 as libc::c_int {
		message(
			b"nothing happens\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
	} else {
		(*wand).class -= 1;
		(*wand).class;
		row = rogue.row;
		col = rogue.col;
		monster = get_zapped_monster(dir as libc::c_int, &mut row, &mut col);
		if !monster.is_null() {
			wake_up(monster);
			zap_monster(monster, (*wand).which_kind as libc::c_int);
			relight();
		}
	}
	reg_move();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn get_zapped_monster(
	mut dir: libc::c_short,
	mut row: *mut libc::c_short,
	mut col: *mut libc::c_short,
) -> *mut object {
	let mut orow: libc::c_short = 0;
	let mut ocol: libc::c_short = 0;
	loop {
		orow = *row;
		ocol = *col;
		get_dir_rc(dir as libc::c_int, row, col, 0 as libc::c_int);
		if *row as libc::c_int == orow as libc::c_int
			&& *col as libc::c_int == ocol as libc::c_int
			|| dungeon[*row as usize][*col as usize] as libc::c_int
			& (0o10 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o20 as libc::c_int as libc::c_ushort as libc::c_int) != 0
			|| dungeon[*row as usize][*col as usize] as libc::c_int
			== 0 as libc::c_int as libc::c_ushort as libc::c_int
		{
			return 0 as *mut object;
		}
		if dungeon[*row as usize][*col as usize] as libc::c_int
			& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
		{
			if imitating(*row as libc::c_int, *col as libc::c_int) == 0 {
				return object_at(
					&mut level_monsters,
					*row as libc::c_int,
					*col as libc::c_int,
				);
			}
		}
	};
}

#[no_mangle]
pub unsafe extern "C" fn wizardize() -> libc::c_int {
	let mut buf: [libc::c_char; 100] = [0; 100];
	if wizard != 0 {
		wizard = 0 as libc::c_int as libc::c_char;
		message(
			b"not wizard anymore\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
	} else if get_input_line(
		b"wizard's password:\0" as *const u8 as *const libc::c_char,
		b"\0" as *const u8 as *const libc::c_char,
		buf.as_mut_ptr(),
		b"\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
		0 as libc::c_int,
	) != 0
	{
		xxx(1 as libc::c_int);
		xxxx(buf.as_mut_ptr(), strlen(buf.as_mut_ptr()));
		if strncmp(
			buf.as_mut_ptr(),
			b"\xA7DV\xBAM\xA3\x17\0" as *const u8 as *const libc::c_char,
			7 as libc::c_int as libc::c_ulong,
		) == 0
		{
			wizard = 1 as libc::c_int as libc::c_char;
			score_only = 1 as libc::c_int as libc::c_char;
			message(
				b"Welcome, mighty wizard!\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
		} else {
			message(b"sorry\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
		}
	}
	panic!("Reached end of non-void function without returning");
}
