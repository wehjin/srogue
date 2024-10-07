use crate::actions::GameUpdater;
use crate::init::{Dungeon, GameState};
use crate::monster::mv_aquatars_legacy;
use crate::objects::Object;
use crate::pack::{unwear, CURSE_MESSAGE};
use crate::resources::avatar::Avatar;
use crate::systems::play_level::LevelResult;

pub struct TakeOff;

impl GameUpdater for TakeOff {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if let Some(armor_id) = game.player.armor_id() {
			if game.as_rogue_pack().check_object(armor_id, Object::is_cursed) {
				game.diary.add_entry(CURSE_MESSAGE);
			} else {
				mv_aquatars_legacy(game);
				if let Some(armor) = unwear(&mut game.player) {
					let armor_id = armor.id();
					let obj_desc = game.player.get_obj_desc(armor_id);
					let msg = format!("was wearing {}", obj_desc);
					game.diary.add_entry(&msg);
				}
				game.as_diary_mut().set_stats_changed(true);
				game.yield_turn_to_monsters();
			}
		} else {
			game.diary.add_entry("not wearing any");
		}
		None
	}
}

