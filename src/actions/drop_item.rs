use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::monster::mv_aquatars;
use crate::motion::reg_move;
use crate::objects::{Object, place_at};
use crate::pack;
use crate::pack::CURSE_MESSAGE;
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::ring::un_put_hand;

pub struct DropItem;

impl PlayerAction for DropItem {
	fn commit(&self, game: &mut GameState) {
		drop_item(game);
	}
}

fn drop_item(game: &mut GameState) {
	let player_cell = game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize];
	if player_cell.has_object() || player_cell.is_stairs() || player_cell.is_any_trap() {
		game.dialog.message("there's already something there", 0);
		return;
	}
	if game.player.pack().is_empty() {
		game.dialog.message("you have nothing to drop", 0);
		return;
	}
	let ch = pack::pack_letter("drop what?", AllObjects, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0)
		}
		Some(obj_id) => {
			if game.player.check_object(obj_id, Object::is_being_wielded) {
				if game.player.check_object(obj_id, Object::is_cursed) {
					game.dialog.message(CURSE_MESSAGE, 0);
					return;
				}
				pack::unwield(&mut game.player);
			} else if game.player.check_object(obj_id, Object::is_being_worn) {
				if game.player.check_object(obj_id, Object::is_cursed) {
					game.dialog.message(CURSE_MESSAGE, 0);
					return;
				}
				mv_aquatars(game);
				pack::unwear(&mut game.player);
				game.stats_changed = true;
			} else if let Some(hand) = game.player.ring_hand(obj_id) {
				if game.player.check_ring(hand, Object::is_cursed) {
					game.dialog.message(CURSE_MESSAGE, 0);
					return;
				}
				un_put_hand(hand, game);
			}
			let place_obj = if let Some(obj) = game.player.pack_mut().object_if_mut(obj_id, |obj| obj.quantity > 1 && obj.what_is != Weapon) {
				obj.quantity -= 1;
				let mut new = obj.clone_with_new_id();
				new.quantity = 1;
				new
			} else {
				let mut obj = pack::take_from_pack(obj_id, &mut game.player.rogue.pack).expect("take from pack");
				obj.ichar = 'L';
				obj
			};
			let obj_desc = get_obj_desc(&place_obj, game.player.settings.fruit.to_string(), &game.player);
			place_at(place_obj, game.player.rogue.row, game.player.rogue.col, &mut game.level, &mut game.ground);
			game.dialog.message(&format!("dropped {}", obj_desc), 0);
			reg_move(game);
		}
	}
}
