use rand::thread_rng;
use OnceResult::Idle;

use crate::actions::action_set::PlayerEvent;
use crate::init::{Dungeon, GameState, GameTurn};
use crate::motion::{multiple_move_rogue, one_move_rogue_legacy};
use crate::render_system;
use crate::resources::diary;
use crate::resources::keyboard::rgetchar;
use crate::systems::play_level::LevelResult;
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
	game.turn = GameTurn::Player;
	game.diary.turn_page();
	if key_code.is_digit(10) {
		render_system::refresh(game);
		return Counting(key_code.to_string());
	}
	match PlayerEvent::try_from(key_code) {
		Ok(player_event) => {
			if let Some(level_result) = dispatch_player_event(game, player_event) {
				return Leaving(level_result);
			}
			if game.turn == GameTurn::Monsters {
				// TODO reg_move(game);
			}
		}
		Err(e) => {
			game.diary.add_entry(e.to_string());
		}
	}
	diary::show_current_page(&game.diary);
	render_system::refresh(game);
	Idle
}

fn dispatch_player_event(game: &mut GameState, player_event: PlayerEvent) -> Option<LevelResult> {
	let rng = &mut thread_rng();
	match player_event {
		PlayerEvent::MoveRogue(direction, until) => {
			match until {
				Some(until) => { multiple_move_rogue(direction, until, game); }
				None => { one_move_rogue_legacy(direction, true, game, rng); }
			}
			None
		}
		PlayerEvent::Update(update_game) => update_game(game),
	}
}

fn test_and_clear_loop_context(game: &mut GameState) -> Option<LevelResult> {
	game.player.interrupted = false;
	if !game.as_diary().hit_message.is_none() {
		game.player.interrupt_and_slurp();
		let diary = game.as_diary_mut();
		let message = diary.hit_message.take().unwrap_or_else(String::new);
		diary.add_entry(&message);
	}
	if game.level.trap_door {
		game.level.trap_door = false;
		return Some(LevelResult::TrapDoorDown);
	}
	render_system::refresh(game);
	None
}

