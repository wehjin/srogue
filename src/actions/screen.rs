use crate::actions::PlayerAction;
use crate::init::GameState;

pub struct ReMessage;

impl PlayerAction for ReMessage {
	fn update(_input_key: char, game: &mut GameState) {
		game.dialog.re_message()
	}
}