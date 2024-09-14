use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::systems::play_level::LevelResult;

pub struct ReMessage;

impl GameUpdater for ReMessage {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		game.dialog.re_message();
		None
	}
}