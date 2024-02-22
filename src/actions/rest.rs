use crate::actions::PlayerAction;
use crate::init::GameState;

pub struct Rest;

impl PlayerAction for Rest {
	fn commit(&self, game: &mut GameState) {
		rest(game);
	}
}

fn rest(game: &mut GameState) {
	game.yield_turn_to_monsters();
}


