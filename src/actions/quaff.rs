use crate::actions::GameUpdater;
use crate::init::{Dungeon, GameState};
use crate::pack::pack_letter;
use crate::potions::quaff::quaff_potion;
use crate::prelude::object_what::ObjectWhat::Potion;
use crate::prelude::object_what::PackFilter::Potions;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::LevelResult;

pub struct Quaff;

impl GameUpdater for Quaff {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		quaff(game);
		None
	}
}

pub fn quaff(game: &mut GameState) {
	let ch = pack_letter("quaff what?", Potions, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.diary.add_entry("no such item.");
			return;
		}
		Some(obj_id) => {
			match game.player.expect_object(obj_id).potion_kind() {
				None => {
					game.diary.add_entry("you can't drink that");
					return;
				}
				Some(potion_kind) => {
					quaff_potion(potion_kind, game);
					game.as_diary_mut().set_stats_changed(true);
					game.player.notes.identify_if_un_called(Potion, potion_kind.to_index());
					crate::r#use::vanish(obj_id, true, game);
				}
			}
		}
	}
}

pub const STRANGE_FEELING: &'static str = "you have a strange feeling for a moment, then it passes";
