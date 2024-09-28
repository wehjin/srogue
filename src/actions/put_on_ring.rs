use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::objects::Object;
use crate::pack::pack_letter;
use crate::player::rings::HandUsage;
use crate::prelude::object_what::ObjectWhat::Ring;
use crate::prelude::object_what::PackFilter::Rings;
use crate::resources::avatar::Avatar;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::ring::{ask_ring_hand, ring_stats, PlayerHand};
use crate::systems::play_level::LevelResult;

pub struct PutOnRing;

impl GameUpdater for PutOnRing {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.hand_usage() == HandUsage::Both {
			game.diary.add_entry("wearing two rings already");
			return None;
		}
		let ch = pack_letter("put on what?", Rings, game);
		if ch == CANCEL_CHAR {
			return None;
		}
		match game.player.object_id_with_letter(ch) {
			None => {
				game.diary.add_entry("no such item.");
				return None;
			}
			Some(obj_id) => {
				if game.player.object_what(obj_id) != Ring {
					game.diary.add_entry("that's not a ring");
					return None;
				}
				let ring_id = obj_id;
				if game.player.check_object(ring_id, Object::is_on_either_hand) {
					game.diary.add_entry("that ring is already being worn");
					return None;
				}
				let hand = match game.player.hand_usage() {
					HandUsage::None => match ask_ring_hand(game) {
						None => {
							return None;
						}
						Some(ring_hand) => ring_hand,
					},
					HandUsage::Left => PlayerHand::Right,
					HandUsage::Right => PlayerHand::Left,
					HandUsage::Both => unreachable!("both hands checked at top of put_on_ring")
				};
				if !game.player.hand_is_free(hand) {
					game.diary.add_entry("there's already a ring on that hand");
					return None;
				}
				game.player.put_ring(ring_id, hand);
				ring_stats(true, game);
				{
					let msg = game.player.get_obj_desc(ring_id);
					game.diary.add_entry(&msg);
				}
				game.yield_turn_to_monsters()
			}
		}
		None
	}
}