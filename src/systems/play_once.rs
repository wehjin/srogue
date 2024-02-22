use crate::actions::action_set::ACTION_UPDATES;
use crate::init::{GameState, GameTurn};
use crate::level::show_average_hp;
use crate::monster::show_monsters;
use crate::motion::reg_move;
use crate::objects::{new_object_for_wizard, show_objects};
use crate::pack::kick_into_pack;
use crate::render_system;
use crate::resources::keyboard::rgetchar;
use crate::room::draw_magic_map;
use crate::save::save_game;
use crate::systems::play_level::{PlayResult, UNKNOWN_COMMAND};
use crate::systems::play_once::PlayOnceResult::{Counting, Leaving};
use crate::trap::show_traps;

pub enum PlayOnceResult {
	Counting(String),
	Leaving(PlayResult),
	Idle,
}

pub fn play_once(key_code: Option<char>, game: &mut GameState) -> PlayOnceResult {
	if let Some(ending) = check_reset_loop_flags(game) {
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
		match key_code {
			'\x13' => if game.player.wizard {
				draw_magic_map(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x14' => if game.player.wizard {
				show_traps(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x0f' => if game.player.wizard {
				show_objects(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x01' => {
				show_average_hp(game);
			}
			'\x03' => if game.player.wizard {
				new_object_for_wizard(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x0d' => if game.player.wizard {
				show_monsters(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'S' => if save_game(game) {
				return Leaving(PlayResult::ExitSaved);
			},
			',' => {
				kick_into_pack(game);
			}
			_ => {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			}
		}
	}
	render_system::refresh(game);
	return PlayOnceResult::Idle;
}

fn check_reset_loop_flags(game: &mut GameState) -> Option<PlayResult> {
	game.player.interrupted = false;
	if !game.player.hit_message.is_empty() {
		game.player.interrupt_and_slurp();
		game.dialog.message(&game.player.hit_message, 1);
		game.player.hit_message.clear();
	}
	if game.level.trap_door {
		game.level.trap_door = false;
		return Some(PlayResult::TrapDoorDown);
	}
	render_system::refresh(game);
	return None;
}

