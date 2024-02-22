use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::inventory::inventory;
use crate::prelude::object_what::PackFilter::AllObjects;

pub struct Inventory;

impl PlayerAction for Inventory {
	fn update(game: &mut GameState) {
		inventory(AllObjects, game);
	}
}

