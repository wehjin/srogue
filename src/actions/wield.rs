use crate::actions::PlayerAction;
use crate::init::{GameState, GameTurn};
use crate::pack::{CURSE_MESSAGE, do_wield, pack_letter};
use crate::prelude::object_what::ObjectWhat::{Armor, Ring};
use crate::prelude::object_what::PackFilter::Weapons;
use crate::resources::keyboard::CANCEL_CHAR;

pub struct Wield;

impl PlayerAction for Wield {
	fn update(game: &mut GameState) {
		if game.player.wields_cursed_weapon() {
			game.dialog.message(CURSE_MESSAGE, 0);
			return;
		}
		let ch = pack_letter("wield what?", Weapons, game);
		if ch == CANCEL_CHAR {
			return;
		}
		match game.player.object_with_letter_mut(ch) {
			None => {
				game.dialog.message("No such item.", 0);
				return;
			}
			Some(obj) => {
				if obj.what_is == Armor || obj.what_is == Ring {
					let item_name = if obj.what_is == Armor { "armor" } else { "rings" };
					let msg = format!("you can't wield {}", item_name);
					game.dialog.message(&msg, 0);
					return;
				}
				if obj.is_being_wielded() {
					game.dialog.message("in use", 0);
				} else {
					let obj_id = obj.id();
					let obj_desc = game.player.get_obj_desc(obj_id);
					game.player.unwield_weapon();
					game.dialog.message(&format!("wielding {}", obj_desc), 0);
					do_wield(obj_id, &mut game.player);
					game.turn = GameTurn::Monsters;
				}
			}
		}
	}
}
