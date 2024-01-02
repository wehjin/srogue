#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::addch;
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::prelude::SpotFlag::{Floor, Monster};


#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct tr {
	pub trap_type: i16,
	pub trap_row: i16,
	pub trap_col: i16,
}

pub type trap = tr;

#[no_mangle]
pub static mut traps: [trap; 10] = [tr {
	trap_type: 0,
	trap_row: 0,
	trap_col: 0,
}; 10];
pub static mut trap_door: libc::c_char = 0 as libc::c_char;
pub static mut bear_trap: usize = 0;
#[no_mangle]
pub static mut trap_strings: [*mut libc::c_char; 12] = [
	b"trap door\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"you fell down a trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"bear trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"you are caught in a bear trap\0" as *const u8 as *const libc::c_char
		as *mut libc::c_char,
	b"teleport trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"teleport\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"poison dart trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"a small dart just hit you in the shoulder\0" as *const u8 as *const libc::c_char
		as *mut libc::c_char,
	b"sleeping gas trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"a strange white mist envelops you and you fall asleep\0" as *const u8
		as *const libc::c_char as *mut libc::c_char,
	b"rust trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"a gush of water hits you on the head\0" as *const u8 as *const libc::c_char
		as *mut libc::c_char,
];

#[no_mangle]
pub unsafe extern "C" fn trap_at(
	mut row: i64,
	mut col: i64,
) -> i64 {
	let mut i: libc::c_short = 0;
	i = 0;
	while (i as i64) < 10 as i64
		&& traps[i as usize].trap_type as i64 != -(1)
	{
		if traps[i as usize].trap_row as i64 == row
			&& traps[i as usize].trap_col as i64 == col
		{
			return traps[i as usize].trap_type as i64;
		}
		i += 1;
		i;
	}
	return -(1);
}

#[no_mangle]
pub unsafe extern "C" fn trap_player(mut row: i64, mut col: i64) -> i64 {
	let mut t: libc::c_short = 0;
	t = trap_at(row as i64, col as i64) as libc::c_short;
	if t as i64 == -(1) {
		return;
	}
	dungeon[row
		as usize][col
		as usize] = (dungeon[row as usize][col as usize] as i64
		& !(0o1000 as i64 as libc::c_ushort as i64)) as libc::c_ushort;
	if rand_percent((rogue.exp + ring_exp) as usize) != 0 {
		message(
			b"the trap failed\0" as *const u8 as *const libc::c_char,
			1,
		);
		return;
	}
	match t as i64 {
		0 => {
			trap_door = 1 as libc::c_char;
			new_level_message = trap_strings[(t as i64 * 2 as i64
				+ 1) as usize];
		}
		1 => {
			message(
				trap_strings[(t as i64 * 2 as i64 + 1)
					as usize],
				1,
			);
			bear_trap = get_rand(4 as i64, 7 as i64) as libc::c_short;
		}
		2 => {
			if ncurses::wmove(ncurses::stdscr(), rogue.row as i64, rogue.col as i64)
				== -(1)
			{
				-(1);
			} else {
				addch('^' as i32 as ncurses::chtype);
			};
			tele();
		}
		3 => {
			message(
				trap_strings[(t as i64 * 2 as i64 + 1)
					as usize],
				1,
			);
			rogue
				.hp_current = (rogue.hp_current as i64
				- get_damage(
				b"1d6\0" as *const u8 as *const libc::c_char,
				1,
			)) as libc::c_short;
			if rogue.hp_current as i64 <= 0 as i64 {
				rogue.hp_current = 0;
			}
			if sustain_strength == 0 && rand_percent(40) != 0
				&& rogue.str_current as i64 >= 3 as i64
			{
				rogue.str_current -= 1;
				rogue.str_current;
			}
			print_stats(0o4 as i64 | 0o10 as i64);
			if rogue.hp_current as i64 <= 0 as i64 {
				killed_by(0 as *mut object, 3);
			}
		}
		4 => {
			message(
				trap_strings[(t as i64 * 2 as libc::c_int + 1 as libc::c_int)
					as usize],
				1 as libc::c_int,
			);
			take_a_nap();
		}
		5 => {
			message(
				trap_strings[(t as libc::c_int * 2 as libc::c_int + 1 as libc::c_int)
					as usize],
				1 as libc::c_int,
			);
			rust(0 as *mut object);
		}
		_ => {}
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn add_traps() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut tries: libc::c_short = 0 as libc::c_int as libc::c_short;
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	if cur_level as libc::c_int <= 2 as libc::c_int {
		n = 0 as libc::c_int as libc::c_short;
	} else if cur_level as libc::c_int <= 7 as libc::c_int {
		n = get_rand(0 as libc::c_int, 2 as libc::c_int) as libc::c_short;
	} else if cur_level as libc::c_int <= 11 as libc::c_int {
		n = get_rand(1 as libc::c_int, 2 as libc::c_int) as libc::c_short;
	} else if cur_level as libc::c_int <= 16 as libc::c_int {
		n = get_rand(2 as libc::c_int, 3 as libc::c_int) as libc::c_short;
	} else if cur_level as libc::c_int <= 21 as libc::c_int {
		n = get_rand(2 as libc::c_int, 4 as libc::c_int) as libc::c_short;
	} else if cur_level as libc::c_int <= 26 as libc::c_int + 2 as libc::c_int {
		n = get_rand(3 as libc::c_int, 5 as libc::c_int) as libc::c_short;
	} else {
		n = get_rand(5 as libc::c_int, 10 as libc::c_int) as libc::c_short;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < n as libc::c_int {
		traps[i as usize]
			.trap_type = get_rand(0 as libc::c_int, 6 as libc::c_int - 1 as libc::c_int)
			as libc::c_short;
		if i as libc::c_int == 0 as libc::c_int
			&& party_room as libc::c_int != -(1 as libc::c_int)
		{
			loop {
				row = get_rand(
					(*rooms.as_mut_ptr().offset(party_room as isize)).top_row + 1,
					(*rooms.as_mut_ptr().offset(party_room as isize)).bottom_row - 1,
				);
				col = get_rand(
					(*rooms.as_mut_ptr().offset(party_room as isize)).left_col + 1,
					(*rooms.as_mut_ptr().offset(party_room as isize)).right_col - 1,
				);
				tries += 1;
				tries;
				if !((dungeon[row as usize][col as usize] as libc::c_int
					& (0o1 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o400 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o200 as libc::c_int as libc::c_ushort as libc::c_int) != 0
					|| dungeon[row as usize][col as usize] as libc::c_int
					== 0 as libc::c_int as libc::c_ushort as libc::c_int)
					&& (tries as libc::c_int) < 15 as libc::c_int)
				{
					break;
				}
			}
			if tries as libc::c_int >= 15 as libc::c_int {
				gr_row_col(&mut row, &mut col, vec![Floor, Monster]);
			}
		} else {
			gr_row_col(&mut row, &mut col, vec![Floor, Monster],
			);
		}
		traps[i as usize].trap_row = row;
		traps[i as usize].trap_col = col;
		dungeon[row
			as usize][col
			as usize] = (dungeon[row as usize][col as usize] as libc::c_int
			| (0o400 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o1000 as libc::c_int as libc::c_ushort as libc::c_int))
			as libc::c_ushort;
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn id_trap() -> libc::c_int {
	let mut dir: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut t: libc::c_short = 0;
	message(b"direction? \0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
	loop {
		dir = rgetchar() as libc::c_short;
		if !(is_direction(dir as libc::c_int) == 0) {
			break;
		}
		sound_bell();
	}
	check_message();
	if dir as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	row = rogue.row;
	col = rogue.col;
	get_dir_rc(dir as libc::c_int, &mut row, &mut col, 0 as libc::c_int);
	if dungeon[row as usize][col as usize] as libc::c_int
		& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
		&& dungeon[row as usize][col as usize] as libc::c_int
		& 0o1000 as libc::c_int as libc::c_ushort as libc::c_int == 0
	{
		t = trap_at(row as libc::c_int, col as libc::c_int) as libc::c_short;
		message(
			trap_strings[(t as libc::c_int * 2 as libc::c_int) as usize],
			0 as libc::c_int,
		);
	} else {
		message(
			b"no trap there\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn show_traps() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 24 as libc::c_int {
		j = 0 as libc::c_int as libc::c_short;
		while (j as libc::c_int) < 80 as libc::c_int {
			if dungeon[i as usize][j as usize] as libc::c_int
				& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
			{
				if ncurses::wmove(ncurses::stdscr(), i as libc::c_int, j as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					addch('^' as i32 as ncurses::chtype);
				};
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn search(
	mut n: libc::c_short,
	is_auto: bool,
) -> libc::c_int {
	let mut s: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut t: libc::c_short = 0;
	let mut shown: libc::c_short = 0 as libc::c_int as libc::c_short;
	let mut found: libc::c_short = 0 as libc::c_int as libc::c_short;
	static mut reg_search: libc::c_char = 0;
	i = -(1 as libc::c_int) as libc::c_short;
	while i as libc::c_int <= 1 as libc::c_int {
		j = -(1 as libc::c_int) as libc::c_short;
		while j as libc::c_int <= 1 as libc::c_int {
			row = (rogue.row as libc::c_int + i as libc::c_int) as libc::c_short;
			col = (rogue.col as libc::c_int + j as libc::c_int) as libc::c_short;
			if !((row as libc::c_int) < 1 as libc::c_int
				|| row as libc::c_int >= 24 as libc::c_int - 1 as libc::c_int
				|| (col as libc::c_int) < 0 as libc::c_int
				|| col as libc::c_int >= 80 as libc::c_int)
			{
				if dungeon[row as usize][col as usize] as libc::c_int
					& 0o1000 as libc::c_int as libc::c_ushort as libc::c_int != 0
				{
					found += 1;
					found;
				}
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	s = 0 as libc::c_int as libc::c_short;
	while (s as libc::c_int) < n as libc::c_int {
		i = -(1 as libc::c_int) as libc::c_short;
		while i as libc::c_int <= 1 as libc::c_int {
			j = -(1 as libc::c_int) as libc::c_short;
			while j as libc::c_int <= 1 as libc::c_int {
				row = (rogue.row as libc::c_int + i as libc::c_int) as libc::c_short;
				col = (rogue.col as libc::c_int + j as libc::c_int) as libc::c_short;
				if !((row as libc::c_int) < 1 as libc::c_int
					|| row as libc::c_int >= 24 as libc::c_int - 1 as libc::c_int
					|| (col as libc::c_int) < 0 as libc::c_int
					|| col as libc::c_int >= 80 as libc::c_int)
				{
					if dungeon[row as usize][col as usize] as libc::c_int
						& 0o1000 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						if rand_percent(
							17
								+ (rogue.exp + ring_exp) as usize,
						) != 0
						{
							dungeon[row
								as usize][col
								as usize] = (dungeon[row as usize][col as usize]
								as libc::c_int
								& !(0o1000 as libc::c_int as libc::c_ushort as libc::c_int))
								as libc::c_ushort;
							if blind == 0
								&& (row as libc::c_int != rogue.row as libc::c_int
								|| col as libc::c_int != rogue.col as libc::c_int)
							{
								if ncurses::wmove(ncurses::stdscr(), row as libc::c_int, col as libc::c_int)
									== -(1 as libc::c_int)
								{
									-(1 as libc::c_int);
								} else {
									addch(get_dungeon_char(row as usize, col as usize) as ncurses::chtype);
								};
							}
							shown += 1;
							shown;
							if dungeon[row as usize][col as usize] as libc::c_int
								& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
							{
								t = trap_at(row as libc::c_int, col as libc::c_int)
									as libc::c_short;
								message(
									trap_strings[(t as libc::c_int * 2 as libc::c_int)
										as usize],
									1 as libc::c_int,
								);
							}
						}
					}
					if shown as libc::c_int == found as libc::c_int
						&& found as libc::c_int > 0 as libc::c_int
						|| interrupted as libc::c_int != 0
					{
						return;
					}
				}
				j += 1;
				j;
			}
			i += 1;
			i;
		}
		if is_auto == 0
			&& {
			reg_search = (reg_search == 0) as libc::c_int as libc::c_char;
			reg_search as libc::c_int != 0
		}
		{
			reg_move();
		}
		s += 1;
		s;
	}
	panic!("Reached end of non-void function without returning");
}
