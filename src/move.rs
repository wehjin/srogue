#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch, refresh};

use MoveResult::MoveFailed;

use crate::components::hunger::{FAINT_MOVES_LEFT, HungerLevel, HUNGRY_MOVES_LEFT, STARVE_MOVES_LEFT, WEAK_MOVES_LEFT};
use crate::hit::{get_dir_rc, rogue_hit};
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::message::{CANCEL, print_stats, rgetchar, sound_bell};
use crate::monster::{mv_mons, put_wanderer, wake_room};
use crate::odds::R_TELE_PERCENT;
use crate::pack::{pick_up, PickUpResult};
use crate::player::{Player, RoomMark};
use crate::prelude::ending::Ending;
use crate::prelude::MIN_ROW;
use crate::prelude::stat_const::{STAT_HP, STAT_HUNGER};
use crate::r#move::MoveResult::{Moved, StoppedOnSomething};
use crate::r#use::{hallucinate_on_screen, tele, unblind, unconfuse, unhallucinate};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::{darken_room, get_dungeon_char, light_passage, light_up_room};
use crate::score::killed_by;
use crate::throw::Move;
use crate::trap::{is_off_screen, search, trap_player};

pub const YOU_CAN_MOVE_AGAIN: &'static str = "you can move again";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}


pub fn one_move_rogue(dirch: char, pickup: bool, game: &mut GameState) -> MoveResult {
	let dirch = if game.player.confused.is_active() {
		Move::random8().to_char()
	} else {
		dirch
	};
	let mut row = game.player.rogue.row;
	let mut col = game.player.rogue.col;
	get_dir_rc(dirch, &mut row, &mut col, true);
	if !can_move(game.player.rogue.row, game.player.rogue.col, row, col, &game.level) {
		return MoveFailed;
	}
	if game.level.being_held || game.level.bear_trap > 0 {
		if !game.level.dungeon[row as usize][col as usize].has_monster() {
			if game.level.being_held {
				game.player.interrupt_and_slurp();
				game.dialog.message("you are being held", 1);
			} else {
				game.dialog.message("you are still stuck in the bear trap", 0);
				reg_move(game);
			}
			return MoveFailed;
		}
	}
	if game.player.ring_effects.has_teleport() && rand_percent(R_TELE_PERCENT) {
		tele(game);
		return StoppedOnSomething;
	}
	if game.level.dungeon[row as usize][col as usize].has_monster() {
		let mon_id = game.mash.monster_id_at_spot(row, col).expect("monster in mash at monster spot one_move_rogue");
		rogue_hit(mon_id, false, game);
		reg_move(game);
		return MoveFailed;
	}

	let to_cell = game.level.cell(row, col);
	if to_cell.is_door() {
		match game.player.cur_room {
			RoomMark::None => {}
			RoomMark::Passage => {
				// tunnel to door
				game.player.cur_room = game.level.room(row, col);
				let cur_rn = game.player.cur_room.rn().expect("current room should be the room at rol,col");
				light_up_room(cur_rn, game);
				wake_room(cur_rn, true, row, col, game);
			}
			RoomMark::Area(_) => {
				// room to door
				light_passage(row, col, game);
			}
		}
	} else if game.player.cur_cell(&game.level).is_door() && to_cell.is_tunnel() {
		// door to tunnel
		light_passage(row, col, game);
		let rn = game.player.cur_room.rn().expect("player room not an area moving from door to passage");
		wake_room(rn, false, game.player.rogue.row, game.player.rogue.col, game);
		darken_room(rn, game);
		game.player.cur_room = RoomMark::Passage;
	} else if to_cell.is_tunnel() {
		// tunnel to tunnel.
		light_passage(row, col, game);
	} else {
		// room to room, door to room
	}
	mvaddch(game.player.rogue.row as i32, game.player.rogue.col as i32, get_dungeon_char(game.player.rogue.row, game.player.rogue.col, game));
	mvaddch(row as i32, col as i32, chtype::from(game.player.rogue.fchar));
	if !game.player.settings.jump {
		refresh();
	}

	game.player.rogue.row = row;
	game.player.rogue.col = col;
	let player_cell = game.level.dungeon[row as usize][col as usize];
	if player_cell.has_object() {
		if !pickup {
			stopped_on_something_with_moved_onto_message(row, col, game)
		} else {
			if game.player.levitate.is_active() {
				StoppedOnSomething
			} else {
				match pick_up(row, col, game) {
					PickUpResult::TurnedToDust => {
						moved_unless_hungry_or_confused(game)
					}
					PickUpResult::AddedToGold(obj) => {
						let msg = get_obj_desc(&obj, game.player.settings.fruit.to_string(), &game.player);
						stopped_on_something_with_message(&msg, game)
					}
					PickUpResult::AddedToPack { added_id, .. } => {
						let msg = game.player.get_obj_desc(added_id);
						stopped_on_something_with_message(&msg, game)
					}
					PickUpResult::PackTooFull => {
						stopped_on_something_with_moved_onto_message(row, col, game)
					}
				}
			}
		}
	} else if player_cell.is_door() || player_cell.is_stairs() || player_cell.is_trap() {
		if game.player.levitate.is_inactive() && player_cell.is_trap() {
			trap_player(row as usize, col as usize, game);
		}
		reg_move(game);
		StoppedOnSomething
	} else {
		moved_unless_hungry_or_confused(game)
	}
}

fn stopped_on_something_with_moved_onto_message(row: i64, col: i64, game: &mut GameState) -> MoveResult {
	let obj = game.ground.find_object_at(row, col).expect("moved-on object");
	let obj_desc = get_obj_desc(obj, game.player.settings.fruit.to_string(), &game.player);
	let desc = format!("moved onto {}", obj_desc);
	return stopped_on_something_with_message(&desc, game);
}

fn stopped_on_something_with_message(desc: &str, game: &mut GameState) -> MoveResult {
	game.player.interrupt_and_slurp();
	game.dialog.message(desc, 1);
	reg_move(game);
	return StoppedOnSomething;
}

fn moved_unless_hungry_or_confused(game: &mut GameState) -> MoveResult {
	if reg_move(game) {
		/* fainted from hunger */
		StoppedOnSomething
	} else {
		if game.player.confused.is_active() {
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

pub fn multiple_move_rogue(dirch: i64, game: &mut GameState) {
	let dirch = dirch as u8 as char;
	match dirch {
		BS | LF | VT | FF | EM | NAK | SO | STX => loop {
			let row = game.player.rogue.row;
			let col = game.player.rogue.col;
			let m = one_move_rogue((dirch as u8 + 96) as char, true, game);
			if m == MoveFailed || m == StoppedOnSomething || game.player.interrupted {
				break;
			}
			if next_to_something(row, col, &game.player, &game.level) {
				break;
			}
		},
		'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' => {
			loop {
				if game.player.interrupted {
					break;
				}
				let one_move_result = one_move_rogue((dirch as u8 + 32) as char, true, game);
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

pub fn next_to_something(drow: i64, dcol: i64, player: &Player, level: &Level) -> bool {
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

pub fn move_onto(game: &mut GameState) {
	let ch = get_dir_or_cancel(game);
	game.dialog.clear_message();
	if ch != CANCEL {
		one_move_rogue(ch, false, game);
	}
}

pub fn get_dir_or_cancel(game: &mut GameState) -> char {
	let mut dir: char;
	let mut first_miss: bool = true;
	loop {
		dir = rgetchar();
		if is_direction(dir) {
			break;
		}
		sound_bell();
		if first_miss {
			game.dialog.message("direction? ", 0);
			first_miss = false;
		}
	}
	dir
}

pub fn is_direction(c: char) -> bool {
	c == 'h' || c == 'j' || c == 'k' || c == 'l'
		|| c == 'b' || c == 'y' || c == 'u' || c == 'n'
		|| c == CANCEL
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HungerCheckResult {
	StillWalking,
	DidFaint,
	DidStarve,
}

fn get_hunger_transition(moves_left: isize) -> Option<HungerLevel> {
	match moves_left {
		HUNGRY_MOVES_LEFT => Some(HungerLevel::Hungry),
		WEAK_MOVES_LEFT => Some(HungerLevel::Weak),
		FAINT_MOVES_LEFT => Some(HungerLevel::Faint),
		STARVE_MOVES_LEFT => Some(HungerLevel::Starved),
		_ => if moves_left < STARVE_MOVES_LEFT { Some(HungerLevel::Starved) } else { None },
	}
}

fn get_hunger_transition_with_burn_count(moves_left: isize, moves_burned: isize) -> Option<HungerLevel> {
	match moves_burned {
		0 => None,
		1 => get_hunger_transition(moves_left),
		2 => get_hunger_transition(moves_left).or_else(|| get_hunger_transition(moves_left + 1)),
		_ => panic!("invalid moves_burned")
	}
}

pub fn check_hunger(game: &mut GameState) -> HungerCheckResult {
	let moves_to_burn = match game.player.ring_effects.calorie_burn() {
		-2 => 0,
		-1 => game.player.rogue.moves_left % 2,
		0 => 1,
		1 => 1 + (game.player.rogue.moves_left % 2),
		2 => 2,
		_ => panic!("invalid calorie burn")
	};
	if moves_to_burn == 0 {
		return HungerCheckResult::StillWalking;
	}
	game.player.rogue.moves_left -= moves_to_burn;
	if let Some(next_hunger) = get_hunger_transition_with_burn_count(game.player.rogue.moves_left, moves_to_burn) {
		game.player.hunger = next_hunger;
		game.dialog.message(&game.player.hunger.as_str(), 0);
		print_stats(STAT_HUNGER, &mut game.player);
	}

	if game.player.hunger == HungerLevel::Starved {
		killed_by(Ending::Starvation, game);
		return HungerCheckResult::DidStarve;
	}
	if game.player.hunger == HungerLevel::Faint && random_faint(game) {
		return HungerCheckResult::DidFaint;
	}
	return HungerCheckResult::StillWalking;
}

fn random_faint(game: &mut GameState) -> bool {
	let n = get_rand(0, FAINT_MOVES_LEFT - game.player.rogue.moves_left);
	if n > 0 {
		if rand_percent(40) {
			game.player.rogue.moves_left += 1;
		}
		game.player.interrupt_and_slurp();
		game.dialog.message("you faint", 1);
		for _ in 0..n {
			if coin_toss() {
				mv_mons(game);
			}
		}
		game.player.interrupt_and_slurp();
		game.dialog.message(YOU_CAN_MOVE_AGAIN, 1);
		true
	} else {
		false
	}
}

pub fn reg_move(game: &mut GameState) -> bool {
	let hunger_check = if game.player.rogue.moves_left <= HUNGRY_MOVES_LEFT || game.player.cur_depth >= game.player.max_depth {
		check_hunger(game)
	} else {
		HungerCheckResult::StillWalking
	};
	if hunger_check == HungerCheckResult::DidStarve {
		return true;
	}
	mv_mons(game);
	game.mash.m_moves += 1;
	if game.mash.m_moves >= 120 {
		game.mash.m_moves = 0;
		put_wanderer(game);
	}
	if game.player.halluc.is_active() {
		game.player.halluc.decr();
		if game.player.halluc.is_active() {
			hallucinate_on_screen(game);
		} else {
			unhallucinate(game);
		}
	}
	if game.player.blind.is_active() {
		game.player.blind.decr();
		if game.player.blind.is_inactive() {
			unblind(game);
		}
	}
	if game.player.confused.is_active() {
		game.player.confused.decr();
		if game.player.confused.is_inactive() {
			unconfuse(game);
		}
	}
	if game.level.bear_trap > 0 {
		game.level.bear_trap -= 1;
	}
	if game.player.levitate.is_active() {
		game.player.levitate.decr();
		if game.player.levitate.is_inactive() {
			game.player.interrupt_and_slurp();
			game.dialog.message("you float gently to the ground", 1);
			if game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_trap() {
				trap_player(game.player.rogue.row as usize, game.player.rogue.col as usize, game);
			}
		}
	}
	if game.player.haste_self.is_active() {
		game.player.haste_self.decr();
		if game.player.haste_self.is_inactive() {
			game.dialog.message("you feel yourself slowing down", 0);
		}
	}
	heal(&mut game.player);
	{
		let auto_search = game.player.ring_effects.auto_search();
		if auto_search > 0 {
			search(auto_search as usize, auto_search > 0, game);
		}
	}
	return hunger_check == HungerCheckResult::DidFaint;
}

pub fn rest(count: libc::c_int, game: &mut GameState) {
	game.player.interrupted = false;
	for _i in 0..count {
		if game.player.interrupted {
			break;
		}
		reg_move(game);
	}
}


pub fn heal(player: &mut Player) {
	static mut heal_level: isize = -1;
	static mut time_between_heals: isize = 0;
	static mut time_since_last_heal_or_damage: isize = 0;
	static mut double_healing_toggle: bool = false;

	if player.rogue.hp_current == player.rogue.hp_max {
		unsafe { time_since_last_heal_or_damage = 0; }
		return;
	}

	if player.rogue.exp != unsafe { heal_level } {
		unsafe { heal_level = player.rogue.exp; }
		unsafe {
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
	}

	unsafe { time_since_last_heal_or_damage += 1; }

	if unsafe { time_since_last_heal_or_damage } >= unsafe { time_between_heals } {
		unsafe { time_since_last_heal_or_damage = 0; }
		unsafe { double_healing_toggle = !double_healing_toggle; }

		let mut healing = player.rogue.hp_current;
		healing += 1;
		if unsafe { double_healing_toggle } {
			healing += 1;
		}
		healing += player.ring_effects.regeneration();
		player.rogue.hp_current = healing.min(player.rogue.hp_max);
		print_stats(STAT_HP, player);
	}
}
