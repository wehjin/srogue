use rand::{thread_rng, Rng};
use MoveResult::MoveFailed;

use crate::components::hunger::{HungerLevel, FAINT_MOVES_LEFT, HUNGRY_MOVES_LEFT, STARVE_MOVES_LEFT, WEAK_MOVES_LEFT};
use crate::hit::rogue_hit;
use crate::init::{Dungeon, GameState};
use crate::inventory::get_obj_desc_legacy;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::message::sound_bell;
use crate::monster::{mv_mons, put_wanderer};
use crate::motion::MoveResult::{Moved, StoppedOnSomething};
use crate::odds::R_TELE_PERCENT;
use crate::pack::{pick_up_legacy, PickUpResult};
use crate::player::{Player, RoomMark};
use crate::prelude::ending::Ending;
use crate::prelude::{DungeonSpot, MIN_ROW};
use crate::r#use::tele;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::render_system;
use crate::render_system::darken_room;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::diary;
use crate::resources::dice::roll_chance;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::resources::level::wake::wake_room_legacy;
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
				if dungeon.as_health().being_held {
					dungeon.interrupt_and_slurp();
					dungeon.as_diary_mut().add_entry("you are being held");
					return MoveEffect::Fail { consume_time: false };
				} else {
					dungeon.as_diary_mut().add_entry("you are still stuck in the bear trap");
					return MoveEffect::Fail { consume_time: true };
				}
			}
			if dungeon.as_ring_effects().has_teleport() && roll_chance(R_TELE_PERCENT, rng) {
				return MoveEffect::Teleport;
			}
			if dungeon.has_monster_at(row, col) {
				return MoveEffect::Fight { row, col };
			}
			if dungeon.is_any_door_at(row, col) {
				return MoveEffect::PrepToDoor { row, col, rogue_row, rogue_col };
			} else if dungeon.is_any_tunnel_at(row, col) {
				if dungeon.is_any_door_at(rogue_row, rogue_col) {
					return MoveEffect::PrepDoorToTunnel { row, col, rogue_row, rogue_col };
				} else {
					return MoveEffect::PrepTunnelToTunnel { row, col, rogue_row, rogue_col };
				}
			} else {
				// room to room, door to room
				return MoveEffect::PrepWithinRoom { row, col, rogue_row, rogue_col };
			}
		}
		MoveEvent::Continue { row, col, rogue_row, rogue_col } => {
			dungeon.set_rogue_row_col(row, col);
			MoveEffect::Done { row, col, rogue_row, rogue_col }
		}
	}
}

pub fn one_move_rogue_legacy(direction: MoveDirection, pickup: bool, game: &mut GameState, rng: &mut impl Rng) -> MoveResult {
	let mut next_event = Some(MoveEvent::Start { direction, pickup });
	while let Some(event) = next_event.take() {
		match dispatch_move_event(event, game, rng) {
			MoveEffect::Fail { consume_time } => {
				// TODO Call to reg_move into dispatch_move_event.
				if consume_time {
					reg_move(game);
				}
				return MoveFailed;
			}
			MoveEffect::Teleport => {
				// TODO Call to reg_move into dispatch_move_event.
				tele(game);
				return StoppedOnSomething;
			}
			MoveEffect::Fight { row, col } => {
				// TODO Call to reg_move into dispatch_move_event.
				let mon_id = game.mash.monster_id_at_spot(row, col).expect("monster in mash at monster spot one_move_rogue");
				rogue_hit(mon_id, false, game);
				reg_move(game);
				return MoveFailed;
			}
			MoveEffect::PrepToDoor { row, col, rogue_row, rogue_col } => {
				// TODO Call to reg_move into dispatch_move_event.
				match game.player.cur_room {
					RoomMark::None => {}
					RoomMark::Passage => {
						// tunnel to door
						game.player.cur_room = game.level.room(row, col);
						let cur_rn = game.player.cur_room.rn().expect("current room should be the room at rol,col");
						visit_room(cur_rn, game);
						wake_room_legacy(cur_rn, true, row, col, game);
					}
					RoomMark::Cavern(_) => {
						// room to door
						visit_spot_area(row, col, game);
					}
				}
				next_event = Some(MoveEvent::Continue { row, col, rogue_row, rogue_col })
			}
			MoveEffect::PrepDoorToTunnel { row, col, rogue_row, rogue_col } => {
				// door to tunnel
				visit_spot_area(row, col, game);
				let rn = game.player.cur_room.rn().expect("player room not an area moving from door to passage");
				wake_room_legacy(rn, false, rogue_row, rogue_col, game);
				darken_room(rn, game);
				game.player.cur_room = RoomMark::Passage;
				next_event = Some(MoveEvent::Continue { row, col, rogue_row, rogue_col })
			}
			MoveEffect::PrepTunnelToTunnel { row, col, rogue_row, rogue_col } => {
				visit_spot_area(row, col, game);
				next_event = Some(MoveEvent::Continue { row, col, rogue_row, rogue_col })
			}
			MoveEffect::PrepWithinRoom { row, col, rogue_row, rogue_col } => {
				next_event = Some(MoveEvent::Continue { row, col, rogue_row, rogue_col })
			}
			MoveEffect::Done { row, col, rogue_row, rogue_col } => {
				let vacated = DungeonSpot { row: rogue_row, col: rogue_col };
				let occupied = DungeonSpot { row, col };
				game.render_spot(vacated);
				game.render_spot(occupied);
				if !game.player.settings.jump {
					render_system::refresh(game);
				}
			}
		}
	}
	// TODO Move this portion into dispatch_move_event.
	let row = game.rogue_row();
	let col = game.rogue_col();
	let player_cell = game.level.dungeon[row as usize][col as usize];
	if player_cell.has_object() {
		if !pickup {
			stopped_on_something_with_moved_onto_message(row, col, game)
		} else {
			if game.as_health().levitate.is_active() {
				StoppedOnSomething
			} else {
				match pick_up_legacy(row, col, game) {
					PickUpResult::TurnedToDust => {
						moved_unless_hungry_or_confused(game)
					}
					PickUpResult::AddedToGold(obj) => {
						let msg = get_obj_desc_legacy(&obj, game.player.settings.fruit.to_string(), &game.player);
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
		if game.as_health().levitate.is_inactive() && player_cell.is_any_trap() {
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
	let obj_desc = get_obj_desc_legacy(obj, game.player.settings.fruit.to_string(), &game.player);
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
		if game.player.health.confused.is_active() {
			StoppedOnSomething
		} else {
			Moved
		}
	}
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

pub fn check_hunger(game: &mut impl Dungeon) -> HungerCheckResult {
	let moves_to_burn = match game.as_ring_effects().calorie_burn() {
		-2 => 0,
		-1 => game.as_fighter().moves_left % 2,
		0 => 1,
		1 => 1 + (game.as_fighter().moves_left % 2),
		2 => 2,
		_ => panic!("invalid calorie burn")
	};
	if moves_to_burn == 0 {
		return HungerCheckResult::StillWalking;
	}
	game.as_fighter_mut().moves_left -= moves_to_burn;
	if let Some(next_hunger) = get_hunger_transition_with_burn_count(game.as_fighter().moves_left, moves_to_burn) {
		let health = game.as_health_mut();
		health.hunger = next_hunger;

		let diary = game.as_diary_mut();
		diary.add_entry(next_hunger.as_str());
		diary.stats_changed = true;
	}

	let hunger = game.as_health().hunger;
	if hunger == HungerLevel::Starved {
		killed_by(Ending::Starvation, game);
		return HungerCheckResult::DidStarve;
	}
	if hunger == HungerLevel::Faint && random_faint(game) {
		return HungerCheckResult::DidFaint;
	}
	HungerCheckResult::StillWalking
}

fn random_faint(game: &mut impl Dungeon) -> bool {
	let n = get_rand(0, FAINT_MOVES_LEFT - game.as_fighter().moves_left);
	if n > 0 {
		if rand_percent(40) {
			game.as_fighter_mut().moves_left += 1;
		}
		game.interrupt_and_slurp();
		game.as_diary_mut().add_entry("you faint");
		for _ in 0..n {
			if coin_toss() {
				mv_mons(game);
			}
		}
		game.interrupt_and_slurp();
		game.as_diary_mut().add_entry(YOU_CAN_MOVE_AGAIN);
		true
	} else {
		false
	}
}

pub fn reg_move(game: &mut impl Dungeon) -> bool {
	let hunger_check = if game.as_fighter().moves_left <= HUNGRY_MOVES_LEFT || game.is_max_depth() {
		check_hunger(game)
	} else {
		HungerCheckResult::StillWalking
	};
	if hunger_check == HungerCheckResult::DidStarve {
		return true;
	}
	mv_mons(game);
	let next_m_move = game.m_moves() + 1;
	if next_m_move >= 120 {
		*game.m_moves_mut() = 0;
		put_wanderer(game);
	} else {
		*game.m_moves_mut() = next_m_move;
	}
	if game.as_health().halluc.is_active() {
		game.as_health_mut().halluc.decr();
		if game.as_health().halluc.is_active() {
			// TODO show_hallucination(game);
		} else {
			// TODO unhallucinate(game);
		}
	}
	if game.as_health().blind.is_active() {
		game.as_health_mut().blind.decr();
		if game.as_health().blind.is_inactive() {
			//TODO unblind(game);
		}
	}
	if game.as_health().confused.is_active() {
		game.as_health_mut().confused.decr();
		if game.as_health().confused.is_inactive() {
			// TODO unconfuse(game);
		}
	}
	if game.as_health().bear_trap > 0 {
		game.as_health_mut().bear_trap -= 1;
	}
	if game.as_health().levitate.is_active() {
		game.as_health_mut().levitate.decr();
		if game.as_health().levitate.is_inactive() {
			game.interrupt_and_slurp();
			game.as_diary_mut().add_entry("you float gently to the ground");
			let rogue_row = game.rogue_row();
			let rogue_col = game.rogue_col();
			if game.is_any_tunnel_at(rogue_row, rogue_col) {
				// TODO trap_player(rogue_row as usize, rogue_col as usize, game);
			}
		}
	}
	if game.as_health().haste_self.is_active() {
		game.as_health_mut().haste_self.decr();
		if game.as_health().haste_self.is_inactive() {
			game.as_diary_mut().add_entry("you feel yourself slowing down");
		}
	}
	//TODO game.heal_player();
	{
		let auto_search = game.as_ring_effects().auto_search();
		if auto_search > 0 {
			// TODO search(SearchKind::Auto { n: auto_search as usize }, game);
		}
	}
	hunger_check == HungerCheckResult::DidFaint
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