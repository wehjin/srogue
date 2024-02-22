use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::motion::one_move_rogue;

pub struct MoveOnce;

impl PlayerAction for MoveOnce {
	fn update(input_key: char, game: &mut GameState) {
		one_move_rogue(input_key, true, game);
	}
}