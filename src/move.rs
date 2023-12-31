#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	static mut regeneration: libc::c_short;
	static mut auto_search: libc::c_short;
}

use libc::{c_short, strcpy, strlen};
use ncurses::{addch, chtype, mvaddch, refresh};
use MoveResult::MoveFailed;
use crate::odds::R_TELE_PERCENT;
use crate::prelude::*;
use crate::prelude::object_what::ObjectWhat::Gold;
use crate::prelude::SpotFlag::{Door, Hidden, Monster, Nothing, Object, Stairs, Trap, Tunnel};
use crate::prelude::stat_const::STAT_HUNGER;
use crate::r#move::MoveResult::{Moved, StoppedOnSomething};
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
pub static you_can_move_again: &'static str = "you can move again";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}


pub unsafe fn one_move_rogue(dirch: char, pickup: bool) -> MoveResult {
	let dirch = if confused { Move::random8().to_char() } else { dirch };
	let mut row = rogue.row;
	let mut col = rogue.col;
	get_dir_rc(dirch, &mut row, &mut col, true);
	if !can_move(rogue.row as i64, rogue.col as i64, row, col) {
		return MoveFailed;
	}
	if being_held || bear_trap {
		if !Monster.is_set(dungeon[row as usize][col as usize]) {
			if being_held {
				message("you are being held", 1);
			} else {
				message("you are still stuck in the bear trap", 0);
				reg_move();
			}
			return MoveFailed;
		}
	}
	if r_teleport {
		if rand_percent(R_TELE_PERCENT) {
			tele();
			return StoppedOnSomething;
		}
	}
	if Monster.is_set(dungeon[row as usize][col as usize]) {
		rogue_hit(object_at(&mut level_monsters, row, col), false);
		reg_move();
		return MoveFailed;
	}
	if Door.is_set(dungeon[row as usize][col as usize]) {
		if cur_room == PASSAGE {
			cur_room = get_room_number(row as i64, col as i64);
			light_up_room(cur_room);
			wake_room(cur_room, true, row, col);
		} else {
			light_passage(row, col);
		}
	} else if Door.is_set(dungeon[rogue.row as usize][rogue.col as usize]) && Tunnel.is_set(dungeon[row as usize][col as usize]) {
		light_passage(row as i64, col as i64);
		wake_room(cur_room, false, rogue.row as i64, rogue.col as i64);
		darken_room(cur_room);
		cur_room = PASSAGE;
	} else if Tunnel.is_set(dungeon[row as usize][col as usize]) {
		light_passage(row, col);
	}
	mvaddch(rogue.row as i32, rogue.col as i32, get_dungeon_char(rogue.row as i64, rogue.col as i64));
	mvaddch(row as i32, col as i32, chtype::from(rogue.fchar));
	if !jump() {
		refresh();
	}
	rogue.row = row;
	rogue.col = col;
	if Object.is_set(dungeon[row as usize][col as usize]) {
		if levitate && pickup {
			return StoppedOnSomething;
		}
		if pickup && !levitate {
			let mut status = 0;
			let obj = pick_up(row, col, &mut status);
			if !obj.is_null() {
				if (*obj).what_is.what_is() == Gold {
					free_object(obj);
					return not_in_pack(&get_desc(&*obj));
				} else {
					let desc = format!("{}({})", get_desc(&*obj), (*obj).ichar);
					return not_in_pack(&desc);
				}
			} else if status != 0 {
				return mved();
			} else {
				return move_on(row, col);
			}
		} else {
			return move_on(row, col);
		}
	} else if SpotFlag::is_any_set(&vec![Door, Stairs, Trap], dungeon[row as usize][col as usize]) {
		if !levitate && Trap.is_set(dungeon[row as usize][col as usize]) {
			trap_player(row, col);
		}
		reg_move();
		return StoppedOnSomething;
	} else {
		return mved();
	}
}

unsafe fn move_on(row: i64, col: i64) -> MoveResult {
	let obj = object_at(&mut level_objects, row, col);
	let desc = format!("moved onto {}", get_desc(&*obj));
	return not_in_pack(&desc);
}

unsafe fn not_in_pack(desc: &str) -> MoveResult {
	message(desc, 1);
	reg_move();
	return StoppedOnSomething;
}

unsafe fn mved() -> MoveResult {
	if reg_move() != 0 {
		/* fainted from hunger */
		return StoppedOnSomething;
	} else {
		return if confused { StoppedOnSomething } else { Moved };
	}
}

#[no_mangle]
pub unsafe extern "C" fn multiple_move_rogue(mut dirch: i64) -> i64 {
	let mut row = 0;
	let mut col = 0;
	let mut m: libc::c_short = 0;
	match dirch {
		8 | 10 | 11 | 12 | 25 | 21 | 14 | 2 => {
			loop {
				row = rogue.row;
				col = rogue.col;
				m = one_move_rogue((dirch as u8 + 96) as char, true)
					as libc::c_short;
				if m as i64 == -(1)
					|| m as libc::c_int == -(2 as libc::c_int)
					|| interrupted as libc::c_int != 0
				{
					break;
				}
				if next_to_something(row, col) {
					break;
				}
			}
		}
		72 | 74 | 75 | 76 | 66 | 89 | 85 | 78 => {
			while !interrupted && one_move_rogue((dirch as u8 + 32) as char, true) == Moved {}
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

pub unsafe fn next_to_something(drow: i64, dcol: i64) -> bool {
	if confused {
		return true;
	}
	if blind {
		return false;
	}
	let mut row = 0;
	let mut col = 0;
	let mut pass_count = 0;
	let i_end = if (rogue.row < (DROWS as i64 - 2)) { 1 } else { 0 };
	let j_end = if (rogue.col < (DCOLS as i64 - 1)) { 1 } else { 0 };
	let i_start = if (rogue.row > MIN_ROW) { -1 } else { 0 };
	let j_start = if (rogue.col > 0) { -1 } else { 0 };
	for i in i_start..=i_end {
		for j in j_start..=j_end {
			if ((i == 0) && (j == 0)) {
				continue;
			}
			if (((rogue.row + i) == drow) && ((rogue.col + j) == dcol)) {
				continue;
			}
			row = rogue.row + i;
			col = rogue.col + j;
			let s = dungeon[row as usize][col as usize];
			if Hidden.is_set(s) {
				continue;
			}
			/* If the rogue used to be right, up, left, down, or right of
			 * row,col, and now isn't, then don't stop */
			if SpotFlag::is_any_set(&vec![Monster, Object, Stairs], s) {
				if (((row == drow) || (col == dcol)) &&
					(!((row == rogue.row) || (col == rogue.col)))) {
					continue;
				}
				return true;
			}
			if Trap.is_set(s) {
				if !Hidden.is_set(s) {
					if (((row == drow) || (col == dcol)) &&
						(!((row == rogue.row) || (col == rogue.col)))) {
						continue;
					}
					return true;
				}
			}
			if ((((i - j) == 1) || ((i - j) == -1)) && (Tunnel.is_set(s))) {
				pass_count += 1;
				if pass_count > 1 {
					return true;
				}
			}
			if (Door.is_set(s)) && ((i == 0) || (j == 0)) {
				return true;
			}
		}
	}
	return false;
}

pub unsafe fn can_move(row1: i64, col1: i64, row2: i64, col2: i64) -> bool {
	if is_passable(row2 as libc::c_int, col2 as libc::c_int) {
		if row1 != row2 && col1 != col2 {
			if Door.is_set(dungeon[row1 as usize][col1 as usize]) || Door.is_set(dungeon[row2 as usize][col2 as usize]) {
				return false;
			}
			if Nothing.is_set(dungeon[row1 as usize][col2 as usize]) || Nothing.is_set(dungeon[row2 as usize][col1 as usize]) {
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
			message("direction? ", 0);
			first_miss = 0 as libc::c_int as libc::c_char;
		}
	}
	check_message();
	let ch = ch as u8 as char;
	if ch != '\u{1b}' {
		one_move_rogue(ch, false);
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
		hunger_str = "hungry".to_string();
		message(&hunger_str, 0);
		print_stats(STAT_HUNGER);
	}
	if rogue.moves_left as libc::c_int == 150 as libc::c_int {
		hunger_str = "weak".to_string();
		message(&hunger_str, 1);
		print_stats(STAT_HUNGER);
	}
	if rogue.moves_left as libc::c_int <= 20 as libc::c_int {
		if rogue.moves_left as libc::c_int == 20 as libc::c_int {
			hunger_str = "faint".to_string();
			message(&hunger_str, 1);
			print_stats(STAT_HUNGER);
		}
		n = get_rand(
			0 as libc::c_int,
			20 as libc::c_int - rogue.moves_left as libc::c_int,
		) as libc::c_short;
		if n as libc::c_int > 0 as libc::c_int {
			fainted = 1 as libc::c_int as libc::c_char;
			if rand_percent(40) {
				rogue.moves_left += 1;
				rogue.moves_left;
			}
			message("you faint", 1);
			i = 0 as libc::c_int as libc::c_short;
			while (i as libc::c_int) < n as libc::c_int {
				if coin_toss() {
					mv_mons();
				}
				i += 1;
				i;
			}
			message(you_can_move_again, 1);
		}
	}
	if messages_only != 0 {
		return fainted;
	}
	if rogue.moves_left as libc::c_int <= 0 as libc::c_int {
		killed_by(0 as *mut object, 2);
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
			check_hunger(1);
			rogue
				.moves_left = (rogue.moves_left as libc::c_int
				- rogue.moves_left as libc::c_int % 2 as libc::c_int) as libc::c_short;
		}
		2 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
			check_hunger(1);
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
		fainted = check_hunger(0);
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
			message("you float gently to the ground", 1);
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
	interrupted = false;
	i = 0 as libc::c_int;
	while i < count {
		if interrupted {
			break;
		}
		reg_move();
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}