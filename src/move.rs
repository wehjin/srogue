#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{chtype, mvaddch, refresh};
use MoveResult::MoveFailed;
use crate::hunger::HUNGRY;
use crate::odds::R_TELE_PERCENT;
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::Gold;
use crate::prelude::SpotFlag::{Door, Hidden, Monster, Nothing, Object, Stairs, Trap, Tunnel};
use crate::prelude::stat_const::{STAT_HP, STAT_HUNGER};
use crate::r#move::MoveResult::{Moved, StoppedOnSomething};
use crate::settings::jump;

pub static mut m_moves: i16 = 0;
pub static YOU_CAN_MOVE_AGAIN: &'static str = "you can move again";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}


pub unsafe fn one_move_rogue(dirch: char, pickup: bool, depth: &RogueDepth, level: &Level) -> MoveResult {
	let dirch = if confused != 0 { Move::random8().to_char() } else { dirch };
	let mut row = rogue.row;
	let mut col = rogue.col;
	get_dir_rc(dirch, &mut row, &mut col, true);
	if !can_move(rogue.row, rogue.col, row, col) {
		return MoveFailed;
	}
	if being_held || bear_trap > 0 {
		if !Monster.is_set(dungeon[row as usize][col as usize]) {
			if being_held {
				message("you are being held", 1);
			} else {
				message("you are still stuck in the bear trap", 0);
				reg_move(depth, level);
			}
			return MoveFailed;
		}
	}
	if r_teleport {
		if rand_percent(R_TELE_PERCENT) {
			tele(level);
			return StoppedOnSomething;
		}
	}
	if Monster.is_set(dungeon[row as usize][col as usize]) {
		rogue_hit(
			&mut MASH.monster_at_spot_mut(row, col).expect("monster at spot"),
			false,
			depth,
		);
		reg_move(depth, level);
		return MoveFailed;
	}
	if Door.is_set(dungeon[row as usize][col as usize]) {
		if cur_room == PASSAGE {
			cur_room = get_room_number(row, col, level);
			light_up_room(cur_room, level);
			wake_room(cur_room, true, row, col, level);
		} else {
			light_passage(row, col);
		}
	} else if Door.is_set(dungeon[rogue.row as usize][rogue.col as usize]) && Tunnel.is_set(dungeon[row as usize][col as usize]) {
		light_passage(row, col);
		wake_room(cur_room, false, rogue.row, rogue.col, level);
		darken_room(cur_room, level);
		cur_room = PASSAGE;
	} else if Tunnel.is_set(dungeon[row as usize][col as usize]) {
		light_passage(row, col);
	}
	mvaddch(rogue.row as i32, rogue.col as i32, get_dungeon_char(rogue.row, rogue.col));
	mvaddch(row as i32, col as i32, chtype::from(rogue.fchar));
	if !jump() {
		refresh();
	}
	rogue.row = row;
	rogue.col = col;
	if Object.is_set(dungeon[row as usize][col as usize]) {
		if levitate != 0 && pickup {
			return StoppedOnSomething;
		}
		if pickup && levitate == 0 {
			let mut status = 0;
			let obj = pick_up(row, col, &mut status, depth, level);
			if !obj.is_null() {
				if (*obj).what_is == Gold {
					free_object(obj);
					not_in_pack(&get_desc(&*obj), depth, level)
				} else {
					let desc = format!("{}({})", get_desc(&*obj), (*obj).ichar);
					not_in_pack(&desc, depth, level)
				}
			} else if status != 0 {
				mved(depth, level)
			} else {
				move_on(row, col, depth, level)
			}
		} else {
			move_on(row, col, depth, level)
		}
	} else if SpotFlag::is_any_set(&vec![Door, Stairs, Trap], dungeon[row as usize][col as usize]) {
		if levitate == 0 && Trap.is_set(dungeon[row as usize][col as usize]) {
			trap_player(row as usize, col as usize, depth, level);
		}
		reg_move(depth, level);
		StoppedOnSomething
	} else {
		mved(depth, level)
	}
}

unsafe fn move_on(row: i64, col: i64, depth: &RogueDepth, level: &Level) -> MoveResult {
	let obj = object_at(&level_objects, row, col);
	let desc = format!("moved onto {}", get_desc(&*obj));
	return not_in_pack(&desc, depth, level);
}

unsafe fn not_in_pack(desc: &str, depth: &RogueDepth, level: &Level) -> MoveResult {
	message(desc, 1);
	reg_move(depth, level);
	return StoppedOnSomething;
}

unsafe fn mved(depth: &RogueDepth, level: &Level) -> MoveResult {
	if reg_move(depth, level) {
		/* fainted from hunger */
		StoppedOnSomething
	} else {
		if confused != 0 { StoppedOnSomething } else { Moved }
	}
}

const BS: char = '\x08';
const LF: char = '\x0a';
const VT: char = '\x0b';
const FF: char = '\x0c';
const EM: char = '\x19';
const NAK: char = '\x15';
const SO: char = '\x0e';
const STX: char = '\x02';

pub unsafe fn multiple_move_rogue(dirch: i64, depth: &RogueDepth, level: &Level) {
	let dirch = dirch as u8 as char;
	match dirch {
		BS | LF | VT | FF | EM | NAK | SO | STX => loop {
			let row = rogue.row;
			let col = rogue.col;
			let m = one_move_rogue((dirch as u8 + 96) as char, true, depth, level);
			if m == MoveFailed || m == StoppedOnSomething || interrupted {
				break;
			}
			if next_to_something(row, col) {
				break;
			}
		},
		'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' => {
			loop {
				if interrupted {
					break;
				}
				let one_move_result = one_move_rogue((dirch as u8 + 32) as char, true, depth, level);
				if one_move_result != Moved {
					break;
				}
			}
		}
		_ => {}
	}
}

pub unsafe fn is_passable(row: i64, col: i64) -> bool {
	if is_off_screen(row, col) {
		return false;
	}
	if Hidden.is_set(dungeon[row as usize][col as usize]) {
		return if Trap.is_set(dungeon[row as usize][col as usize]) { true } else { false };
	}
	return SpotFlag::is_any_set(&vec![SpotFlag::Floor, Tunnel, Door, Stairs, Trap], dungeon[row as usize][col as usize]);
}

pub unsafe fn next_to_something(drow: i64, dcol: i64) -> bool {
	if confused != 0 {
		return true;
	}
	if blind != 0 {
		return false;
	}
	let mut row = 0;
	let mut col = 0;
	let mut pass_count = 0;
	let i_end = if rogue.row < (DROWS as i64 - 2) { 1 } else { 0 };
	let j_end = if rogue.col < (DCOLS as i64 - 1) { 1 } else { 0 };
	let i_start = if rogue.row > MIN_ROW { -1 } else { 0 };
	let j_start = if rogue.col > 0 { -1 } else { 0 };
	for i in i_start..=i_end {
		for j in j_start..=j_end {
			if (i == 0) && (j == 0) {
				continue;
			}
			if ((rogue.row + i) == drow) && ((rogue.col + j) == dcol) {
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
				if ((row == drow) || (col == dcol)) &&
					(!((row == rogue.row) || (col == rogue.col))) {
					continue;
				}
				return true;
			}
			if Trap.is_set(s) {
				if !Hidden.is_set(s) {
					if ((row == drow) || (col == dcol)) &&
						(!((row == rogue.row) || (col == rogue.col))) {
						continue;
					}
					return true;
				}
			}
			if (((i - j) == 1) || ((i - j) == -1)) && (Tunnel.is_set(s)) {
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
	if is_passable(row2, col2) {
		if row1 != row2 && col1 != col2 {
			if Door.is_set(dungeon[row1 as usize][col1 as usize]) || Door.is_set(dungeon[row2 as usize][col2 as usize]) {
				return false;
			}
			if Nothing.is_set(dungeon[row1 as usize][col2 as usize]) || Nothing.is_set(dungeon[row2 as usize][col1 as usize]) {
				return false;
			}
		}
		true
	} else {
		false
	}
}

pub unsafe fn move_onto(depth: &RogueDepth, level: &Level) {
	let ch = get_dir_or_cancel();
	check_message();
	if ch != CANCEL {
		one_move_rogue(ch, false, depth, level);
	}
}

pub unsafe fn get_dir_or_cancel() -> char {
	let mut dir = CANCEL;
	let mut first_miss: bool = true;
	loop {
		dir = rgetchar();
		if is_direction(dir) {
			break;
		}
		sound_bell();
		if first_miss {
			message("direction? ", 0);
			first_miss = false;
		}
	}
	dir
}

pub unsafe fn is_direction(c: char) -> bool {
	c == 'h' || c == 'j' || c == 'k' || c == 'l'
		|| c == 'b' || c == 'y' || c == 'u' || c == 'n'
		|| c == CANCEL
}

pub unsafe fn check_hunger(mut messages_only: libc::c_char, depth: &RogueDepth, level: &Level) -> bool {
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut fainted: bool = false;
	if rogue.moves_left as libc::c_int == 300 as libc::c_int {
		hunger_str = "hungry".to_string();
		message(&hunger_str, 0);
		print_stats(STAT_HUNGER, depth.cur);
	}
	if rogue.moves_left as libc::c_int == 150 as libc::c_int {
		hunger_str = "weak".to_string();
		message(&hunger_str, 1);
		print_stats(STAT_HUNGER, depth.cur);
	}
	if rogue.moves_left as libc::c_int <= 20 as libc::c_int {
		if rogue.moves_left as libc::c_int == 20 as libc::c_int {
			hunger_str = "faint".to_string();
			message(&hunger_str, 1);
			print_stats(STAT_HUNGER, depth.cur);
		}
		n = get_rand(
			0 as libc::c_int,
			20 as libc::c_int - rogue.moves_left as libc::c_int,
		) as libc::c_short;
		if n as libc::c_int > 0 as libc::c_int {
			fainted = true;
			if rand_percent(40) {
				rogue.moves_left += 1;
				rogue.moves_left;
			}
			message("you faint", 1);
			i = 0 as libc::c_int as libc::c_short;
			while (i as libc::c_int) < n as libc::c_int {
				if coin_toss() {
					mv_mons(depth, level);
				}
				i += 1;
			}
			message(YOU_CAN_MOVE_AGAIN, 1);
		}
	}
	if messages_only != 0 {
		return fainted;
	}
	if rogue.moves_left as libc::c_int <= 0 as libc::c_int {
		killed_by(Ending::Starvation, depth.max);
	}
	match e_rings as libc::c_int {
		-1 => {
			rogue.moves_left = (rogue.moves_left as libc::c_int - rogue.moves_left as libc::c_int % 2 as libc::c_int) as usize;
		}
		0 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
		}
		1 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
			check_hunger(1, depth, level);
			rogue.moves_left = (rogue.moves_left as libc::c_int - rogue.moves_left as libc::c_int % 2 as libc::c_int) as usize;
		}
		2 => {
			rogue.moves_left -= 1;
			rogue.moves_left;
			check_hunger(1, depth, level);
			rogue.moves_left -= 1;
			rogue.moves_left;
		}
		_ => {}
	}
	return fainted;
}

pub unsafe fn reg_move(depth: &RogueDepth, level: &Level) -> bool {
	let fainted = if rogue.moves_left <= HUNGRY || depth.cur >= depth.max {
		check_hunger(0, depth, level)
	} else {
		false
	};
	mv_mons(depth, level);
	m_moves += 1;
	if m_moves >= 120 {
		m_moves = 0;
		put_wanderer(depth.cur, level);
	}
	if halluc != 0 {
		halluc -= 1;
		if halluc == 0 {
			unhallucinate(level);
		} else {
			hallucinate();
		}
	}
	if blind != 0 {
		blind -= 1;
		if blind == 0 {
			unblind(level);
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
	}
	if levitate != 0 {
		levitate -= 1;
		if levitate == 0 {
			message("you float gently to the ground", 1);
			if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
				& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
			{
				trap_player(rogue.row as usize, rogue.col as usize, depth, level);
			}
		}
	}
	if haste_self > 0 {
		haste_self -= 1;
		if haste_self == 0 {
			message("you feel yourself slowing down", 0);
		}
	}
	heal(depth.cur);
	if auto_search > 0 {
		search(auto_search as usize, auto_search > 0, depth, level);
	}
	return fainted;
}

pub unsafe fn rest(count: libc::c_int, depth: &RogueDepth, level: &Level) {
	interrupted = false;
	for _i in 0..count {
		if interrupted {
			break;
		}
		reg_move(depth, level);
	}
}


pub unsafe fn heal(level_depth: usize) {
	static mut heal_exp: isize = -1;
	static mut n: isize = 0;
	static mut c: isize = 0;
	static mut alt: bool = false;

	if rogue.hp_current == rogue.hp_max {
		c = 0;
		return;
	}
	if rogue.exp != heal_exp {
		heal_exp = rogue.exp;
		match heal_exp {
			1 => { n = 20 }
			2 => { n = 18 }
			3 => { n = 17 }
			4 => { n = 14 }
			5 => { n = 13 }
			6 => { n = 10 }
			7 => { n = 9 }
			8 => { n = 8 }
			9 => { n = 7 }
			10 => { n = 4 }
			11 => { n = 3 }
			_ => { n = 2; }
		}
	}
	c += 1;
	if c >= n {
		c = 0;
		rogue.hp_current += 1;
		alt = !alt;
		if alt {
			rogue.hp_current += 1;
		}
		rogue.hp_current += regeneration;
		if rogue.hp_current > rogue.hp_max {
			rogue.hp_current = rogue.hp_max;
		}
		print_stats(STAT_HP, level_depth);
	}
}
