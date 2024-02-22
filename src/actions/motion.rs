use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::level::drop_check;
use crate::motion::{multiple_move_rogue, one_move_rogue};
use crate::systems::play_level::PlayResult;
use crate::systems::play_level::PlayResult::StairsDown;

pub struct MoveOnce;

impl PlayerAction for MoveOnce {
	fn update(input_key: char, game: &mut GameState) -> Option<PlayResult> {
		one_move_rogue(input_key, true, game);
		None
	}
}

pub struct MoveMultiple;

impl PlayerAction for MoveMultiple {
	fn update(input_key: char, game: &mut GameState) -> Option<PlayResult> {
		multiple_move_rogue(input_key, game);
		None
	}
}

pub struct Descend;

impl PlayerAction for Descend {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		if drop_check(game) {
			return Some(StairsDown);
		}
		return None;
	}
}