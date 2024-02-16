use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::message::{CANCEL, message, print_stats};
use crate::pack::{do_wear, pack_letter};
use crate::prelude::object_what::ObjectWhat::Armor;
use crate::prelude::object_what::PackFilter::Armors;
use crate::prelude::stat_const::STAT_ARMOR;

pub struct Wear;

impl PlayerAction for Wear {
	fn commit(&self, game: &mut GameState) {
		if game.player.armor_id().is_some() {
			unsafe { message("your already wearing some", 0); }
			return;
		}
		let ch = unsafe { pack_letter("wear what?", Armors, &game.player) };
		if ch == CANCEL {
			return;
		}
		match game.player.object_with_letter_mut(ch) {
			None => unsafe {
				message("no such item.", 0);
				return;
			}
			Some(obj) => unsafe {
				if obj.what_is != Armor {
					message("you can't wear that", 0);
					return;
				}
				obj.identified = true;
				let obj_id = obj.id();
				let obj_desc = game.player.get_obj_desc(obj_id);
				message(&format!("wearing {}", obj_desc), 0);
				do_wear(obj_id, &mut game.player);
				print_stats(STAT_ARMOR, &mut game.player);
				game.commit_player_turn();
			}
		};
	}
}



