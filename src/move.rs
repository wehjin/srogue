#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn object_at() -> *mut object;
	static mut regeneration: libc::c_short;
	static mut auto_search: libc::c_short;
	static mut r_teleport: libc::c_char;
}

use libc::{strcpy, strlen};
use ncurses::{addch};
use crate::prelude::*;
use crate::prelude::SpotFlag::{Door, Nothing};
use crate::settings::jump;


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


pub static mut m_moves: i16 = 0;
#[no_mangle]
pub static mut you_can_move_again: *mut libc::c_char = b"you can move again\0"
	as *const u8 as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn one_move_rogue(mut dirch: libc::c_short, pickup: bool) -> i64 {
	let mut current_block: u64;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut n: libc::c_short = 0;
	let mut status: libc::c_short = 0;
	row = rogue.row;
	col = rogue.col;
	if confused {
		dirch = gr_dir() as libc::c_short;
	}
	get_dir_rc(dirch, &mut row, &mut col, true);
	if can_move(rogue.row as usize, rogue.col as usize, row as usize, col as usize) == false {
		return -(1);
	}
	if being_held as i64 != 0 || bear_trap as i64 != 0 {
		if dungeon[row as usize][col as usize] as i64
			& 0o2 as i64 as libc::c_ushort as i64 == 0
		{
			if being_held != 0 {
				message(
					b"you are being held\0" as *const u8 as *const libc::c_char,
					1,
				);
			} else {
				message(
					b"you are still stuck in the bear trap\0" as *const u8
						as *const libc::c_char,
					0 as i64,
				);
				reg_move();
			}
			return -(1);
		}
	}
	if r_teleport != 0 {
		if rand_percent(8) != 0 {
			tele();
			return -(2 as i64);
		}
	}
	if dungeon[row as usize][col as usize] as i64
		& 0o2 as i64 as libc::c_ushort as i64 != 0
	{
		rogue_hit(
			object_at(&mut level_monsters, row as i64, col as i64),
			0 as i64,
		);
		reg_move();
		return -(1);
	}
	if dungeon[row as usize][col as usize] as i64
		& 0o40 as i64 as libc::c_ushort as i64 != 0
	{
		if cur_room as i64 == -(3 as i64) {
			cur_room = get_room_number(row as i64, col as i64)
				as libc::c_short;
			light_up_room(cur_room as i64);
			wake_room(
				cur_room as i64,
				1,
				row as i64,
				col as i64,
			);
		} else {
			light_passage(row as i64, col as i64);
		}
	} else if dungeon[rogue.row as usize][rogue.col as usize] as i64
		& 0o40 as i64 as libc::c_ushort as i64 != 0
		&& dungeon[row as usize][col as usize] as i64
		& 0o200 as i64 as libc::c_ushort as i64 != 0
	{
		light_passage(row as i64, col as i64);
		wake_room(
			cur_room as i64,
			0 as i64,
			rogue.row as i64,
			rogue.col as i64,
		);
		darken_room(cur_room as i64);
		cur_room = -(3 as i64) as libc::c_short;
	} else if dungeon[row as usize][col as usize] as i64
		& 0o200 as i64 as libc::c_ushort as i64 != 0
	{
		light_passage(row as i64, col as i64);
	}
	if ncurses::wmove(ncurses::stdscr(), rogue.row as i64, rogue.col as i64)
		== -(1)
	{
		-(1);
	} else {
		addch(get_dungeon_char(rogue.row as usize, rogue.col as usize) as ncurses::chtype);
	};
	if ncurses::wmove(ncurses::stdscr(), row as i64, col as i64) == -(1) {
		-(1);
	} else {
		addch(rogue.fchar as ncurses::chtype);
	};
	if !jump() {
		ncurses::refresh();
	}
	rogue.row = row;
	rogue.col = col;
	if dungeon[row as usize][col as usize] as i64
		& 0o1 as libc::c_ushort as i64 != 0
	{
		if levitate as i64 != 0 && pickup as i64 != 0 {
			return -(2 as i64);
		}
		if pickup as i64 != 0 && levitate == 0 {
			obj = pick_up(row as i64, col as i64, &mut status);
			if !obj.is_null() {
				let desc = get_desc(&obj);
				if (*obj).what_is as i64
					== 0o20 as i64 as libc::c_ushort as i64
				{
					free_object(obj);
				} else {
					n = strlen(desc.as_mut_ptr()) as libc::c_short;
					desc[n as usize] = '(' as i32 as libc::c_char;
					desc[(n as i64 + 1)
						as usize] = (*obj).ichar as libc::c_char;
					desc[(n as i64 + 2 as i64)
						as usize] = ')' as i32 as libc::c_char;
					desc[(n as i64 + 3 as i64)
						as usize] = 0 as i64 as libc::c_char;
				}
				current_block = 18080986910393262295;
			} else if status == 0 {
				current_block = 5297757197365674617;
			} else {
				current_block = 11696287569021009278;
			}
		} else {
			current_block = 11696287569021009278;
		}
		match current_block {
			5297757197365674617 => {}
			_ => {
				match current_block {
					11696287569021009278 => {
						obj = object_at(
							&mut level_objects,
							row as i64,
							col as i64,
						);
						strcpy(
							desc.as_mut_ptr(),
							b"moved onto \0" as *const u8 as *const libc::c_char,
						);
						get_desc(
							obj,
							desc.as_mut_ptr().offset(11 as isize),
						);
					}
					_ => {}
				}
				message(desc.as_mut_ptr(), 1);
				reg_move();
				return -(2 as i64);
			}
		}
	} else if dungeon[row as usize][col as usize] as i64
		& (0o40 as i64 as libc::c_ushort as i64
		| 0o4 as i64 as libc::c_ushort as i64
		| 0o400 as i64 as libc::c_ushort as i64) != 0
	{
		if levitate == 0
			&& dungeon[row as usize][col as usize] as i64
			& 0o400 as i64 as libc::c_ushort as i64 != 0
		{
			trap_player(row as i64, col as i64);
		}
		reg_move();
		return -(2 as i64);
	}
	if reg_move() != 0 {
		return -(2 as i64);
	}
	return if confused as i64 != 0 {
		-(2 as i64)
	} else {
		0 as i64
	};
}

#[no_mangle]
pub unsafe extern "C" fn multiple_move_rogue(mut dirch: i64) -> i64 {
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut m: libc::c_short = 0;
	match dirch {
		8 | 10 | 11 | 12 | 25 | 21 | 14 | 2 => {
			loop {
				row = rogue.row;
				col = rogue.col;
				m = one_move_rogue(dirch + 96 as i64, 1)
					as libc::c_short;
				if m as i64 == -(1)
					|| m as libc::c_int == -(2 as libc::c_int)
					|| interrupted as libc::c_int != 0
				{
					break;
				}
				if !(next_to_something(row as libc::c_int, col as libc::c_int) == 0) {
					break;
				}
			}
		}
		72 | 74 | 75 | 76 | 66 | 89 | 85 | 78 => {
			while interrupted == 0
				&& one_move_rogue(dirch + 32 as libc::c_int, 1 as libc::c_int)
				== 0 as libc::c_int
			{}
		}
		_ => {}
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn is_passable(
	mut row: libc::c_int,
	mut col: libc::c_int,
) -> bool {
	if row < 1 as libc::c_int || row > 24 as libc::c_int - 2 as libc::c_int
		|| col < 0 as libc::c_int || col > 80 as libc::c_int - 1 as libc::c_int
	{
		return false;
	}
	if dungeon[row as usize][col as usize] as libc::c_int
		& 0o1000 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		return if dungeon[row as usize][col as usize] as libc::c_int
			& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
		{
			true
		} else {
			false
		};
	}
	const flags: Vec<SpotFlag> = vec![SpotFlag::Floor, SpotFlag::Tunnel, SpotFlag::Door, SpotFlag::Stairs, SpotFlag::Trap];
	return SpotFlag::is_any_set(&flags, dungeon[row as usize][col as usize]);
}

pub unsafe fn can_move(row1: usize, col1: usize, row2: usize, col2: usize) -> bool {
	if is_passable(row2 as libc::c_int, col2 as libc::c_int) {
		if row1 != row2 && col1 != col2 {
			if Door.is_set(dungeon[row1][col1]) || Door::is_set(dungeon[row2][col2]) {
				return false;
			}
			if Nothing.is_set(dungeon[row1][col2]) || Nothing.is_set(dungeon[row2][col1]) {
				return false;
			}
		}
		return true;
	} else {
		return false;
	}
}

#[no_mangle]
pub unsafe extern "C" fn move_onto() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut first_miss: libc::c_char = 1 as libc::c_int as libc::c_char;
	loop {
		ch = rgetchar() as libc::c_short;
		if !(is_direction(ch as libc::c_int) == 0) {
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
	if ch as libc::c_int != '\u{1b}' as i32 {
		one_move_rogue(ch as libc::c_int, 0 as libc::c_int);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn is_direction(mut c: i32) -> libc::c_char {
	return (c == 'h' as i32 || c == 'j' as i32 || c == 'k' as i32 || c == 'l' as i32
		|| c == 'b' as i32 || c == 'y' as i32 || c == 'u' as i32 || c == 'n' as i32
		|| c == '\u{1b}' as i32) as libc::c_int as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn check_hunger(mut messages_only: libc::c_char) -> libc::c_char {
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut fainted: libc::c_char = 0 as libc::c_int as libc::c_char;
	if rogue.moves_left as libc::c_int == 300 as libc::c_int {
		strcpy(hunger_str.as_mut_ptr(), b"hungry\0" as *const u8 as *const libc::c_char);
		message(hunger_str.as_mut_ptr(), 0 as libc::c_int);
		print_stats(0o100 as libc::c_int);
	}
	if rogue.moves_left as libc::c_int == 150 as libc::c_int {
		strcpy(hunger_str.as_mut_ptr(), b"weak\0" as *const u8 as *const libc::c_char);
		message(hunger_str.as_mut_ptr(), 1 as libc::c_int);
		print_stats(0o100 as libc::c_int);
	}
	if rogue.moves_left as libc::c_int <= 20 as libc::c_int {
		if rogue.moves_left as libc::c_int == 20 as libc::c_int {
			strcpy(
				hunger_str.as_mut_ptr(),
				b"faint\0" as *const u8 as *const libc::c_char,
			);
			message(hunger_str.as_mut_ptr(), 1 as libc::c_int);
			print_stats(0o100 as libc::c_int);
		}
		n = get_rand(
			0 as libc::c_int,
			20 as libc::c_int - rogue.moves_left as libc::c_int,
		) as libc::c_short;
		if n as libc::c_int > 0 as libc::c_int {
			fainted = 1 as libc::c_int as libc::c_char;
			if rand_percent(40) != 0 {
				rogue.moves_left += 1;
				rogue.moves_left;
			}
			message(
				b"you faint\0" as *const u8 as *const libc::c_char,
				1 as libc::c_int,
			);
			i = 0 as libc::c_int as libc::c_short;
			while (i as libc::c_int) < n as libc::c_int {
				if coin_toss() != 0 {
					mv_mons();
				}
				i += 1;
				i;
			}
			message(you_can_move_again, 1 as libc::c_int);
		}
	}
	if messages_only != 0 {
		return fainted;
	}
	if rogue.moves_left as libc::c_int <= 0 as libc::c_int {
		killed_by(0 as *mut object, 2 as libc::c_int);
	}
	match e_rings as libc::c_int {
		-1 => {
			rogue
				.moves_left = (rogue.moves_left as libc::c_int
				- rogue.moves_left as libc::c_int % 2 as libc::c_int) as libc::c_short;
		}
		0 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
		}
		1 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
			check_hunger(1 as libc::c_int);
			rogue
				.moves_left = (rogue.moves_left as libc::c_int
				- rogue.moves_left as libc::c_int % 2 as libc::c_int) as libc::c_short;
		}
		2 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
			check_hunger(1 as libc::c_int);
			rogue.moves_left -= 1;
			rogue.moves_left;
		}
		_ => {}
	}
	return fainted;
}

#[no_mangle]
pub unsafe extern "C" fn reg_move() -> libc::c_char {
	let mut fainted: libc::c_char = 0;
	if rogue.moves_left as libc::c_int <= 300 as libc::c_int
		|| cur_level as libc::c_int >= max_level as libc::c_int
	{
		fainted = check_hunger(0 as libc::c_int);
	} else {
		fainted = 0 as libc::c_int as libc::c_char;
	}
	mv_mons();
	m_moves += 1;
	if m_moves as libc::c_int >= 120 as libc::c_int {
		m_moves = 0 as libc::c_int as libc::c_short;
		wanderer();
	}
	if halluc != 0 {
		halluc -= 1;
		if halluc == 0 {
			unhallucinate();
		} else {
			hallucinate();
		}
	}
	if blind != 0 {
		blind -= 1;
		if blind == 0 {
			unblind();
		}
	}
	if confused != 0 {
		confused -= 1;
		if confused == 0 {
			unconfuse();
		}
	}
	if bear_trap != 0 {
		bear_trap -= 1;
		bear_trap;
	}
	if levitate != 0 {
		levitate -= 1;
		if levitate == 0 {
			message(
				b"you float gently to the ground\0" as *const u8 as *const libc::c_char,
				1 as libc::c_int,
			);
			if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
				& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
			{
				trap_player(rogue.row as libc::c_int, rogue.col as libc::c_int);
			}
		}
	}
	if haste_self != 0 {
		haste_self -= 1;
		if haste_self == 0 {
			message(
				b"you feel yourself slowing down\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
		}
	}
	heal();
	if auto_search as libc::c_int > 0 as libc::c_int {
		search(auto_search as libc::c_int, auto_search as libc::c_int);
	}
	return fainted;
}

#[no_mangle]
pub unsafe extern "C" fn rest(mut count: libc::c_int) -> libc::c_int {
	let mut i: libc::c_int = 0;
	interrupted = 0 as libc::c_int as libc::c_char;
	i = 0 as libc::c_int;
	while i < count {
		if interrupted != 0 {
			break;
		}
		reg_move();
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}
