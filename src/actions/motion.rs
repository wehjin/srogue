use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::pack::has_amulet;
use crate::score::win;
use crate::systems::play_level::LevelResult;
use crate::systems::play_level::LevelResult::{ExitWon, StairsDown, StairsUp};

pub struct Descend;

impl GameUpdater for Descend {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if drop_check(game) {
			Some(StairsDown)
		} else {
			None
		}
	}
}

pub struct Ascend;

impl GameUpdater for Ascend {
	fn update(game: &mut GameState) -> Option<LevelResult> {
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
			game.diary.add_entry("you're floating in the air!");
			return false;
		}
		return true;
	}
	game.diary.add_entry("I see no way down");
	false
}

pub enum UpResult {
	KeepLevel,
	UpLevel,
	WonGame,
}

fn check_up(game: &mut GameState) -> UpResult {
	if !game.player.wizard {
		if !game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_stairs() {
			game.diary.add_entry("I see no way up");
			return UpResult::KeepLevel;
		}
		if !has_amulet(&game.player) {
			game.diary.add_entry("Your way is magically blocked");
			return UpResult::KeepLevel;
		}
	}
	game.level.new_level_message = Some("you feel a wrenching sensation in your gut".to_string());
	if game.player.cur_depth == 1 {
		win(game);
		UpResult::WonGame
	} else {
		game.player.ascend();
		UpResult::UpLevel
	}
}
