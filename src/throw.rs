#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn wrefresh(_: *mut WINDOW) -> libc::c_int;
	fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_monsters: object;
	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn name_of() -> *mut libc::c_char;
	fn is_direction() -> libc::c_char;
	fn alloc_object() -> *mut object;
	fn get_letter_object() -> *mut object;
	fn object_at() -> *mut object;
	static mut curse_message: *mut libc::c_char;
	static mut hit_message: [libc::c_char; 0];
}

use crate::prelude::*;
use crate::throw::Move::{Up, UpLeft, UpRight, Left, Right, Same, Down, DownLeft, DownRight};


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
pub unsafe extern "C" fn throw() -> libc::c_int {
	let mut wch: libc::c_short = 0;
	let mut first_miss: libc::c_char = 1 as libc::c_int as libc::c_char;
	let mut weapon: *mut object = 0 as *mut object;
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
		b"throw what?\0" as *const u8 as *const libc::c_char,
		0o2 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if wch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	check_message();
	weapon = get_letter_object(wch as libc::c_int);
	if weapon.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*weapon).in_use_flags as libc::c_int
		& 0o17 as libc::c_int as libc::c_ushort as libc::c_int != 0
		&& (*weapon).is_cursed as libc::c_int != 0
	{
		message(curse_message, 0 as libc::c_int);
		return;
	}
	row = rogue.row;
	col = rogue.col;
	if (*weapon).in_use_flags as libc::c_int
		& 0o1 as libc::c_int as libc::c_ushort as libc::c_int != 0
		&& (*weapon).quantity as libc::c_int <= 1 as libc::c_int
	{
		unwield(rogue.weapon);
	} else if (*weapon).in_use_flags as libc::c_int
		& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		mv_aquatars();
		unwear(rogue.armor);
		print_stats(0o20 as libc::c_int);
	} else if (*weapon).in_use_flags as libc::c_int
		& 0o14 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		un_put_on(weapon);
	}
	monster = get_thrown_at_monster(weapon, dir as libc::c_int, &mut row, &mut col);
	if wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int)
		== -(1 as libc::c_int)
	{
		-(1 as libc::c_int);
	} else {
		waddch(stdscr, rogue.fchar as chtype);
	};
	wrefresh(stdscr);
	if rogue_can_see(row as libc::c_int, col as libc::c_int) != 0
		&& (row as libc::c_int != rogue.row as libc::c_int
		|| col as libc::c_int != rogue.col as libc::c_int)
	{
		if wmove(stdscr, row as libc::c_int, col as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddch(
				stdscr,
				get_dungeon_char(row as libc::c_int, col as libc::c_int) as chtype,
			);
		};
	}
	if !monster.is_null() {
		wake_up(monster);
		check_gold_seeker(monster);
		if throw_at_monster(monster, weapon) == 0 {
			flop_weapon(weapon, row as libc::c_int, col as libc::c_int);
		}
	} else {
		flop_weapon(weapon, row as libc::c_int, col as libc::c_int);
	}
	vanish(weapon, 1 as libc::c_int, &mut rogue.pack);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn get_thrown_at_monster(
	mut obj: *mut object,
	mut dir: libc::c_short,
	mut row: *mut libc::c_short,
	mut col: *mut libc::c_short,
) -> *mut object {
	let mut orow: libc::c_short = 0;
	let mut ocol: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut ch: libc::c_short = 0;
	orow = *row;
	ocol = *col;
	ch = get_mask_char((*obj).what_is as libc::c_int) as libc::c_short;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 24 as libc::c_int {
		get_dir_rc(dir as libc::c_int, row, col, 0 as libc::c_int);
		if dungeon[*row as usize][*col as usize] as libc::c_int
			== 0 as libc::c_int as libc::c_ushort as libc::c_int
			|| dungeon[*row as usize][*col as usize] as libc::c_int
			& (0o10 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o20 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o1000 as libc::c_int as libc::c_ushort as libc::c_int) != 0
			&& dungeon[*row as usize][*col as usize] as libc::c_int
			& 0o400 as libc::c_int as libc::c_ushort as libc::c_int == 0
		{
			*row = orow;
			*col = ocol;
			return 0 as *mut object;
		}
		if i as libc::c_int != 0 as libc::c_int
			&& rogue_can_see(orow as libc::c_int, ocol as libc::c_int) != 0
		{
			if wmove(stdscr, orow as libc::c_int, ocol as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				waddch(
					stdscr,
					get_dungeon_char(orow as libc::c_int, ocol as libc::c_int) as chtype,
				);
			};
		}
		if rogue_can_see(*row as libc::c_int, *col as libc::c_int) != 0 {
			if dungeon[*row as usize][*col as usize] as libc::c_int
				& 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
			{
				if wmove(stdscr, *row as libc::c_int, *col as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					waddch(stdscr, ch as chtype);
				};
			}
			wrefresh(stdscr);
		}
		orow = *row;
		ocol = *col;
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
		if dungeon[*row as usize][*col as usize] as libc::c_int
			& 0o200 as libc::c_int as libc::c_ushort as libc::c_int != 0
		{
			i = (i as libc::c_int + 2 as libc::c_int) as libc::c_short;
		}
		i += 1;
		i;
	}
	return 0 as *mut object;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Move {
	DownRight,
	DownLeft,
	UpRight,
	UpLeft,
	Right,
	Down,
	Same,
	Up,
	Left,
}

impl Move {
	pub fn delta(&self) -> (isize, isize) {
		match self {
			DownRight => (1, 1),
			DownLeft => (1, -1),
			UpRight => (-1, 1),
			UpLeft => (-1, -1),
			Right => (0, 1),
			Down => (1, 0),
			Same => (0, 0),
			Up => (-1, 0),
			Left => (0, -1),
		}
	}
	pub fn apply(&self, row: isize, col: isize) -> (isize, isize) {
		let (r_delta, c_delta) = self.delta();
		(row + r_delta, col + c_delta)
	}
}

pub unsafe fn rand_around(i: u8, r: isize, c: isize) -> (isize, isize) {
	static mut moves: [Move; 9] = [Left, Up, DownLeft, UpLeft, Right, Down, UpRight, Same, DownRight];
	static mut row: usize = 0;
	static mut col: usize = 0;

	if i == 0 {
		row = *r;
		col = *c;
		let o = get_rand(1, 8);
		for _j in 0..5 {
			let x = get_rand(0, 8) as usize;
			let y = (x + o) % 9;
			let t = moves[x];
			moves[x] = moves[y];
			moves[y] = t;
		}
	}
	moves[i as usize].apply(r, c)
}
