use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::monster::mv_aquatars;
use crate::motion::{reg_move, MoveDirection};
use crate::objects::{place_at, Object};
use crate::pack::{kick_into_pack, CURSE_MESSAGE};
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::ring::un_put_hand;
use crate::systems::play_level::LevelResult;
use crate::{motion, pack};

pub struct KickIntoPack;

impl GameUpdater for KickIntoPack {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		kick_into_pack(game);
		None
	}
}

pub struct DropItem;

impl GameUpdater for DropItem {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		drop_item(game);
		None
	}
}

fn drop_item(game: &mut GameState) {
	let player_cell = game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize];
	if player_cell.has_object() || player_cell.is_stairs() || player_cell.is_any_trap() {
		game.diary.add_entry("there's already something there");
		return;
	}
	if game.player.pack().is_empty() {
		game.diary.add_entry("you have nothing to drop");
		return;
	}
	let ch = pack::pack_letter("drop what?", AllObjects, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.diary.add_entry("no such item.")
		}
		Some(obj_id) => {
			if game.player.check_object(obj_id, Object::is_being_wielded) {
				if game.player.check_object(obj_id, Object::is_cursed) {
					game.diary.add_entry(CURSE_MESSAGE);
					return;
				}
				pack::unwield(&mut game.player);
			} else if game.player.check_object(obj_id, Object::is_being_worn) {
				if game.player.check_object(obj_id, Object::is_cursed) {
					game.diary.add_entry(CURSE_MESSAGE);
					return;
				}
				mv_aquatars(game);
				pack::unwear(&mut game.player);
				game.stats_changed = true;
			} else if let Some(hand) = game.player.ring_hand(obj_id) {
				if game.player.check_ring(hand, Object::is_cursed) {
					game.diary.add_entry(CURSE_MESSAGE);
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
			game.diary.add_entry(&format!("dropped {}", obj_desc));
			reg_move(game);
		}
	}
}

pub struct MoveOnto;

impl GameUpdater for MoveOnto {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		move_onto(game);
		None
	}
}

pub fn move_onto(game: &mut GameState) {
	let ch = motion::get_dir_or_cancel(game);
	if ch != CANCEL_CHAR {
		motion::one_move_rogue(MoveDirection::from(ch), false, game);
	}
}
