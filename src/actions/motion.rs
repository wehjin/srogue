use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::motion::{multiple_move_rogue, one_move_rogue};

pub struct MoveOnce;

impl PlayerAction for MoveOnce {
	fn update(input_key: char, game: &mut GameState) {
		one_move_rogue(input_key, true, game);
	}
}

pub struct MoveMultiple;

impl PlayerAction for MoveMultiple {
	fn update(input_key: char, game: &mut GameState) {
		multiple_move_rogue(input_key, game)
	}
}