use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::pack::pack_letter;
use crate::potions::quaff::quaff_potion;
use crate::prelude::object_what::ObjectWhat::Potion;
use crate::prelude::object_what::PackFilter::Potions;
use crate::resources::keyboard::CANCEL_CHAR;

pub struct Quaff;

impl PlayerAction for Quaff {
	fn update(game: &mut GameState) {
		quaff(game)
	}
}

pub fn quaff(game: &mut GameState) {
	let ch = pack_letter("quaff what?", Potions, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			match game.player.expect_object(obj_id).potion_kind() {
				None => {
					game.dialog.message("you can't drink that", 0);
					return;
				}
				Some(potion_kind) => {
					quaff_potion(potion_kind, game);
					game.stats_changed = true;
					game.player.notes.identify_if_un_called(Potion, potion_kind.to_index());
					crate::r#use::vanish(obj_id, true, game);
				}
			}
		}
	}
}

pub const STRANGE_FEELING: &'static str = "you have a strange feeling for a moment, then it passes";
