#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::os::raw::c_int;
use crate::message::{message, print_stats};
use crate::monster::wake_room;
use crate::objects::put_amulet;
use crate::pack::has_amulet;
use crate::random::{get_rand, rand_percent};
use crate::room::{gr_row_col, is_all_connected, light_passage, light_up_room};
use crate::score::win;

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn wclear(_: *mut WINDOW) -> libc::c_int;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut rooms: [room; 0];
	static mut traps: [trap; 0];
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut being_held: libc::c_char;
	static mut wizard: libc::c_char;
	static mut detect_monster: libc::c_char;
	static mut see_invisible: libc::c_char;
	static mut bear_trap: libc::c_short;
	static mut levitate: libc::c_short;
	static mut extra_hp: libc::c_short;
	static mut less_hp: libc::c_short;
	static mut party_counter: libc::c_short;
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dr {
	pub oth_room: libc::c_short,
	pub oth_row: libc::c_short,
	pub oth_col: libc::c_short,
	pub door_row: libc::c_short,
	pub door_col: libc::c_short,
}

pub type door = dr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct rm {
	pub bottom_row: libc::c_char,
	pub right_col: libc::c_char,
	pub left_col: libc::c_char,
	pub top_row: libc::c_char,
	pub doors: [door; 4],
	pub is_room: libc::c_ushort,
}

pub type room = rm;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tr {
	pub trap_type: libc::c_short,
	pub trap_row: libc::c_short,
	pub trap_col: libc::c_short,
}

pub type trap = tr;

#[no_mangle]
pub static mut cur_level: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut max_level: libc::c_short = 1 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut cur_room: libc::c_short = 0;
#[no_mangle]
pub static mut new_level_message: *mut libc::c_char = 0 as *const libc::c_char
	as *mut libc::c_char;
#[no_mangle]
pub static mut party_room: libc::c_short = -(1 as libc::c_int) as libc::c_short;
#[no_mangle]
pub static mut r_de: libc::c_short = 0;
#[no_mangle]
pub static mut level_points: [libc::c_long; 21] = [
	10 as libc::c_long,
	20 as libc::c_long,
	40 as libc::c_long,
	80 as libc::c_long,
	160 as libc::c_long,
	320 as libc::c_long,
	640 as libc::c_long,
	1300 as libc::c_long,
	2600 as libc::c_long,
	5200 as libc::c_long,
	10000 as libc::c_long,
	20000 as libc::c_long,
	40000 as libc::c_long,
	80000 as libc::c_long,
	160000 as libc::c_long,
	320000 as libc::c_long,
	1000000 as libc::c_long,
	3333333 as libc::c_long,
	6666666 as libc::c_long,
	10000000 as libc::c_long,
	99900000 as libc::c_long,
];
#[no_mangle]
pub static mut random_rooms: [libc::c_char; 10] = [
	3 as libc::c_int as libc::c_char,
	7 as libc::c_int as libc::c_char,
	5 as libc::c_int as libc::c_char,
	2 as libc::c_int as libc::c_char,
	0 as libc::c_int as libc::c_char,
	6 as libc::c_int as libc::c_char,
	1 as libc::c_int as libc::c_char,
	4 as libc::c_int as libc::c_char,
	8 as libc::c_int as libc::c_char,
	0,
];

#[no_mangle]
pub unsafe extern "C" fn make_level() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut must_exist1: libc::c_short = 0;
	let mut must_exist2: libc::c_short = 0;
	let mut must_exist3: libc::c_short = 0;
	let mut big_room: libc::c_char = 0;
	if (cur_level as libc::c_int) < 99 as libc::c_int {
		cur_level += 1;
		cur_level;
	}
	if cur_level as libc::c_int > max_level as libc::c_int {
		max_level = cur_level;
	}
	must_exist1 = get_rand(0 as libc::c_int, 5 as libc::c_int) as libc::c_short;
	match must_exist1 as libc::c_int {
		0 => {
			must_exist1 = 0 as libc::c_int as libc::c_short;
			must_exist2 = 1 as libc::c_int as libc::c_short;
			must_exist3 = 2 as libc::c_int as libc::c_short;
		}
		1 => {
			must_exist1 = 3 as libc::c_int as libc::c_short;
			must_exist2 = 4 as libc::c_int as libc::c_short;
			must_exist3 = 5 as libc::c_int as libc::c_short;
		}
		2 => {
			must_exist1 = 6 as libc::c_int as libc::c_short;
			must_exist2 = 7 as libc::c_int as libc::c_short;
			must_exist3 = 8 as libc::c_int as libc::c_short;
		}
		3 => {
			must_exist1 = 0 as libc::c_int as libc::c_short;
			must_exist2 = 3 as libc::c_int as libc::c_short;
			must_exist3 = 6 as libc::c_int as libc::c_short;
		}
		4 => {
			must_exist1 = 1 as libc::c_int as libc::c_short;
			must_exist2 = 4 as libc::c_int as libc::c_short;
			must_exist3 = 7 as libc::c_int as libc::c_short;
		}
		5 => {
			must_exist1 = 2 as libc::c_int as libc::c_short;
			must_exist2 = 5 as libc::c_int as libc::c_short;
			must_exist3 = 8 as libc::c_int as libc::c_short;
		}
		_ => {}
	}
	big_room = (cur_level as libc::c_int == party_counter as libc::c_int
		&& rand_percent(1 as libc::c_int)) as libc::c_int as libc::c_char;
	if big_room != 0 {
		make_room(
			10 as libc::c_int,
			0 as libc::c_int,
			0 as libc::c_int,
			0 as libc::c_int,
		);
	} else {
		i = 0 as libc::c_int as libc::c_short;
		while (i as libc::c_int) < 9 as libc::c_int {
			make_room(
				i as libc::c_int,
				must_exist1 as libc::c_int,
				must_exist2 as libc::c_int,
				must_exist3 as libc::c_int,
			);
			i += 1;
			i;
		}
	}
	if big_room == 0 {
		add_mazes();
		mix_random_rooms();
		j = 0 as libc::c_int as libc::c_short;
		while (j as libc::c_int) < 9 as libc::c_int {
			i = random_rooms[j as usize] as libc::c_short;
			if (i as libc::c_int) < 9 as libc::c_int - 1 as libc::c_int {
				connect_rooms(i as libc::c_int, i as libc::c_int + 1 as libc::c_int);
			}
			if (i as libc::c_int) < 9 as libc::c_int - 3 as libc::c_int {
				connect_rooms(i as libc::c_int, i as libc::c_int + 3 as libc::c_int);
			}
			if (i as libc::c_int) < 9 as libc::c_int - 2 as libc::c_int {
				if (*rooms
					.as_mut_ptr()
					.offset((i as libc::c_int + 1 as libc::c_int) as isize))
					.is_room as libc::c_int
					& 0o1 as libc::c_int as libc::c_ushort as libc::c_int != 0
				{
					if connect_rooms(
						i as libc::c_int,
						i as libc::c_int + 2 as libc::c_int,
					) != 0
					{
						(*rooms
							.as_mut_ptr()
							.offset((i as libc::c_int + 1 as libc::c_int) as isize))
							.is_room = 0o20 as libc::c_int as libc::c_ushort;
					}
				}
			}
			if (i as libc::c_int) < 9 as libc::c_int - 6 as libc::c_int {
				if (*rooms
					.as_mut_ptr()
					.offset((i as libc::c_int + 3 as libc::c_int) as isize))
					.is_room as libc::c_int
					& 0o1 as libc::c_int as libc::c_ushort as libc::c_int != 0
				{
					if connect_rooms(
						i as libc::c_int,
						i as libc::c_int + 6 as libc::c_int,
					) != 0
					{
						(*rooms
							.as_mut_ptr()
							.offset((i as libc::c_int + 3 as libc::c_int) as isize))
							.is_room = 0o20 as libc::c_int as libc::c_ushort;
					}
				}
			}
			if is_all_connected() != 0 {
				break;
			}
			j += 1;
			j;
		}
		fill_out_level();
	}
	if has_amulet() == 0 && cur_level as libc::c_int >= 26 as libc::c_int {
		put_amulet();
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn make_room(rn: libc::c_int, r1: libc::c_int, r2: libc::c_int, r3: libc::c_int) {
	let rn: usize = rn as usize;
	let (left, right, top, bottom, do_shrink, room_index) =
		match rn {
			0 => (0 as c_int, COL1 - 1 as c_int, MIN_ROW, ROW1 - 1 as c_int, true, rn),
			1 => (COL1 + 1, COL2 - 1, MIN_ROW, ROW1 - 1, true, rn),
			2 => (COL2 + 1, DCOLS - 1, MIN_ROW, ROW1 - 1, true, rn),
			3 => (0, COL1 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			4 => (COL1 + 1, COL2 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			5 => (COL2 + 1, DCOLS - 1, ROW1 + 1, ROW2 - 1, true, rn),
			6 => (0, COL1 - 1, ROW2 + 1, DROWS - 2, true, rn),
			7 => (COL1 + 1, COL2 - 1, ROW2 + 1, DROWS - 2, true, rn),
			8 => (COL2 + 1, DCOLS - 1, ROW2 + 1, DROWS - 2, true, rn),
			BIG_ROOM => {
				let top = get_rand(MIN_ROW, MIN_ROW + 5);
				let bottom = get_rand(DROWS - 7, DROWS - 2);
				let left = get_rand(0, 10);
				let right = get_rand(DCOLS - 11, DCOLS - 1);
				(left, right, top, bottom, false, 0)
			}
			_ => panic!("Invalid value in parameter rn")
		};
	let (left, right, top, bottom, fill_dungeon) = if do_shrink {
		let height = get_rand(4, bottom - top + 1);
		let width = get_rand(7, (right - left) - 2);
		let row_offset = get_rand(0, (bottom - top) - height + 1);
		let col_offset = get_rand(0, (right - left) - width + 1);
		let top = top + row_offset;
		let bottom = top + height - 1 as c_int;
		let left = left + col_offset;
		let right = left + width - 1;
		let skip_walls = (room_index != r1 as usize) && (room_index != r2 as usize) && (room_index != r3 as usize) && rand_percent(40 as c_int);
		(left, right, top, bottom, !skip_walls)
	} else {
		(left, right, top, bottom, true)
	};
	let (left, right, top, bottom) = (left as usize, right as usize, top as usize, bottom as usize);
	if fill_dungeon {
		rooms[room_index].is_room = R_ROOM;
		for i in top..(bottom + 1) {
			let top_border = i == top;
			let bottom_border = i == bottom;
			for j in left..(right + 1) {
				let left_border = j == left;
				let right_border = j == right;
				let ch = if top_border || bottom_border {
					HORWALL
				} else if !top_border && !bottom_border && (left_border || right_border) {
					VERTWALL
				} else {
					FLOOR
				};
				dungeon[i][j] = ch;
			}
		}
	}
	rooms[rn].top_row = top as libc::c_char;
	rooms[rn].bottom_row = bottom as libc::c_char;
	rooms[rn].left_col = left as libc::c_char;
	rooms[rn].right_col = right as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn clear_level() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 9 as libc::c_int {
		(*rooms.as_mut_ptr().offset(i as isize))
			.is_room = 0o1 as libc::c_int as libc::c_ushort;
		j = 0 as libc::c_int as libc::c_short;
		while (j as libc::c_int) < 4 as libc::c_int {
			(*rooms.as_mut_ptr().offset(i as isize))
				.doors[j as usize]
				.oth_room = -(1 as libc::c_int) as libc::c_short;
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 10 as libc::c_int {
		(*traps.as_mut_ptr().offset(i as isize))
			.trap_type = -(1 as libc::c_int) as libc::c_short;
		i += 1;
		i;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 24 as libc::c_int {
		j = 0 as libc::c_int as libc::c_short;
		while (j as libc::c_int) < 80 as libc::c_int {
			dungeon[i as usize][j as usize] = 0 as libc::c_int as libc::c_ushort;
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	see_invisible = 0 as libc::c_int as libc::c_char;
	detect_monster = see_invisible;
	bear_trap = 0 as libc::c_int as libc::c_short;
	being_held = bear_trap as libc::c_char;
	party_room = -(1 as libc::c_int) as libc::c_short;
	rogue.col = -(1 as libc::c_int) as libc::c_short;
	rogue.row = rogue.col;
	wclear(stdscr);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn mask_room(
	mut rn: libc::c_short,
	mut row: *mut libc::c_short,
	mut col: *mut libc::c_short,
	mut mask: libc::c_ushort,
) -> libc::c_char {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	i = (*rooms.as_mut_ptr().offset(rn as isize)).top_row as libc::c_short;
	while i as libc::c_int
		<= (*rooms.as_mut_ptr().offset(rn as isize)).bottom_row as libc::c_int
	{
		j = (*rooms.as_mut_ptr().offset(rn as isize)).left_col as libc::c_short;
		while j as libc::c_int
			<= (*rooms.as_mut_ptr().offset(rn as isize)).right_col as libc::c_int
		{
			if dungeon[i as usize][j as usize] as libc::c_int & mask as libc::c_int != 0
			{
				*row = i;
				*col = j;
				return 1 as libc::c_int as libc::c_char;
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	return 0 as libc::c_int as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn put_player(mut nr: libc::c_short) -> libc::c_int {
	let mut rn: libc::c_short = nr;
	let mut misses: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	misses = 0 as libc::c_int as libc::c_short;
	while (misses as libc::c_int) < 2 as libc::c_int
		&& rn as libc::c_int == nr as libc::c_int
	{
		gr_row_col(
			&mut row,
			&mut col,
			(0o100 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o200 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o1 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) as libc::c_ushort,
		);
		rn = get_room_number(row as libc::c_int, col as libc::c_int) as libc::c_short;
		misses += 1;
		misses;
	}
	rogue.row = row;
	rogue.col = col;
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o200 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		cur_room = -(3 as libc::c_int) as libc::c_short;
	} else {
		cur_room = rn;
	}
	if cur_room as libc::c_int != -(3 as libc::c_int) {
		light_up_room(cur_room as libc::c_int);
	} else {
		light_passage(rogue.row as libc::c_int, rogue.col as libc::c_int);
	}
	wake_room(
		get_room_number(rogue.row as libc::c_int, rogue.col as libc::c_int),
		1 as libc::c_int,
		rogue.row as libc::c_int,
		rogue.col as libc::c_int,
	);
	if !new_level_message.is_null() {
		message(new_level_message, 0 as libc::c_int);
		new_level_message = 0 as *mut libc::c_char;
	}
	if wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int)
		== -(1 as libc::c_int)
	{
		-(1 as libc::c_int);
	} else {
		waddch(stdscr, rogue.fchar as chtype);
	};
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn drop_check() -> libc::c_int {
	if wizard != 0 {
		return 1 as libc::c_int;
	}
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o4 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		if levitate != 0 {
			message(
				b"you're floating in the air!\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			return 0 as libc::c_int;
		}
		return 1 as libc::c_int;
	}
	message(
		b"I see no way down\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
	);
	return 0 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn check_up() -> libc::c_int {
	if wizard == 0 {
		if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
			& 0o4 as libc::c_int as libc::c_ushort as libc::c_int == 0
		{
			message(
				b"I see no way up\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			return 0 as libc::c_int;
		}
		if has_amulet() == 0 {
			message(
				b"Your way is magically blocked\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			return 0 as libc::c_int;
		}
	}
	new_level_message = b"you feel a wrenching sensation in your gut\0" as *const u8
		as *const libc::c_char as *mut libc::c_char;
	if cur_level as libc::c_int == 1 as libc::c_int {
		win();
	} else {
		cur_level = (cur_level as libc::c_int - 2 as libc::c_int) as libc::c_short;
		return 1 as libc::c_int;
	}
	return 0 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn add_exp(
	mut e: libc::c_int,
	mut promotion: libc::c_char,
) -> libc::c_int {
	let mut mbuf: [libc::c_char; 40] = [0; 40];
	let mut new_exp: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut hp: libc::c_short = 0;
	rogue.exp_points += e as libc::c_long;
	if rogue.exp_points
		>= level_points[(rogue.exp as libc::c_int - 1 as libc::c_int) as usize]
	{
		new_exp = get_exp_level(rogue.exp_points) as libc::c_short;
		if rogue.exp_points > 10000000 as libc::c_long {
			rogue
				.exp_points = 10000000 as libc::c_long
				+ 1 as libc::c_int as libc::c_long;
		}
		i = (rogue.exp as libc::c_int + 1 as libc::c_int) as libc::c_short;
		while i as libc::c_int <= new_exp as libc::c_int {
			sprintf(
				mbuf.as_mut_ptr(),
				b"welcome to level %d\0" as *const u8 as *const libc::c_char,
				i as libc::c_int,
			);
			message(mbuf.as_mut_ptr(), 0 as libc::c_int);
			if promotion != 0 {
				hp = hp_raise() as libc::c_short;
				rogue
					.hp_current = (rogue.hp_current as libc::c_int + hp as libc::c_int)
					as libc::c_short;
				rogue
					.hp_max = (rogue.hp_max as libc::c_int + hp as libc::c_int)
					as libc::c_short;
			}
			rogue.exp = i;
			print_stats(0o4 as libc::c_int | 0o40 as libc::c_int);
			i += 1;
			i;
		}
	} else {
		print_stats(0o40 as libc::c_int);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn show_average_hp() -> libc::c_int {
	let mut mbuf: [libc::c_char; 80] = [0; 80];
	let mut real_average: libc::c_int = 0;
	let mut effective_average: libc::c_int = 0;
	if rogue.exp as libc::c_int == 1 as libc::c_int {
		effective_average = 0.00f64 as libc::c_int;
		real_average = effective_average;
	} else {
		real_average = (rogue.hp_max as libc::c_int - extra_hp as libc::c_int
			- 12 as libc::c_int + less_hp as libc::c_int)
			/ (rogue.exp as libc::c_int - 1 as libc::c_int);
		effective_average = (rogue.hp_max as libc::c_int - 12 as libc::c_int)
			/ (rogue.exp as libc::c_int - 1 as libc::c_int);
	}
	sprintf(
		mbuf.as_mut_ptr(),
		b"R-Hp: %.2f, E-Hp: %.2f (!: %d, V: %d)\0" as *const u8 as *const libc::c_char,
		real_average,
		effective_average,
		extra_hp as libc::c_int,
		less_hp as libc::c_int,
	);
	message(mbuf.as_mut_ptr(), 0 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}
