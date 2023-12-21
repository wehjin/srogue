#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;


	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn name_of() -> *mut libc::c_char;
	fn is_direction() -> libc::c_char;
	fn alloc_object() -> *mut object;
	fn get_letter_object() -> *mut object;
}

use ncurses::addch;
use crate::prelude::*;
use crate::throw::Move::{Up, UpLeft, UpRight, Left, Right, Same, Down, DownLeft, DownRight};




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



#[no_mangle]
pub unsafe extern "C" fn throw() -> i64 {
	let mut wch: libc::c_short = 0;
	let mut first_miss: libc::c_char = 1 as libc::c_char;
	let mut weapon: *mut object = 0 as *mut object;
	let mut dir: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut monster: *mut object = 0 as *mut object;
	loop {
		dir = rgetchar() as libc::c_short;
		if !(is_direction(dir as i32) == 0) {
			break;
		}
		sound_bell();
		if first_miss != 0 {
			message(
				b"direction? \0" as *const u8 as *const libc::c_char,
				0 as i64,
			);
			first_miss = 0 as i64 as libc::c_char;
		}
	}
	check_message();
	if dir as i64 == '\u{1b}' as i32 {
		return;
	}
	wch = pack_letter(
		b"throw what?\0" as *const u8 as *const libc::c_char,
		0o2 as i64 as libc::c_ushort as i64,
	) as libc::c_short;
	if wch as i64 == '\u{1b}' as i32 {
		return;
	}
	check_message();
	weapon = get_letter_object(wch as i64);
	if weapon.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*weapon).in_use_flags as i64
		& 0o17 as i64 as libc::c_ushort as i64 != 0
		&& (*weapon).is_cursed as i64 != 0
	{
		message(curse_message, 0 as i64);
		return;
	}
	row = rogue.row;
	col = rogue.col;
	if (*weapon).in_use_flags as i64
		& 0o1 as libc::c_ushort as i64 != 0
		&& (*weapon).quantity as i64 <= 1
	{
		unwield(rogue.weapon);
	} else if (*weapon).in_use_flags as i64
		& 0o2 as i64 as libc::c_ushort as i64 != 0
	{
		mv_aquatars();
		unwear(rogue.armor);
		print_stats(0o20 as i64);
	} else if (*weapon).in_use_flags as i64
		& 0o14 as i64 as libc::c_ushort as i64 != 0
	{
		un_put_on(weapon);
	}
	monster = get_thrown_at_monster(weapon, dir as i64, &mut row, &mut col);
	if ncurses::wmove(ncurses::stdscr(), rogue.row as i64, rogue.col as i64)
		== -(1)
	{
		-(1);
	} else {
		addch(rogue.fchar as ncurses::chtype);
	};
	ncurses::refresh();
	if rogue_can_see(row as i64, col as i64) != 0
		&& (row as i64 != rogue.row as i64
		|| col as i64 != rogue.col as i64)
	{
		if ncurses::wmove(ncurses::stdscr(), row as i64, col as i64) == -(1) {
			-(1);
		} else {
			addch(get_dungeon_char(row as usize, col as usize) as ncurses::chtype);
		};
	}
	if !monster.is_null() {
		wake_up(monster);
		check_gold_seeker(monster);
		if throw_at_monster(monster, weapon) == 0 {
			flop_weapon(weapon, row as i64, col as i64);
		}
	} else {
		flop_weapon(weapon, row as i64, col as i64);
	}
	vanish(weapon, 1, &mut rogue.pack);
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
	ch = get_mask_char((*obj).what_is as i64) as libc::c_short;
	i = 0 as i64 as libc::c_short;
	while (i as i64) < 24 as i64 {
		get_dir_rc(dir as i64, row, col, 0 as i64);
		if dungeon[*row as usize][*col as usize] as i64
			== 0 as i64 as libc::c_ushort as i64
			|| dungeon[*row as usize][*col as usize] as i64
			& (0o10 as i64 as libc::c_ushort as i64
			| 0o20 as i64 as libc::c_ushort as i64
			| 0o1000 as i64 as libc::c_ushort as i64) != 0
			&& dungeon[*row as usize][*col as usize] as i64
			& 0o400 as i64 as libc::c_ushort as i64 == 0
		{
			*row = orow;
			*col = ocol;
			return 0 as *mut object;
		}
		if i as i64 != 0 as i64
			&& rogue_can_see(orow as i64, ocol as i64) != 0
		{
			if ncurses::wmove(ncurses::stdscr(), orow as i64, ocol as i64)
				== -(1)
			{
				-(1);
			} else {
				addch(get_dungeon_char(orow as usize, ocol as usize) as ncurses::chtype);
			};
		}
		if rogue_can_see(*row as i64, *col as i64) != 0 {
			if dungeon[*row as usize][*col as usize] as i64
				& 0o2 as i64 as libc::c_ushort as i64 == 0
			{
				if ncurses::wmove(ncurses::stdscr(), *row as i64, *col as i64)
					== -(1)
				{
					-(1);
				} else {
					addch(ch as ncurses::chtype);
				};
			}
			ncurses::refresh();
		}
		orow = *row;
		ocol = *col;
		if dungeon[*row as usize][*col as usize] as i64
			& 0o2 as i64 as libc::c_ushort as i64 != 0
		{
			if imitating(*row, *col) == 0 {
				return object_at(&mut level_monsters, *row, *col);
			}
		}
		if dungeon[*row as usize][*col as usize] as i64
			& 0o200 as i64 as libc::c_ushort as i64 != 0
		{
			i = (i as i64 + 2 as i64) as libc::c_short;
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
