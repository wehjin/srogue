

use crate::init::GameState;
use crate::inventory::{get_obj_desc, inventory};
use crate::message::{CANCEL, get_input_line, LIST, print_stats, rgetchar, sound_bell};
use crate::monster::mv_aquatars;
use crate::objects::{Object, ObjectId, ObjectPack, place_at, Title};
use crate::objects::NoteStatus::{Called, Identified, Unidentified};
use crate::player::Player;
use crate::prelude::food_kind::FRUIT;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::ObjectWhat::{Food, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter;
use crate::prelude::object_what::PackFilter::{AllObjects, Amulets, AnyFrom, Armors, Foods, Potions, Rings, Scrolls, Wands, Weapons};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_GOLD};
use crate::r#move::reg_move;
use crate::ring::un_put_hand;
use crate::scrolls::ScrollKind::ScareMonster;
use crate::weapons::kind::WeaponKind;

pub const CURSE_MESSAGE: &'static str = "you can't, it appears to be cursed";
pub const MAX_PACK_COUNT: usize = 24;

pub fn take_from_pack(obj_id: ObjectId, pack: &mut ObjectPack) -> Option<Object> {
	pack.remove(obj_id)
}

pub enum PickUpResult {
	TurnedToDust,
	AddedToGold(Object),
	AddedToPack { added_id: ObjectId, added_kind: usize },
	PackTooFull,
}

pub fn pick_up(row: i64, col: i64, game: &mut GameState) -> PickUpResult {
	let obj_id = game.ground.find_id_at(row, col).expect("obj_id in level-objects at pick-up spot");
	if game.ground.check_object(obj_id, Object::is_used_scare_monster_scroll) {
		game.dialog.message("the scroll turns to dust as you pick it up", 0);
		game.level.dungeon[row as usize][col as usize].clear_object();
		game.ground.remove(obj_id);
		if game.player.notes.scrolls[ScareMonster.to_index()].status == Unidentified {
			game.player.notes.scrolls[ScareMonster.to_index()].status = Identified
		}
		PickUpResult::TurnedToDust
	} else if let Some(quantity) = game.ground.try_map_object(obj_id, Object::gold_quantity) {
		game.player.rogue.gold += quantity;
		game.level.dungeon[row as usize][col as usize].clear_object();
		let removed = game.ground.remove(obj_id).expect("remove level object");
		print_stats(STAT_GOLD, &mut game.player);
		PickUpResult::AddedToGold(removed)
	} else if game.player.pack_weight_with_new_object(game.ground.object(obj_id))
		>= MAX_PACK_COUNT {
		game.player.interrupt_and_slurp();
		game.dialog.message("pack too full", 1);
		PickUpResult::PackTooFull
	} else {
		game.level.dungeon[row as usize][col as usize].clear_object();
		let removed_obj = take_from_pack(obj_id, &mut game.ground).expect("removed object");
		let added_id = game.player.combine_or_add_item_to_pack(removed_obj);
		let added_kind = {
			let obj = game.player.object_mut(added_id).expect("picked-up item in player's pack");
			obj.picked_up = 1;
			obj.which_kind
		};
		PickUpResult::AddedToPack { added_id, added_kind: added_kind as usize }
	}
}

impl Object {
	pub fn is_scare_monster_scroll(&self) -> bool {
		self.what_is == Scroll && ScareMonster.is_kind(self.which_kind)
	}
	pub fn is_used_scare_monster_scroll(&self) -> bool {
		self.is_scare_monster_scroll() && self.picked_up != 0
	}
}

pub fn drop_0(game: &mut GameState) {
	let player_cell = game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize];
	if player_cell.has_object() || player_cell.is_stairs() || player_cell.is_any_trap() {
		game.dialog.message("there's already something there", 0);
		return;
	}
	if game.player.pack().is_empty() {
		game.dialog.message("you have nothing to drop", 0);
		return;
	}
	let ch = pack_letter("drop what?", AllObjects, game);
	if ch == CANCEL {
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
				unwield(&mut game.player);
			} else if game.player.check_object(obj_id, Object::is_being_worn) {
				if game.player.check_object(obj_id, Object::is_cursed) {
					game.dialog.message(CURSE_MESSAGE, 0);
					return;
				}
				mv_aquatars(game);
				unwear(&mut game.player);
				print_stats(STAT_ARMOR, &mut game.player);
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
				let mut obj = take_from_pack(obj_id, &mut game.player.rogue.pack).expect("take from pack");
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

pub fn check_duplicate(obj: &Object, pack: &mut ObjectPack) -> Option<ObjectId> {
	let combinable = match obj.what_is {
		Weapon | Food | Scroll | Potion => true,
		_ => false,
	};
	if !combinable {
		return None;
	}
	if obj.what_is == Food && obj.which_kind == FRUIT {
		return None;
	}
	if let Some(found) = pack.find_object_mut(|pack_obj| obj.can_join_existing_pack_object(pack_obj)) {
		found.quantity += obj.quantity;
		Some(found.id())
	} else {
		None
	}
}

pub fn next_avail_ichar(player: &Player) -> char {
	let mut used = [false; 26];
	for obj in player.rogue.pack.objects() {
		let letter_index = (obj.ichar as u8 - 'a' as u8) as usize;
		used[letter_index] = true;
	}
	if let Some(unused) = used.into_iter().position(|used| used == false) {
		(unused as u8 + 'a' as u8) as char
	} else { '?' }
}

pub fn wait_for_ack() {
	loop {
		if rgetchar() == ' ' {
			break;
		}
	}
}

pub fn pack_letter(prompt: &str, filter: PackFilter, game: &mut GameState) -> char {
	if !mask_pack(&game.player.rogue.pack, filter.clone()) {
		game.dialog.message("nothing appropriate", 0);
		return CANCEL;
	}

	loop {
		game.dialog.message(prompt, 0);
		let pack_op = {
			let mut pack_op;
			loop {
				let ch = rgetchar() as u8 as char;
				pack_op = get_pack_op(ch, filter.clone());
				if pack_op.is_none() {
					sound_bell();
				} else {
					break;
				}
			}
			pack_op.expect("some pack operation")
		};
		game.dialog.clear_message();
		match pack_op {
			PackOp::List(filter) => {
				inventory(filter, game);
			}
			PackOp::Cancel => {
				return CANCEL;
			}
			PackOp::Select(letter) => {
				return letter;
			}
		}
	}
}


pub fn do_wear(obj_id: ObjectId, player: &mut Player) {
	player.rogue.armor = Some(obj_id);
	let obj = player.object_mut(obj_id).expect("wear obj in pack");
	obj.in_use_flags |= BEING_WORN;
	obj.identified = true;
}

pub fn unwear(player: &mut Player) -> Option<&Object> {
	player.unwear_armor()
}


pub fn do_wield(obj_id: ObjectId, player: &mut Player) {
	player.rogue.weapon = Some(obj_id);
	let obj = player.object_mut(obj_id).expect("wield obj in pack");
	obj.in_use_flags |= BEING_WIELDED;
}

pub fn unwield(player: &mut Player) {
	player.unwield_weapon();
}

pub fn call_it(game: &mut GameState) {
	let ch = pack_letter("call what?", AnyFrom(vec![Scroll, Potion, Wand, Ring]), game);
	if ch == CANCEL {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			let what = game.player.object_what(obj_id);
			match what {
				Scroll | Potion | Wand | Ring => {
					let kind = game.player.object_kind(obj_id);
					let new_name = get_input_line::<String>(
						"call it:",
						None,
						Some(game.player.notes.title(what, kind as usize).as_str()),
						true,
						true, game);
					if !new_name.is_empty() {
						let id = game.player.notes.note_mut(what, kind as usize);
						id.status = Called;
						id.title = Title::UserString(new_name);
					}
				}
				_ => {
					game.dialog.message("surely you already know what that's called", 0);
					return;
				}
			}
		}
	}
}

impl Object {
	fn can_combine_weapon(old_kind: WeaponKind, old_quiver: i16, new_obj: &Object) -> bool {
		if let Some(new_kind) = new_obj.weapon_kind() {
			new_kind == old_kind
				&& new_kind.is_arrow_or_throwing_weapon()
				&& new_obj.quiver == old_quiver
		} else {
			false
		}
	}
	pub fn pack_weight_with_new_obj(&self, new_obj: Option<&Object>) -> usize {
		if let Some(weapon_kind) = self.weapon_kind() {
			if let Some(new_obj) = new_obj {
				if Self::can_combine_weapon(weapon_kind, self.quantity, new_obj) {
					// The original C code returns 0 for this case.  This only makes
					// sense if we assume this object will eventually be moved into
					// the new object and the new object will be counted separately.
					0
				} else {
					// Count this object because it will not be merged into the new object.
					1
				}
			} else {
				1
			}
		} else {
			self.quantity as usize
		}
	}
}

impl Player {
	pub fn pack_weight_with_new_object(&self, new_obj: Option<&Object>) -> usize {
		let mut weight = 0;
		for obj in self.rogue.pack.objects() {
			weight += obj.pack_weight_with_new_obj(new_obj);
		}
		// Note: the original C code forgets to include weight of the new object.
		if let Some(new_obj) = new_obj {
			weight += new_obj.pack_weight_with_new_obj(None);
		}
		weight
	}
}

pub fn mask_pack(pack: &ObjectPack, mask: PackFilter) -> bool {
	for obj in pack.objects() {
		if mask.includes(obj.what_is) {
			return true;
		}
	}
	return false;
}

pub enum PackOp {
	Cancel,
	Select(char),
	List(PackFilter),
}

pub fn get_pack_op(c: char, default_filter: PackFilter) -> Option<PackOp> {
	match c {
		LIST => Some(PackOp::List(default_filter)),
		CANCEL => Some(PackOp::Cancel),
		'?' => Some(PackOp::List(Scrolls)),
		'!' => Some(PackOp::List(Potions)),
		':' => Some(PackOp::List(Foods)),
		')' => Some(PackOp::List(Weapons)),
		']' => Some(PackOp::List(Armors)),
		'/' => Some(PackOp::List(Wands)),
		'=' => Some(PackOp::List(Rings)),
		',' => Some(PackOp::List(Amulets)),
		'a'..='z' => Some(PackOp::Select(c)),
		_ => None
	}
}

pub fn has_amulet(player: &Player) -> bool {
	mask_pack(&player.rogue.pack, Amulets)
}

pub fn kick_into_pack(game: &mut GameState) {
	if !game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].has_object() {
		game.dialog.message("nothing here", 0);
	} else {
		let settings = game.player.settings.clone();
		match pick_up(game.player.rogue.row, game.player.rogue.col, game) {
			PickUpResult::TurnedToDust => {
				reg_move(game);
			}
			PickUpResult::AddedToGold(obj) => {
				let msg = get_obj_desc(&obj, settings.fruit.to_string(), &game.player);
				game.dialog.message(&msg, 0);
			}
			PickUpResult::AddedToPack { added_id: obj_id, .. } => {
				let msg = game.player.get_obj_desc(obj_id);
				game.dialog.message(&msg, 0);
			}
			PickUpResult::PackTooFull => {
				// No message, pick_up displays a message
			}
		}
	}
}
