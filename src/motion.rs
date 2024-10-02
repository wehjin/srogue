use rand::{thread_rng, Rng};
use MoveResult::MoveFailed;

use crate::init::{Dungeon, GameState};
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::message::sound_bell;
use crate::motion::MoveResult::{Moved, StoppedOnSomething};
use crate::odds::R_TELE_PERCENT;
use crate::player::Player;
use crate::prelude::MIN_ROW;
use crate::render_system;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::diary;
use crate::resources::dice::roll_chance;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::throw::Motion;
use crate::trap::is_off_screen;

pub const YOU_CAN_MOVE_AGAIN: &'static str = "you can move again";

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MoveResult {
	Moved,
	MoveFailed,
	StoppedOnSomething,
}

pub enum MoveEvent {
	Start { direction: MoveDirection, pickup: bool },
	Continue { row: i64, col: i64, rogue_row: i64, rogue_col: i64 },
}
pub enum MoveEffect {
	Fail { consume_time: bool },
	Teleport,
	Fight { row: i64, col: i64 },
	PrepToDoor { row: i64, col: i64, rogue_row: i64, rogue_col: i64 },
	PrepDoorToTunnel { row: i64, col: i64, rogue_row: i64, rogue_col: i64 },
	PrepTunnelToTunnel { row: i64, col: i64, rogue_row: i64, rogue_col: i64 },
	PrepWithinRoom { row: i64, col: i64, rogue_row: i64, rogue_col: i64 },
	Done { row: i64, col: i64, rogue_row: i64, rogue_col: i64 },
}

pub fn dispatch_move_event(event: MoveEvent, dungeon: &mut impl Dungeon, rng: &mut impl Rng) -> MoveEffect {
	match event {
		MoveEvent::Start { direction, pickup: _pickup } => {
			let confused_direction = if dungeon.as_health().confused.is_active() { MoveDirection::random(rng) } else { direction };
			let rogue_row = dungeon.rogue_row();
			let rogue_col = dungeon.rogue_col();
			let (row, col) = confused_direction.apply(rogue_row, rogue_col);
			if !dungeon.rogue_can_move(row, col) {
				return MoveEffect::Fail { consume_time: false };
			}
			let rogue_stuck = dungeon.as_health().being_held || dungeon.as_health().bear_trap > 0;
			if rogue_stuck && !dungeon.has_monster_at(row, col) {
				return if dungeon.as_health().being_held {
					dungeon.interrupt_and_slurp();
					dungeon.as_diary_mut().add_entry("you are being held");
					MoveEffect::Fail { consume_time: false }
				} else {
					dungeon.as_diary_mut().add_entry("you are still stuck in the bear trap");
					MoveEffect::Fail { consume_time: true }
				};
			}
			if dungeon.as_ring_effects().has_teleport() && roll_chance(R_TELE_PERCENT, rng) {
				return MoveEffect::Teleport;
			}
			if dungeon.has_monster_at(row, col) {
				return MoveEffect::Fight { row, col };
			}
			if dungeon.is_any_door_at(row, col) {
				MoveEffect::PrepToDoor { row, col, rogue_row, rogue_col }
			} else if dungeon.is_any_tunnel_at(row, col) {
				if dungeon.is_any_door_at(rogue_row, rogue_col) {
					MoveEffect::PrepDoorToTunnel { row, col, rogue_row, rogue_col }
				} else {
					MoveEffect::PrepTunnelToTunnel { row, col, rogue_row, rogue_col }
				}
			} else {
				// room to room, door to room
				MoveEffect::PrepWithinRoom { row, col, rogue_row, rogue_col }
			}
		}
		MoveEvent::Continue { row, col, rogue_row, rogue_col } => {
			dungeon.set_rogue_row_col(row, col);
			MoveEffect::Done { row, col, rogue_row, rogue_col }
		}
	}
}

// TODO Delete this function after converting multiple_move_rogue
pub fn one_move_rogue_legacy(_direction: MoveDirection, _pickup: bool, _game: &mut GameState, _rng: &mut impl Rng) -> MoveResult {
	StoppedOnSomething
}


pub fn multiple_move_rogue(direction: MoveDirection, until: MoveUntil, game: &mut GameState) {
	let rng = &mut thread_rng();
	match until {
		MoveUntil::Obstacle => {
			loop {
				if game.player.interrupted {
					break;
				}
				if one_move_rogue_legacy(direction, true, game, rng) != Moved {
					break;
				}
				render_system::refresh(game);
			}
		}
		MoveUntil::NearSomething => {
			loop {
				let row = game.player.rogue.row;
				let col = game.player.rogue.col;
				let result = one_move_rogue_legacy(direction, true, game, rng);
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
	if player.health.confused.is_active() {
		return true;
	}
	if player.health.blind.is_active() {
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum MoveUntil {
	Obstacle,
	NearSomething,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
	pub fn random(rng: &mut impl Rng) -> Self {
		MoveDirection::from(Motion::random8(rng).to_char())
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