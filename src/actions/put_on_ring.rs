use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::objects::Object;
use crate::pack::pack_letter;
use crate::player::rings::HandUsage;
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::prelude::object_what::PackFilter::Rings;
use crate::ring::{ask_ring_hand, PlayerHand, ring_stats};

pub struct PutOnRing;

impl PlayerAction for PutOnRing {
	fn commit(&self, game: &mut GameState) {
		if game.player.hand_usage() == HandUsage::Both {
			game.dialog.message("wearing two rings already", 0);
			return;
		}
		let ch = pack_letter("put on what?", Rings, game);
		if ch == CANCEL_CHAR {
			return;
		}
		match game.player.object_id_with_letter(ch) {
			None => {
				game.dialog.message("no such item.", 0);
				return;
			}
			Some(obj_id) => {
				if game.player.object_what(obj_id) != Ring {
					game.dialog.message("that's not a ring", 0);
					return;
				}
				let ring_id = obj_id;
				if game.player.check_object(ring_id, Object::is_on_either_hand) {
					game.dialog.message("that ring is already being worn", 0);
					return;
				}
				let hand = match game.player.hand_usage() {
					HandUsage::None => match ask_ring_hand(game) {
						None => {
							game.dialog.clear_message();
							return;
						}
						Some(ring_hand) => ring_hand,
					},
					HandUsage::Left => PlayerHand::Right,
					HandUsage::Right => PlayerHand::Left,
					HandUsage::Both => unreachable!("both hands checked at top of put_on_ring")
				};
				if !game.player.hand_is_free(hand) {
					game.dialog.clear_message();
					game.dialog.message("there's already a ring on that hand", 0);
					return;
				}
				game.player.put_ring(ring_id, hand);
				ring_stats(true, game);
				game.dialog.clear_message();
				{
					let msg = game.player.get_obj_desc(ring_id);
					game.dialog.message(&msg, 0);
				}
				game.commit_player_turn()
			}
		}
	}
}