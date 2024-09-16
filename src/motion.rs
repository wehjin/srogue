use MoveResult::MoveFailed;

use crate::actions::search::{search, SearchKind};
use crate::components::hunger::{HungerLevel, FAINT_MOVES_LEFT, HUNGRY_MOVES_LEFT, STARVE_MOVES_LEFT, WEAK_MOVES_LEFT};
use crate::hit::rogue_hit;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::message::sound_bell;
use crate::monster::{mv_mons, put_wanderer, wake_room};
use crate::motion::MoveResult::{Moved, StoppedOnSomething};
use crate::odds::R_TELE_PERCENT;
use crate::pack::{pick_up, PickUpResult};
use crate::player::{Player, RoomMark};
use crate::prelude::ending::Ending;
use crate::prelude::{DungeonSpot, MIN_ROW};
use crate::r#use::{tele, unblind, unconfuse, unhallucinate};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::render_system;
use crate::render_system::darken_room;
use crate::render_system::hallucinate::show_hallucination;
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::room::{visit_room, visit_spot_area};
use crate::score::killed_by;
use crate::throw::Motion;
use crate::trap::{is_off_screen, trap_player};

pub const YOU_CAN_MOVE_AGAIN: &'static str = "you can move again";

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}

pub fn one_move_rogue(direction: MoveDirection, pickup: bool, game: &mut GameState) -> MoveResult {
	let real_direction = if game.player.confused.is_active() {
		MoveDirection::random()
	} else {
		direction
	};
	let (row, col) = real_direction.apply(game.player.rogue.row, game.player.rogue.col);
	if !can_move(game.player.rogue.row, game.player.rogue.col, row, col, &game.level) {
		return MoveFailed;
	}
	if game.level.being_held || game.level.bear_trap > 0 {
		if !game.level.dungeon[row as usize][col as usize].has_monster() {
			if game.level.being_held {
				game.player.interrupt_and_slurp();
				game.diary.add_entry("you are being held");
			} else {
				game.diary.add_entry("you are still stuck in the bear trap");
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

	let to_cell = game.level.cell(DungeonSpot { row, col });
	if to_cell.is_any_door() {
		match game.player.cur_room {
			RoomMark::None => {}
			RoomMark::Passage => {
				// tunnel to door
				game.player.cur_room = game.level.room(row, col);
				let cur_rn = game.player.cur_room.rn().expect("current room should be the room at rol,col");
				visit_room(cur_rn, game);
				wake_room(cur_rn, true, row, col, game);
			}
			RoomMark::Cavern(_) => {
				// room to door
				visit_spot_area(row, col, game);
			}
		}
	} else if game.player.cur_cell(&game.level).is_any_door() && to_cell.is_any_tunnel() {
		// door to tunnel
		visit_spot_area(row, col, game);
		let rn = game.player.cur_room.rn().expect("player room not an area moving from door to passage");
		wake_room(rn, false, game.player.rogue.row, game.player.rogue.col, game);
		darken_room(rn, game);
		game.player.cur_room = RoomMark::Passage;
	} else if to_cell.is_any_tunnel() {
		// tunnel to tunnel.
		visit_spot_area(row, col, game);
	} else {
		// room to room, door to room
	}
	let vacated_spot = game.player.to_spot();
	game.render_spot(vacated_spot);
	game.player.rogue.row = row;
	game.player.rogue.col = col;
	let occupied_post = game.player.to_spot();
	game.render_spot(occupied_post);
	if !game.player.settings.jump {
		render_system::refresh(game);
	}
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
	} else if player_cell.is_any_door() || player_cell.is_stairs() || player_cell.is_any_trap() {
		if game.player.levitate.is_inactive() && player_cell.is_any_trap() {
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
	stopped_on_something_with_message(&desc, game)
}

fn stopped_on_something_with_message(desc: &str, game: &mut GameState) -> MoveResult {
	game.player.interrupt_and_slurp();
	game.diary.add_entry(desc);
	reg_move(game);
	StoppedOnSomething
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


pub fn multiple_move_rogue(direction: MoveDirection, until: MoveUntil, game: &mut GameState) {
	match until {
		MoveUntil::Obstacle => {
			loop {
				if game.player.interrupted {
					break;
				}
				if one_move_rogue(direction, true, game) != Moved {
					break;
				}
				render_system::refresh(game);
			}
		}
		MoveUntil::NearSomething => {
			loop {
				let row = game.player.rogue.row;
				let col = game.player.rogue.col;
				let result = one_move_rogue(direction, true, game);
				if result == MoveFailed || result == StoppedOnSomething || game.player.interrupted {
					break;
				}
				if next_to_something(row, col, &game.player, &game.level) {
					break;
				}
				render_system::refresh(game);
			}
		}
	}
}

pub fn is_passable(row: i64, col: i64, level: &Level) -> bool {
	if is_off_screen(row, col) {
		false
	} else {
		let cell = level.dungeon[row as usize][col as usize];
		if cell.is_any_hidden() {
			cell.is_any_trap()
		} else {
			cell.is_any_floor() || cell.is_any_tunnel() || cell.is_any_door() || cell.is_stairs() || cell.is_any_trap()
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
			if s.is_any_hidden() {
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
			if s.is_any_trap() {
				if !s.is_any_hidden() {
					if ((row == drow) || (col == dcol)) &&
						(!((row == player.rogue.row) || (col == player.rogue.col))) {
						continue;
					}
					return true;
				}
			}
			if (((i - j) == 1) || ((i - j) == -1)) && s.is_any_tunnel() {
				pass_count += 1;
				if pass_count > 1 {
					return true;
				}
			}
			if s.is_any_door() && ((i == 0) || (j == 0)) {
				return true;
			}
		}
	}
	false
}

pub fn can_move(row1: i64, col1: i64, row2: i64, col2: i64, level: &Level) -> bool {
	if !is_passable(row2, col2, level) {
		false
	} else {
		if row1 != row2 && col1 != col2 {
			if level.dungeon[row1 as usize][col1 as usize].is_any_door()
				|| level.dungeon[row2 as usize][col2 as usize].is_any_door() {
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
			diary::show_prompt("direction? ", &mut game.diary);
			first_miss = false;
		}
	}
	dir
}

pub fn is_direction(c: char) -> bool {
	c == 'h' || c == 'j' || c == 'k' || c == 'l'
		|| c == 'b' || c == 'y' || c == 'u' || c == 'n'
		|| c == CANCEL_CHAR
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
		game.diary.add_entry(&game.player.hunger.as_str());
		game.stats_changed = true;
	}

	if game.player.hunger == HungerLevel::Starved {
		killed_by(Ending::Starvation, game);
		return HungerCheckResult::DidStarve;
	}
	if game.player.hunger == HungerLevel::Faint && random_faint(game) {
		return HungerCheckResult::DidFaint;
	}
	HungerCheckResult::StillWalking
}

fn random_faint(game: &mut GameState) -> bool {
	let n = get_rand(0, FAINT_MOVES_LEFT - game.player.rogue.moves_left);
	if n > 0 {
		if rand_percent(40) {
			game.player.rogue.moves_left += 1;
		}
		game.player.interrupt_and_slurp();
		game.diary.add_entry("you faint");
		for _ in 0..n {
			if coin_toss() {
				mv_mons(game);
			}
		}
		game.player.interrupt_and_slurp();
		game.diary.add_entry(YOU_CAN_MOVE_AGAIN);
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
			show_hallucination(game);
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
			game.diary.add_entry("you float gently to the ground");
			if game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_any_trap() {
				trap_player(game.player.rogue.row as usize, game.player.rogue.col as usize, game);
			}
		}
	}
	if game.player.haste_self.is_active() {
		game.player.haste_self.decr();
		if game.player.haste_self.is_inactive() {
			game.diary.add_entry("you feel yourself slowing down");
		}
	}
	game.heal_player();
	{
		let auto_search = game.player.ring_effects.auto_search();
		if auto_search > 0 {
			search(SearchKind::Auto { n: auto_search as usize }, game);
		}
	}
	hunger_check == HungerCheckResult::DidFaint
}


#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum MoveUntil {
	Obstacle,
	NearSomething,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum MoveDirection {
	Left,
	Right,
	Up,
	Down,
	DownLeft,
	DownRight,
	UpLeft,
	UpRight,
}

impl From<char> for MoveDirection {
	fn from(value: char) -> Self {
		// Moves CTRL and SHIFT chars into the lower-case region of the ascii table.
		let ascii = value as u8;
		let lowercase = ((ascii % 32) + 96) as char;
		match lowercase {
			'h' => MoveDirection::Left,
			'j' => MoveDirection::Down,
			'k' => MoveDirection::Up,
			'l' => MoveDirection::Right,
			'y' => MoveDirection::UpLeft,
			'u' => MoveDirection::UpRight,
			'n' => MoveDirection::DownRight,
			'b' => MoveDirection::DownLeft,
			_ => unreachable!("invalid direction")
		}
	}
}

impl MoveDirection {
	pub fn random() -> Self {
		MoveDirection::from(Motion::random8().to_char())
	}
	pub fn apply_confined(&self, row: i64, col: i64) -> (usize, usize) {
		let (free_row, free_col) = self.apply(row, col);
		let confined_row = free_row.max(MIN_ROW).min(DROWS as i64 - 2) as usize;
		let confined_col = free_col.max(0).min(DCOLS as i64 - 1) as usize;
		(confined_row, confined_col)
	}
	pub fn apply(&self, row: i64, col: i64) -> (i64, i64) {
		let (drow, dcol) = self.to_offsets();
		(row + drow, col + dcol)
	}
	pub fn to_offsets(&self) -> (i64, i64) {
		match self {
			MoveDirection::Left => (0, -1),
			MoveDirection::Right => (0, 1),
			MoveDirection::Up => (-1, 0),
			MoveDirection::Down => (1, 0),
			MoveDirection::DownLeft => (1, -1),
			MoveDirection::DownRight => (1, 1),
			MoveDirection::UpLeft => (-1, -1),
			MoveDirection::UpRight => (-1, 1),
		}
	}
}