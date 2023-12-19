#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	static mut level_monsters: object;
	fn object_at() -> *mut object;
	static mut cur_room: libc::c_short;
	static mut halluc: libc::c_short;
	static mut blind: libc::c_short;
	static mut levitate: libc::c_short;
	static mut cur_level: libc::c_short;
	static mut max_level: libc::c_short;
	static mut bear_trap: libc::c_short;
	static mut haste_self: libc::c_short;
	static mut confused: libc::c_short;
	static mut e_rings: libc::c_short;
	static mut regeneration: libc::c_short;
	static mut auto_search: libc::c_short;
	static mut hunger_str: [libc::c_char; 0];
	static mut being_held: libc::c_char;
	static mut r_teleport: libc::c_char;
}

use libc::{strcpy, strlen};
use ncurses::addch;
use crate::prelude::*;
use crate::prelude::SpotFlag::{Door, Nothing};


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
pub static mut m_moves: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut you_can_move_again: *mut libc::c_char = b"you can move again\0"
	as *const u8 as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn one_move_rogue(mut dirch: libc::c_short, pickup: bool, settings: &Settings) -> libc::c_int {
	let mut current_block: u64;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut n: libc::c_short = 0;
	let mut status: libc::c_short = 0;
	row = rogue.row;
	col = rogue.col;
	if confused != 0 {
		dirch = gr_dir() as libc::c_short;
	}
	get_dir_rc(dirch, &mut row, &mut col, true);
	if can_move(rogue.row as usize, rogue.col as usize, row as usize, col as usize) == false {
		return -(1 as libc::c_int);
	}
	if being_held as libc::c_int != 0 || bear_trap as libc::c_int != 0 {
		if dungeon[row as usize][col as usize] as libc::c_int
			& 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
		{
			if being_held != 0 {
				message(
					b"you are being held\0" as *const u8 as *const libc::c_char,
					1 as libc::c_int,
				);
			} else {
				message(
					b"you are still stuck in the bear trap\0" as *const u8
						as *const libc::c_char,
					0 as libc::c_int,
				);
				reg_move();
			}
			return -(1 as libc::c_int);
		}
	}
	if r_teleport != 0 {
		if rand_percent(8 as libc::c_int) != 0 {
			tele();
			return -(2 as libc::c_int);
		}
	}
	if dungeon[row as usize][col as usize] as libc::c_int
		& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		rogue_hit(
			object_at(&mut level_monsters, row as libc::c_int, col as libc::c_int),
			0 as libc::c_int,
		);
		reg_move();
		return -(1 as libc::c_int);
	}
	if dungeon[row as usize][col as usize] as libc::c_int
		& 0o40 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		if cur_room as libc::c_int == -(3 as libc::c_int) {
			cur_room = get_room_number(row as libc::c_int, col as libc::c_int)
				as libc::c_short;
			light_up_room(cur_room as libc::c_int);
			wake_room(
				cur_room as libc::c_int,
				1 as libc::c_int,
				row as libc::c_int,
				col as libc::c_int,
			);
		} else {
			light_passage(row as libc::c_int, col as libc::c_int);
		}
	} else if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o40 as libc::c_int as libc::c_ushort as libc::c_int != 0
		&& dungeon[row as usize][col as usize] as libc::c_int
		& 0o200 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		light_passage(row as libc::c_int, col as libc::c_int);
		wake_room(
			cur_room as libc::c_int,
			0 as libc::c_int,
			rogue.row as libc::c_int,
			rogue.col as libc::c_int,
		);
		darken_room(cur_room as libc::c_int);
		cur_room = -(3 as libc::c_int) as libc::c_short;
	} else if dungeon[row as usize][col as usize] as libc::c_int
		& 0o200 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		light_passage(row as libc::c_int, col as libc::c_int);
	}
	if wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int)
		== -(1 as libc::c_int)
	{
		-(1 as libc::c_int);
	} else {
		addch(get_dungeon_char(rogue.row as usize, rogue.col as usize) as chtype);
	};
	if wmove(stdscr, row as libc::c_int, col as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		addch(rogue.fchar as chtype);
	};
	if !settings.jump {
		ncurses::refresh();
	}
	rogue.row = row;
	rogue.col = col;
	if dungeon[row as usize][col as usize] as libc::c_int
		& 0o1 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		if levitate as libc::c_int != 0 && pickup as libc::c_int != 0 {
			return -(2 as libc::c_int);
		}
		if pickup as libc::c_int != 0 && levitate == 0 {
			obj = pick_up(row as libc::c_int, col as libc::c_int, &mut status);
			if !obj.is_null() {
				get_desc(obj, desc.as_mut_ptr());
				if (*obj).what_is as libc::c_int
					== 0o20 as libc::c_int as libc::c_ushort as libc::c_int
				{
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
							row as libc::c_int,
							col as libc::c_int,
						);
						strcpy(
							desc.as_mut_ptr(),
							b"moved onto \0" as *const u8 as *const libc::c_char,
						);
						get_desc(
							obj,
							desc.as_mut_ptr().offset(11 as libc::c_int as isize),
						);
					}
					_ => {}
				}
				message(desc.as_mut_ptr(), 1 as libc::c_int);
				reg_move();
				return -(2 as libc::c_int);
			}
		}
	} else if dungeon[row as usize][col as usize] as libc::c_int
		& (0o40 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o400 as libc::c_int as libc::c_ushort as libc::c_int) != 0
	{
		if levitate == 0
			&& dungeon[row as usize][col as usize] as libc::c_int
			& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
		{
			trap_player(row as libc::c_int, col as libc::c_int);
		}
		reg_move();
		return -(2 as libc::c_int);
	}
	if reg_move() != 0 {
		return -(2 as libc::c_int);
	}
	return if confused as libc::c_int != 0 {
		-(2 as libc::c_int)
	} else {
		0 as libc::c_int
	};
}

#[no_mangle]
pub unsafe extern "C" fn multiple_move_rogue(mut dirch: libc::c_int) -> libc::c_int {
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut m: libc::c_short = 0;
	match dirch {
		8 | 10 | 11 | 12 | 25 | 21 | 14 | 2 => {
			loop {
				row = rogue.row;
				col = rogue.col;
				m = one_move_rogue(dirch + 96 as libc::c_int, 1 as libc::c_int)
					as libc::c_short;
				if m as libc::c_int == -(1 as libc::c_int)
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
pub unsafe extern "C" fn is_direction(mut c: libc::c_int) -> libc::c_char {
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
			if rand_percent(40 as libc::c_int) != 0 {
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
