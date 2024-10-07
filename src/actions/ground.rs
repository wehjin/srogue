use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::motion;
use crate::motion::MoveDirection;
use crate::pack::kick_into_pack;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::LevelResult;
use rand::thread_rng;

pub struct KickIntoPack;

impl GameUpdater for KickIntoPack {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		kick_into_pack(game);
		None
	}
}

pub struct MoveOnto;

impl GameUpdater for MoveOnto {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		move_onto(game);
		None
	}
}

pub fn move_onto(game: &mut GameState) {
	let ch = motion::get_dir_or_cancel(game);
	if ch != CANCEL_CHAR {
		motion::one_move_rogue_legacy(MoveDirection::from(ch), false, game, &mut thread_rng());
	}
}
