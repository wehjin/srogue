use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::message::{check_message, message};
use crate::objects::Object;
use crate::pack::CURSE_MESSAGE;
use crate::player::rings::HandUsage;
use crate::ring::{inv_rings, PlayerHand, un_put_hand};

pub struct RemoveRing;

impl PlayerAction for RemoveRing {
	fn commit(&self, game: &mut GameState) {
		let hand = match game.player.hand_usage() {
			HandUsage::None => unsafe {
				inv_rings(&game.player);
				return;
			}
			HandUsage::Left => PlayerHand::Left,
			HandUsage::Right => PlayerHand::Right,
			HandUsage::Both => unsafe {
				let asked = crate::ring::ask_ring_hand();
				check_message();
				match asked {
					None => { return; }
					Some(hand) => hand
				}
			}
		};
		if game.player.ring_id(hand).is_none() {
			unsafe { message("there's no ring on that hand", 0); }
			return;
		}
		if game.player.check_ring(hand, Object::is_cursed) {
			unsafe { message(CURSE_MESSAGE, 0); }
			return;
		}
		let removed_id = unsafe { un_put_hand(hand, &mut game.mash, &mut game.player, &mut game.level) }.expect("some removed_id");
		unsafe {
			let removed_obj = game.player.object(removed_id).expect("some removed_obj");
			let removed_desc = get_obj_desc(removed_obj, game.player.settings.fruit.to_string(), &game.player.notes);
			let msg = format!("removed {}", removed_desc);
			message(&msg, 0);
		}
		game.commit_player_turn();
	}
}

