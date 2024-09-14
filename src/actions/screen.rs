use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::systems::play_level::LevelResult;

pub struct ReMessage;

impl PlayerAction for ReMessage {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		game.dialog.re_message();
		None
	}
}