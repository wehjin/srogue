#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch, refresh};
use MoveResult::MoveFailed;
use crate::hunger::HUNGRY;
use crate::level::constants::{DCOLS, DROWS};
use crate::odds::R_TELE_PERCENT;
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::stat_const::{STAT_HP, STAT_HUNGER};
use crate::r#move::MoveResult::{Moved, StoppedOnSomething};
use crate::ring::effects::{auto_search, e_rings};
use crate::settings::jump;

pub static mut m_moves: i16 = 0;
pub const YOU_CAN_MOVE_AGAIN: &'static str = "you can move again";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}


pub unsafe fn one_move_rogue(dirch: char, pickup: bool, player: &mut Player, level: &mut Level) -> MoveResult {
	let dirch = if confused != 0 { Move::random8().to_char() } else { dirch };
	let mut row = player.rogue.row;
	let mut col = player.rogue.col;
	get_dir_rc(dirch, &mut row, &mut col, true);
	if !can_move(player.rogue.row, player.rogue.col, row, col, level) {
		return MoveFailed;
	}
	if level.being_held || level.bear_trap > 0 {
		if !level.dungeon[row as usize][col as usize].is_monster() {
			if level.being_held {
				message("you are being held", 1);
			} else {
				message("you are still stuck in the bear trap", 0);
				reg_move(player, level);
			}
			return MoveFailed;
		}
	}
	if player.ring_effects.has_teleport() && rand_percent(R_TELE_PERCENT) {
		tele(player, level);
		return StoppedOnSomething;
	}
	if level.dungeon[row as usize][col as usize].is_monster() {
		rogue_hit(
			&mut MASH.monster_at_spot_mut(row, col).expect("monster at spot"),
			false,
			player,
			level,
		);
		reg_move(player, level);
		return MoveFailed;
	}
	if level.dungeon[row as usize][col as usize].is_door() {
		if cur_room == PASSAGE {
			cur_room = get_room_number(row, col, level);
			light_up_room(cur_room, player, level);
			wake_room(cur_room, true, row, col, player, level);
		} else {
			light_passage(row, col, level);
		}
	} else if level.dungeon[player.rogue.row as usize][player.rogue.col as usize].is_door() && level.dungeon[row as usize][col as usize].is_tunnel() {
		light_passage(row, col, level);
		wake_room(cur_room, false, player.rogue.row, player.rogue.col, player, level);
		darken_room(cur_room, level);
		cur_room = PASSAGE;
	} else if level.dungeon[row as usize][col as usize].is_tunnel() {
		light_passage(row, col, level);
	}
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, get_dungeon_char(player.rogue.row, player.rogue.col, level));
	mvaddch(row as i32, col as i32, chtype::from(player.rogue.fchar));
	if !jump() {
		refresh();
	}
	player.rogue.row = row;
	player.rogue.col = col;
	if level.dungeon[row as usize][col as usize].is_object() {
		if !pickup {
			stopped_on_something_with_moved_onto_message(row, col, player, level)
		} else {
			if levitate != 0 {
				StoppedOnSomething
			} else {
				match pick_up(row, col, player, level) {
					PickUpResult::TurnedToDust => {
						moved_unless_hungry_or_confused(player, level)
					}
					PickUpResult::AddedToGold(obj) => {
						let msg = get_obj_desc(&obj);
						stopped_on_something_with_message(&msg, player, level)
					}
					PickUpResult::AddedToPack(obj) => {
						let msg = get_inv_obj_desc(obj);
						stopped_on_something_with_message(&msg, player, level)
					}
					PickUpResult::PackTooFull => {
						stopped_on_something_with_moved_onto_message(row, col, player, level)
					}
				}
			}
		}
	} else if level.dungeon[row as usize][col as usize].is_any_kind(&[CellKind::Door, CellKind::Stairs, CellKind::Trap]) {
		if levitate == 0 && level.dungeon[row as usize][col as usize].is_trap() {
			trap_player(row as usize, col as usize, player, level);
		}
		reg_move(player, level);
		StoppedOnSomething
	} else {
		moved_unless_hungry_or_confused(player, level)
	}
}

unsafe fn stopped_on_something_with_moved_onto_message(row: i64, col: i64, player: &mut Player, level: &mut Level) -> MoveResult {
	let obj = level_objects.find_object_at(row, col).expect("moved-on object");
	let desc = format!("moved onto {}", get_obj_desc(obj));
	return stopped_on_something_with_message(&desc, player, level);
}

unsafe fn stopped_on_something_with_message(desc: &str, player: &mut Player, level: &mut Level) -> MoveResult {
	message(desc, 1);
	reg_move(player, level);
	return StoppedOnSomething;
}

unsafe fn moved_unless_hungry_or_confused(player: &mut Player, level: &mut Level) -> MoveResult {
	if reg_move(player, level) {
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

pub unsafe fn multiple_move_rogue(dirch: i64, player: &mut Player, level: &mut Level) {
	let dirch = dirch as u8 as char;
	match dirch {
		BS | LF | VT | FF | EM | NAK | SO | STX => loop {
			let row = player.rogue.row;
			let col = player.rogue.col;
			let m = one_move_rogue((dirch as u8 + 96) as char, true, player, level);
			if m == MoveFailed || m == StoppedOnSomething || interrupted {
				break;
			}
			if next_to_something(row, col, player, level) {
				break;
			}
		},
		'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' => {
			loop {
				if interrupted {
					break;
				}
				let one_move_result = one_move_rogue((dirch as u8 + 32) as char, true, player, level);
				if one_move_result != Moved {
					break;
				}
			}
		}
		_ => {}
	}
}

pub fn is_passable(row: i64, col: i64, level: &Level) -> bool {
	if is_off_screen(row, col) {
		false
	} else {
		if level.dungeon[row as usize][col as usize].is_hidden() {
			level.dungeon[row as usize][col as usize].is_trap()
		} else {
			let PASSABLE_CELL_KINDS = [CellKind::Floor, CellKind::Tunnel, CellKind::Door, CellKind::Stairs, CellKind::Trap];
			level.dungeon[row as usize][col as usize].is_any_kind(&PASSABLE_CELL_KINDS)
		}
	}
}

pub unsafe fn next_to_something(drow: i64, dcol: i64, player: &Player, level: &Level) -> bool {
	if confused != 0 {
		return true;
	}
	if blind != 0 {
		return false;
	}
	let mut row;
	let mut col;
	let mut pass_count = 0;
	let i_end = if player.rogue.row < (DROWS as i64 - 2) { 1 } else { 0 };
	let j_end = if player.rogue.col < (DCOLS as i64 - 1) { 1 } else { 0 };
	let i_start = if player.rogue.row > MIN_ROW { -1 } else { 0 };
	let j_start = if player.rogue.col > 0 { -1 } else { 0 };
	for i in i_start..=i_end {
		for j in j_start..=j_end {
			if (i == 0) && (j == 0) {
				continue;
			}
			if ((player.rogue.row + i) == drow) && ((player.rogue.col + j) == dcol) {
				continue;
			}
			row = player.rogue.row + i;
			col = player.rogue.col + j;
			let s = level.dungeon[row as usize][col as usize];
			if s.is_hidden() {
				continue;
			}
			/* If the rogue used to be right, up, left, down, or right of
			 * row,col, and now isn't, then don't stop */
			if s.is_any_kind(&[CellKind::Monster, CellKind::Object, CellKind::Stairs]) {
				if ((row == drow) || (col == dcol)) &&
					(!((row == player.rogue.row) || (col == player.rogue.col))) {
					continue;
				}
				return true;
			}
			if s.is_trap() {
				if !s.is_hidden() {
					if ((row == drow) || (col == dcol)) &&
						(!((row == player.rogue.row) || (col == player.rogue.col))) {
						continue;
					}
					return true;
				}
			}
			if (((i - j) == 1) || ((i - j) == -1)) && s.is_tunnel() {
				pass_count += 1;
				if pass_count > 1 {
					return true;
				}
			}
			if s.is_door() && ((i == 0) || (j == 0)) {
				return true;
			}
		}
	}
	return false;
}

pub fn can_move(row1: i64, col1: i64, row2: i64, col2: i64, level: &Level) -> bool {
	if !is_passable(row2, col2, level) {
		false
	} else {
		if row1 != row2 && col1 != col2 {
			if level.dungeon[row1 as usize][col1 as usize].is_door()
				|| level.dungeon[row2 as usize][col2 as usize].is_door() {
				return false;
			}
			if level.dungeon[row1 as usize][col2 as usize].is_nothing()
				|| level.dungeon[row2 as usize][col1 as usize].is_nothing() {
				return false;
			}
		}
		true
	}
}

pub unsafe fn move_onto(player: &mut Player, level: &mut Level) {
	let ch = get_dir_or_cancel();
	check_message();
	if ch != CANCEL {
		one_move_rogue(ch, false, player, level);
	}
}

pub unsafe fn get_dir_or_cancel() -> char {
	let mut dir: char;
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

pub unsafe fn check_hunger(messages_only: libc::c_char, player: &mut Player, level: &mut Level) -> bool {
	let mut i: libc::c_short;
	let n: libc::c_short;
	let mut fainted: bool = false;
	if player.rogue.moves_left as libc::c_int == 300 as libc::c_int {
		hunger_str = "hungry".to_string();
		message(&hunger_str, 0);
		print_stats(STAT_HUNGER, player);
	}
	if player.rogue.moves_left as libc::c_int == 150 as libc::c_int {
		hunger_str = "weak".to_string();
		message(&hunger_str, 1);
		print_stats(STAT_HUNGER, player);
	}
	if player.rogue.moves_left as libc::c_int <= 20 as libc::c_int {
		if player.rogue.moves_left as libc::c_int == 20 as libc::c_int {
			hunger_str = "faint".to_string();
			message(&hunger_str, 1);
			print_stats(STAT_HUNGER, player);
		}
		n = get_rand(
			0 as libc::c_int,
			20 as libc::c_int - player.rogue.moves_left as libc::c_int,
		) as libc::c_short;
		if n as libc::c_int > 0 as libc::c_int {
			fainted = true;
			if rand_percent(40) {
				player.rogue.moves_left += 1;
				player.rogue.moves_left;
			}
			message("you faint", 1);
			i = 0 as libc::c_int as libc::c_short;
			while (i as libc::c_int) < n as libc::c_int {
				if coin_toss() {
					mv_mons(player, level);
				}
				i += 1;
			}
			message(YOU_CAN_MOVE_AGAIN, 1);
		}
	}
	if messages_only != 0 {
		return fainted;
	}
	if player.rogue.moves_left as libc::c_int <= 0 as libc::c_int {
		killed_by(Ending::Starvation, player);
	}
	match e_rings as libc::c_int {
		-1 => {
			player.rogue.moves_left = (player.rogue.moves_left as libc::c_int - player.rogue.moves_left as libc::c_int % 2 as libc::c_int) as usize;
		}
		0 => {
			player.rogue.moves_left -= 1;
			player.rogue.moves_left;
		}
		1 => {
			player.rogue.moves_left -= 1;
			player.rogue.moves_left;
			check_hunger(1, player, level);
			player.rogue.moves_left = (player.rogue.moves_left as libc::c_int - player.rogue.moves_left as libc::c_int % 2 as libc::c_int) as usize;
		}
		2 => {
			player.rogue.moves_left -= 1;
			player.rogue.moves_left;
			check_hunger(1, player, level);
			player.rogue.moves_left -= 1;
			player.rogue.moves_left;
		}
		_ => {}
	}
	return fainted;
}

pub unsafe fn reg_move(player: &mut Player, level: &mut Level) -> bool {
	let fainted = if player.rogue.moves_left <= HUNGRY || player.cur_depth >= player.max_depth {
		check_hunger(0, player, level)
	} else {
		false
	};
	mv_mons(player, level);
	m_moves += 1;
	if m_moves >= 120 {
		m_moves = 0;
		put_wanderer(player, level);
	}
	if halluc != 0 {
		halluc -= 1;
		if halluc == 0 {
			unhallucinate(player, level);
		} else {
			hallucinate(player);
		}
	}
	if blind != 0 {
		blind -= 1;
		if blind == 0 {
			unblind(player, level);
		}
	}
	if confused != 0 {
		confused -= 1;
		if confused == 0 {
			unconfuse();
		}
	}
	if level.bear_trap > 0 {
		level.bear_trap -= 1;
	}
	if levitate != 0 {
		levitate -= 1;
		if levitate == 0 {
			message("you float gently to the ground", 1);
			if level.dungeon[player.rogue.row as usize][player.rogue.col as usize].is_trap() {
				trap_player(player.rogue.row as usize, player.rogue.col as usize, player, level);
			}
		}
	}
	if haste_self > 0 {
		haste_self -= 1;
		if haste_self == 0 {
			message("you feel yourself slowing down", 0);
		}
	}
	heal(player);
	if auto_search > 0 {
		search(auto_search as usize, auto_search > 0, player, level);
	}
	return fainted;
}

pub unsafe fn rest(count: libc::c_int, player: &mut Player, level: &mut Level) {
	interrupted = false;
	for _i in 0..count {
		if interrupted {
			break;
		}
		reg_move(player, level);
	}
}


pub unsafe fn heal(player: &mut Player) {
	static mut heal_level: isize = -1;
	static mut time_between_heals: isize = 0;
	static mut time_since_last_heal_or_damage: isize = 0;
	static mut double_healing_toggle: bool = false;

	if player.rogue.hp_current == player.rogue.hp_max {
		time_since_last_heal_or_damage = 0;
		return;
	}
	if player.rogue.exp != heal_level {
		heal_level = player.rogue.exp;
		time_between_heals = match heal_level {
			1 => 20,
			2 => 18,
			3 => 17,
			4 => 14,
			5 => 13,
			6 => 10,
			7 => 9,
			8 => 8,
			9 => 7,
			10 => 4,
			11 => 3,
			_ => 2,
		}
	}
	time_since_last_heal_or_damage += 1;
	if time_since_last_heal_or_damage >= time_between_heals {
		time_since_last_heal_or_damage = 0;
		double_healing_toggle = !double_healing_toggle;

		let mut healing = player.rogue.hp_current;
		healing += 1;
		if double_healing_toggle {
			healing += 1;
		}
		healing += player.ring_effects.regeneration();
		player.rogue.hp_current = healing.min(player.rogue.hp_max);
		print_stats(STAT_HP, player);
	}
}
