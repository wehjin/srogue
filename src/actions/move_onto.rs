use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::motion;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::PlayResult;

pub struct MoveOnto;

impl PlayerAction for MoveOnto {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		move_onto(game);
		None
	}
}

pub fn move_onto(game: &mut GameState) {
	let ch = motion::get_dir_or_cancel(game);
	game.dialog.clear_message();
	if ch != CANCEL_CHAR {
		motion::one_move_rogue(ch, false, game);
	}
}
