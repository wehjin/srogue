use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::systems::play_level::PlayResult;

pub struct Rest;

impl PlayerAction for Rest {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		rest(game);
		None
	}
}

fn rest(game: &mut GameState) {
	game.yield_turn_to_monsters();
}


