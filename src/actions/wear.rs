use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::pack::{do_wear, pack_letter};
use crate::prelude::object_what::ObjectWhat::Armor;
use crate::prelude::object_what::PackFilter::Armors;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::PlayResult;

pub struct Wear;

impl PlayerAction for Wear {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		if game.player.armor_id().is_some() {
			game.dialog.message("your already wearing some", 0);
			return None;
		}
		let ch = pack_letter("wear what?", Armors, game);
		if ch == CANCEL_CHAR {
			return None;
		}
		match game.player.object_with_letter_mut(ch) {
			None => {
				game.dialog.message("no such item.", 0);
				return None;
			}
			Some(obj) => {
				if obj.what_is != Armor {
					game.dialog.message("you can't wear that", 0);
					return None;
				}
				obj.identified = true;
				let obj_id = obj.id();
				let obj_desc = game.player.get_obj_desc(obj_id);
				game.dialog.message(&format!("wearing {}", obj_desc), 0);
				do_wear(obj_id, &mut game.player);
				game.stats_changed = true;
				game.yield_turn_to_monsters();
			}
		}
		None
	}
}



