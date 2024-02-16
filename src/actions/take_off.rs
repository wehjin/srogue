use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::message::{message, print_stats};
use crate::monster::mv_aquatars;
use crate::objects::Object;
use crate::pack::{CURSE_MESSAGE, unwear};
use crate::prelude::stat_const::STAT_ARMOR;

pub struct TakeOff;

impl PlayerAction for TakeOff {
	fn commit(&self, game: &mut GameState) {
		if let Some(armor_id) = game.player.armor_id() {
			if game.player.pack().check_object(armor_id, Object::is_cursed) {
				unsafe { message(CURSE_MESSAGE, 0); }
			} else {
				unsafe {
					mv_aquatars(&mut game.mash, &mut game.player, &mut game.level, &game.ground);
					if let Some(armor) = unwear(&mut game.player) {
						let armor_id = armor.id();
						let obj_desc = game.player.get_obj_desc(armor_id);
						let msg = format!("was wearing {}", obj_desc);
						message(&msg, 0);
					}
				}
				unsafe { print_stats(STAT_ARMOR, &mut game.player); }
				game.commit_player_turn();
			}
		} else {
			unsafe { message("not wearing any", 0); }
		}
	}
}

