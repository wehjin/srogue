use OnceResult::Idle;

use crate::actions::action_set::ACTION_UPDATES;
use crate::init::{GameState, GameTurn};
use crate::motion::reg_move;
use crate::render_system;
use crate::resources::keyboard::rgetchar;
use crate::systems::play_level::{LevelResult, UNKNOWN_COMMAND};
use crate::systems::play_once::OnceResult::{Counting, Leaving};

pub enum OnceResult {
	Counting(String),
	Leaving(LevelResult),
	Idle,
}

pub fn play_once(key_code: Option<char>, game: &mut GameState) -> OnceResult {
	if let Some(ending) = test_and_clear_loop_context(game) {
		return Leaving(ending);
	};
	let key_code = key_code.unwrap_or_else(rgetchar);
	// Keep rgetchar above clear_message(). Otherwise, the dialog row on screen
	// does not draw correctly.
	game.dialog.clear_message();
	game.turn = GameTurn::Player;
	if key_code.is_digit(10) {
		render_system::refresh(game);
		return Counting(key_code.to_string());
	} else if let Some(action_update) = ACTION_UPDATES.get(&key_code) {
		let action_result = action_update(key_code, game);
		if let Some(play_result) = action_result {
			return Leaving(play_result);
		}
		if game.turn == GameTurn::Monsters {
			reg_move(game);
		}
	} else {
		game.dialog.message(UNKNOWN_COMMAND, 0);
	}
	render_system::refresh(game);
	return Idle;
}

fn test_and_clear_loop_context(game: &mut GameState) -> Option<LevelResult> {
	game.player.interrupted = false;
	if !game.player.hit_message.is_empty() {
		game.player.interrupt_and_slurp();
		game.dialog.message(&game.player.hit_message, 1);
		game.player.hit_message.clear();
	}
	if game.level.trap_door {
		game.level.trap_door = false;
		return Some(LevelResult::TrapDoorDown);
	}
	render_system::refresh(game);
	return None;
}

