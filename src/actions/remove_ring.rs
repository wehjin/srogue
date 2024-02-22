use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::objects::Object;
use crate::pack::CURSE_MESSAGE;
use crate::player::rings::HandUsage;
use crate::ring::{inv_rings, PlayerHand, un_put_hand};

pub struct RemoveRing;

impl PlayerAction for RemoveRing {
	fn update(game: &mut GameState) {
		let hand = match game.player.hand_usage() {
			HandUsage::None => {
				inv_rings(game);
				return;
			}
			HandUsage::Left => PlayerHand::Left,
			HandUsage::Right => PlayerHand::Right,
			HandUsage::Both => {
				let asked = crate::ring::ask_ring_hand(game);
				game.dialog.clear_message();
				match asked {
					None => { return; }
					Some(hand) => hand
				}
			}
		};
		if game.player.ring_id(hand).is_none() {
			game.dialog.message("there's no ring on that hand", 0);
			return;
		}
		if game.player.check_ring(hand, Object::is_cursed) {
			game.dialog.message(CURSE_MESSAGE, 0);
			return;
		}
		let removed_id = un_put_hand(hand, game).expect("some removed_id");
		{
			let removed_obj = game.player.object(removed_id).expect("some removed_obj");
			let removed_desc = get_obj_desc(removed_obj, game.player.settings.fruit.to_string(), &game.player);
			let msg = format!("removed {}", removed_desc);
			game.dialog.message(&msg, 0);
		}
		game.yield_turn_to_monsters();
	}
}

