#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch, refresh};

use MoveResult::MoveFailed;

use crate::hit::{get_dir_rc, rogue_hit};
use crate::hunger::{FAINT, HUNGRY, STARVE, WEAK};
use crate::inventory::get_obj_desc;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::message::{CANCEL, check_message, hunger_str, message, print_stats, rgetchar, sound_bell};
use crate::monster::{MonsterMash, mv_mons, put_wanderer, wake_room};
use crate::objects::LEVEL_OBJECTS;
use crate::odds::R_TELE_PERCENT;
use crate::pack::{pick_up, PickUpResult};
use crate::play::interrupted;
use crate::player::{Player, RoomMark};
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::stat_const::{STAT_HP, STAT_HUNGER};
use crate::r#move::MoveResult::{Moved, StoppedOnSomething};
use crate::r#use::{hallucinate_on_screen, tele, unblind, unconfuse, unhallucinate};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::{darken_room, get_dungeon_char, light_passage, light_up_room};
use crate::score::killed_by;
use crate::throw::Move;
use crate::trap::{is_off_screen, search, trap_player};

pub static mut m_moves: i16 = 0;
pub const YOU_CAN_MOVE_AGAIN: &'static str = "you can move again";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}


pub unsafe fn one_move_rogue(dirch: char, pickup: bool, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> MoveResult {
	let dirch = if player.confused.is_active() {
		Move::random8().to_char()
	} else {
		dirch
	};
	let mut row = player.rogue.row;
	let mut col = player.rogue.col;
	get_dir_rc(dirch, &mut row, &mut col, true);
	if !can_move(player.rogue.row, player.rogue.col, row, col, level) {
		return MoveFailed;
	}
	if level.being_held || level.bear_trap > 0 {
		if !level.dungeon[row as usize][col as usize].has_monster() {
			if level.being_held {
				message("you are being held", 1);
			} else {
				message("you are still stuck in the bear trap", 0);
				reg_move(mash, player, level);
			}
			return MoveFailed;
		}
	}
	if player.ring_effects.has_teleport() && rand_percent(R_TELE_PERCENT) {
		tele(mash, player, level);
		return StoppedOnSomething;
	}
	if level.dungeon[row as usize][col as usize].has_monster() {
		let mon_id = mash.monster_id_at_spot(row, col).expect("monster in mash at monster spot one_move_rogue");
		rogue_hit(mon_id, false, mash, player, level);
		reg_move(mash, player, level);
		return MoveFailed;
	}

	let to_cell = level.cell(row, col);
	if to_cell.is_door() {
		match player.cur_room {
			RoomMark::None => {}
			RoomMark::Passage => {
				// tunnel to door
				player.cur_room = level.room(row, col);
				let cur_rn = player.cur_room.rn().expect("current room should be the room at rol,col");
				light_up_room(cur_rn, mash, player, level);
				wake_room(cur_rn, true, row, col, mash, player, level);
			}
			RoomMark::Area(_) => {
				// room to door
				light_passage(row, col, mash, player, level);
			}
		}
	} else if player.cur_cell(level).is_door() && to_cell.is_tunnel() {
		// door to tunnel
		light_passage(row, col, mash, player, level);
		let rn = player.cur_room.rn().expect("player room not an area moving from door to passage");
		wake_room(rn, false, player.rogue.row, player.rogue.col, mash, player, level);
		darken_room(rn, mash, player, level);
		player.cur_room = RoomMark::Passage;
	} else if to_cell.is_tunnel() {
		// tunnel to tunnel.
		light_passage(row, col, mash, player, level);
	} else {
		// room to room, door to room
	}
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, get_dungeon_char(player.rogue.row, player.rogue.col, mash, player, level));
	mvaddch(row as i32, col as i32, chtype::from(player.rogue.fchar));
	if !player.settings.jump {
		refresh();
	}

	player.rogue.row = row;
	player.rogue.col = col;
	let player_cell = level.dungeon[row as usize][col as usize];
	if player_cell.has_object() {
		if !pickup {
			stopped_on_something_with_moved_onto_message(row, col, mash, player, level)
		} else {
			if player.levitate.is_active() {
				StoppedOnSomething
			} else {
				match pick_up(row, col, player, level) {
					PickUpResult::TurnedToDust => {
						moved_unless_hungry_or_confused(mash, player, level)
					}
					PickUpResult::AddedToGold(obj) => {
						let msg = get_obj_desc(&obj, player.settings.fruit.to_string(), &player.notes);
						stopped_on_something_with_message(&msg, mash, player, level)
					}
					PickUpResult::AddedToPack { added_id, .. } => {
						let msg = player.get_obj_desc(added_id);
						stopped_on_something_with_message(&msg, mash, player, level)
					}
					PickUpResult::PackTooFull => {
						stopped_on_something_with_moved_onto_message(row, col, mash, player, level)
					}
				}
			}
		}
	} else if player_cell.is_door() || player_cell.is_stairs() || player_cell.is_trap() {
		if player.levitate.is_inactive() && player_cell.is_trap() {
			trap_player(row as usize, col as usize, mash, player, level);
		}
		reg_move(mash, player, level);
		StoppedOnSomething
	} else {
		moved_unless_hungry_or_confused(mash, player, level)
	}
}

unsafe fn stopped_on_something_with_moved_onto_message(row: i64, col: i64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> MoveResult {
	let obj = LEVEL_OBJECTS.find_object_at(row, col).expect("moved-on object");
	let obj_desc = get_obj_desc(obj, player.settings.fruit.to_string(), &player.notes);
	let desc = format!("moved onto {}", obj_desc);
	return stopped_on_something_with_message(&desc, mash, player, level);
}

unsafe fn stopped_on_something_with_message(desc: &str, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> MoveResult {
	message(desc, 1);
	reg_move(mash, player, level);
	return StoppedOnSomething;
}

unsafe fn moved_unless_hungry_or_confused(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> MoveResult {
	if reg_move(mash, player, level) {
		/* fainted from hunger */
		StoppedOnSomething
	} else {
		if player.confused.is_active() {
			StoppedOnSomething
		} else {
			Moved
		}
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

pub unsafe fn multiple_move_rogue(dirch: i64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	let dirch = dirch as u8 as char;
	match dirch {
		BS | LF | VT | FF | EM | NAK | SO | STX => loop {
			let row = player.rogue.row;
			let col = player.rogue.col;
			let m = one_move_rogue((dirch as u8 + 96) as char, true, mash, player, level);
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
				let one_move_result = one_move_rogue((dirch as u8 + 32) as char, true, mash, player, level);
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
		let cell = level.dungeon[row as usize][col as usize];
		if cell.is_hidden() {
			cell.is_trap()
		} else {
			cell.is_floor() || cell.is_tunnel() || cell.is_door() || cell.is_stairs() || cell.is_trap()
		}
	}
}

pub unsafe fn next_to_something(drow: i64, dcol: i64, player: &Player, level: &Level) -> bool {
	if player.confused.is_active() {
		return true;
	}
	if player.blind.is_active() {
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
			if s.has_monster() || s.has_object() || s.is_stairs() {
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

pub unsafe fn move_onto(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	let ch = get_dir_or_cancel();
	check_message();
	if ch != CANCEL {
		one_move_rogue(ch, false, mash, player, level);
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

pub unsafe fn check_hunger(messages_only: bool, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> bool {
	if player.rogue.moves_left == HUNGRY {
		hunger_str = "hungry".to_string();
		message(&hunger_str, 0);
		print_stats(STAT_HUNGER, player);
	}
	if player.rogue.moves_left == WEAK {
		hunger_str = "weak".to_string();
		message(&hunger_str, 1);
		print_stats(STAT_HUNGER, player);
	}

	let mut fainted = false;
	if player.rogue.moves_left <= FAINT {
		if player.rogue.moves_left == FAINT {
			hunger_str = "faint".to_string();
			message(&hunger_str, 1);
			print_stats(STAT_HUNGER, player);
		}
		let n = get_rand(0, FAINT - player.rogue.moves_left);
		if n > 0 {
			fainted = true;
			if rand_percent(40) {
				player.rogue.moves_left += 1;
			}
			message("you faint", 1);
			for _ in 0..n {
				if coin_toss() {
					mv_mons(mash, player, level);
				}
			}
			message(YOU_CAN_MOVE_AGAIN, 1);
		}
	}
	if messages_only {
		return fainted;
	}
	if player.rogue.moves_left <= STARVE {
		killed_by(Ending::Starvation, player);
	}

	match player.ring_effects.calorie_burn() {
		-1 => {
			player.rogue.moves_left -= player.rogue.moves_left % 2;
		}
		0 => {
			player.rogue.moves_left -= 1;
		}
		1 => {
			player.rogue.moves_left -= 1;
			check_hunger(true, mash, player, level);
			player.rogue.moves_left -= player.rogue.moves_left % 2;
		}
		2 => {
			player.rogue.moves_left -= 1;
			check_hunger(true, mash, player, level);
			player.rogue.moves_left -= 1;
		}
		_ => {
			// No burn for -2
		}
	}
	return fainted;
}

pub unsafe fn reg_move(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) -> bool {
	let fainted = if player.rogue.moves_left <= HUNGRY || player.cur_depth >= player.max_depth {
		check_hunger(false, mash, player, level)
	} else {
		false
	};
	mv_mons(mash, player, level);
	m_moves += 1;
	if m_moves >= 120 {
		m_moves = 0;
		put_wanderer(mash, player, level);
	}
	if player.halluc.is_active() {
		player.halluc.decr();
		if player.halluc.is_active() {
			hallucinate_on_screen(mash, player);
		} else {
			unhallucinate(mash, player, level);
		}
	}
	if player.blind.is_active() {
		player.blind.decr();
		if player.blind.is_inactive() {
			unblind(mash, player, level);
		}
	}
	if player.confused.is_active() {
		player.confused.decr();
		if player.confused.is_inactive() {
			unconfuse(player);
		}
	}
	if level.bear_trap > 0 {
		level.bear_trap -= 1;
	}
	if player.levitate.is_active() {
		player.levitate.decr();
		if player.levitate.is_inactive() {
			message("you float gently to the ground", 1);
			if level.dungeon[player.rogue.row as usize][player.rogue.col as usize].is_trap() {
				trap_player(player.rogue.row as usize, player.rogue.col as usize, mash, player, level);
			}
		}
	}
	if player.haste_self.is_active() {
		player.haste_self.decr();
		if player.haste_self.is_inactive() {
			message("you feel yourself slowing down", 0);
		}
	}
	heal(player);

	let auto_search = player.ring_effects.auto_search();
	if auto_search > 0 {
		search(auto_search as usize, auto_search > 0, mash, player, level);
	}
	return fainted;
}

pub unsafe fn rest(count: libc::c_int, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	interrupted = false;
	for _i in 0..count {
		if interrupted {
			break;
		}
		reg_move(mash, player, level);
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
