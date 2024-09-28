use crate::actions::inventory::inv_rings;
use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::objects::Object;
use crate::pack::CURSE_MESSAGE;
use crate::player::rings::HandUsage;
use crate::resources::avatar::Avatar;
use crate::ring::{un_put_hand, PlayerHand};
use crate::systems::play_level::LevelResult;

pub struct RemoveRing;

impl GameUpdater for RemoveRing {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		let hand = match game.hand_usage() {
			HandUsage::None => {
				inv_rings(game);
				return None;
			}
			HandUsage::Left => PlayerHand::Left,
			HandUsage::Right => PlayerHand::Right,
			HandUsage::Both => {
				let asked = crate::ring::ask_ring_hand(game);
				match asked {
					None => { return None; }
					Some(hand) => hand
				}
			}
		};
		if game.player.ring_id(hand).is_none() {
			game.diary.add_entry("there's no ring on that hand");
			return None;
		}
		if game.player.check_ring(hand, Object::is_cursed) {
			game.diary.add_entry(CURSE_MESSAGE);
			return None;
		}
		let removed_id = un_put_hand(hand, game).expect("some removed_id");
		{
			let removed_obj = game.player.object(removed_id).expect("some removed_obj");
			let removed_desc = get_obj_desc(removed_obj, game.player.settings.fruit.to_string(), &game.player);
			let msg = format!("removed {}", removed_desc);
			game.diary.add_entry(&msg);
		}
		game.yield_turn_to_monsters();
		None
	}
}

