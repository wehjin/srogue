use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::monster::mv_aquatars;
use crate::objects::Object;
use crate::pack::{CURSE_MESSAGE, unwear};

pub struct TakeOff;

impl PlayerAction for TakeOff {
	fn commit(&self, game: &mut GameState) {
		if let Some(armor_id) = game.player.armor_id() {
			if game.player.pack().check_object(armor_id, Object::is_cursed) {
				game.dialog.message(CURSE_MESSAGE, 0);
			} else {
				mv_aquatars(game);
				if let Some(armor) = unwear(&mut game.player) {
					let armor_id = armor.id();
					let obj_desc = game.player.get_obj_desc(armor_id);
					let msg = format!("was wearing {}", obj_desc);
					game.dialog.message(&msg, 0);
				}
				game.stats_changed = true;
				game.yield_turn_to_monsters();
			}
		} else {
			game.dialog.message("not wearing any", 0);
		}
	}
}

