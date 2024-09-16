use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::pack::{do_wear, pack_letter};
use crate::prelude::object_what::ObjectWhat::Armor;
use crate::prelude::object_what::PackFilter::Armors;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::LevelResult;

pub struct Wear;

impl GameUpdater for Wear {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.armor_id().is_some() {
			game.diary.add_entry("your already wearing some");
			return None;
		}
		let ch = pack_letter("wear what?", Armors, game);
		if ch == CANCEL_CHAR {
			return None;
		}
		match game.player.object_with_letter_mut(ch) {
			None => {
				game.diary.add_entry("no such item.");
				return None;
			}
			Some(obj) => {
				if obj.what_is != Armor {
					game.diary.add_entry("you can't wear that");
					return None;
				}
				obj.identified = true;
				let obj_id = obj.id();
				let obj_desc = game.player.get_obj_desc(obj_id);
				game.diary.add_entry(&format!("wearing {}", obj_desc));
				do_wear(obj_id, &mut game.player);
				game.stats_changed = true;
				game.yield_turn_to_monsters();
			}
		}
		None
	}
}



