use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::inventory::inventory;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::systems::play_level::PlayResult;

pub struct Inventory;

impl PlayerAction for Inventory {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		inventory(AllObjects, game);
		None
	}
}

