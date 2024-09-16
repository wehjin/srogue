use crate::actions::GameUpdater;
use crate::init::{GameState, GameTurn};
use crate::pack::{do_wield, pack_letter, CURSE_MESSAGE};
use crate::prelude::object_what::ObjectWhat::{Armor, Ring};
use crate::prelude::object_what::PackFilter::Weapons;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::LevelResult;


pub struct Wield;

impl GameUpdater for Wield {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wields_cursed_weapon() {
			game.diary.add_entry(CURSE_MESSAGE);
			return None;
		}
		let ch = pack_letter("wield what?", Weapons, game);
		if ch == CANCEL_CHAR {
			return None;
		}
		match game.player.object_with_letter_mut(ch) {
			None => {
				game.diary.add_entry("No such item.");
				return None;
			}
			Some(obj) => {
				if obj.what_is == Armor || obj.what_is == Ring {
					let item_name = if obj.what_is == Armor { "armor" } else { "rings" };
					let msg = format!("you can't wield {}", item_name);
					game.diary.add_entry(&msg);
					return None;
				}
				if obj.is_being_wielded() {
					game.diary.add_entry("in use");
				} else {
					let obj_id = obj.id();
					let obj_desc = game.player.get_obj_desc(obj_id);
					game.player.unwield_weapon();
					game.diary.add_entry(&format!("wielding {}", obj_desc));
					do_wield(obj_id, &mut game.player);
					game.turn = GameTurn::Monsters;
				}
			}
		}
		None
	}
}
