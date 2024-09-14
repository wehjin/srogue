use crate::actions::motion::UpResult::{KeepLevel, UpLevel, WonGame};
use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::motion::{multiple_move_rogue, one_move_rogue, MoveDirection};
use crate::pack::has_amulet;
use crate::score::win;
use crate::systems::play_level::LevelResult;
use crate::systems::play_level::LevelResult::{ExitWon, StairsDown, StairsUp};

pub struct MoveOnce;

impl GameUpdater for MoveOnce {
	fn update(input_key: char, game: &mut GameState) -> Option<LevelResult> {
		let direction = MoveDirection::from(input_key);
		one_move_rogue(direction, true, game);
		None
	}
}

pub struct MoveMultiple;

impl GameUpdater for MoveMultiple {
	fn update(input_key: char, game: &mut GameState) -> Option<LevelResult> {
		multiple_move_rogue(input_key, game);
		None
	}
}

pub struct Descend;

impl GameUpdater for Descend {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		if drop_check(game) {
			Some(StairsDown)
		} else {
			None
		}
	}
}

pub struct Ascend;

impl GameUpdater for Ascend {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		match check_up(game) {
			UpResult::KeepLevel => None,
			UpResult::UpLevel => Some(StairsUp),
			UpResult::WonGame => Some(ExitWon),
		}
	}
}

fn drop_check(game: &mut GameState) -> bool {
	if game.player.wizard {
		return true;
	}
	if game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_stairs() {
		if game.player.levitate.is_active() {
			game.dialog.message("you're floating in the air!", 0);
			return false;
		}
		return true;
	}
	game.dialog.message("I see no way down", 0);
	return false;
}

pub enum UpResult {
	KeepLevel,
	UpLevel,
	WonGame,
}

fn check_up(game: &mut GameState) -> UpResult {
	if !game.player.wizard {
		if !game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_stairs() {
			game.dialog.message("I see no way up", 0);
			return KeepLevel;
		}
		if !has_amulet(&game.player) {
			game.dialog.message("Your way is magically blocked", 0);
			return KeepLevel;
		}
	}
	game.level.new_level_message = Some("you feel a wrenching sensation in your gut".to_string());
	if game.player.cur_depth == 1 {
		win(game);
		WonGame
	} else {
		game.player.ascend();
		UpLevel
	}
}
