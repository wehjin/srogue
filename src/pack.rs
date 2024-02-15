#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use crate::inventory::{get_obj_desc, inventory};
use crate::level::{CellKind, Level};
use crate::message::{CANCEL, check_message, get_input_line, LIST, message, print_stats, rgetchar, sound_bell};
use crate::monster::{MonsterMash, mv_aquatars};
use crate::objects::{level_objects, Object, ObjectId, ObjectPack, place_at, Title};
use crate::objects::NoteStatus::{Called, Identified, Unidentified};
use crate::player::Player;
use crate::prelude::food_kind::FRUIT;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Potion, Ring, Scroll, Wand, Weapon};
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

pub unsafe fn pick_up(row: i64, col: i64, player: &mut Player, level: &mut Level) -> PickUpResult {
	let obj_id = level_objects.find_id_at(row, col).expect("obj_id in level-objects at pick-up spot");
	if level_objects.check_object(obj_id, Object::is_used_scare_monster_scroll) {
		message("the scroll turns to dust as you pick it up", 0);
		level.dungeon[row as usize][col as usize].remove_kind(CellKind::Object);
		level_objects.remove(obj_id);
		if player.notes.scrolls[ScareMonster.to_index()].status == Unidentified {
			player.notes.scrolls[ScareMonster.to_index()].status = Identified
		}
		PickUpResult::TurnedToDust
	} else if let Some(quantity) = level_objects.try_map_object(obj_id, Object::gold_quantity) {
		player.rogue.gold += quantity;
		level.dungeon[row as usize][col as usize].remove_kind(CellKind::Object);
		let removed = level_objects.remove(obj_id).expect("remove level object");
		print_stats(STAT_GOLD, player);
		PickUpResult::AddedToGold(removed)
	} else if player.pack_weight_with_new_object(level_objects.object(obj_id))
		>= MAX_PACK_COUNT {
		message("pack too full", 1);
		PickUpResult::PackTooFull
	} else {
		level.dungeon[row as usize][col as usize].remove_kind(CellKind::Object);
		let removed_obj = take_from_pack(obj_id, &mut level_objects).expect("removed object");
		let added_id = player.combine_or_add_item_to_pack(removed_obj);
		let added_kind = {
			let obj = player.object_mut(added_id).expect("picked-up item in player's pack");
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

pub unsafe fn drop_0(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	if level.dungeon[player.rogue.row as usize][player.rogue.col as usize].is_any_kind(&[CellKind::Object, CellKind::Stairs, CellKind::Trap]) {
		message("there's already something there", 0);
		return;
	}
	if player.pack().is_empty() {
		message("you have nothing to drop", 0);
		return;
	}
	let ch = pack_letter("drop what?", AllObjects, player);
	if ch == CANCEL {
		return;
	}
	match player.object_id_with_letter(ch) {
		None => {
			message("no such item.", 0)
		}
		Some(obj_id) => {
			if player.check_object(obj_id, Object::is_being_wielded) {
				if player.check_object(obj_id, Object::is_cursed) {
					message(CURSE_MESSAGE, 0);
					return;
				}
				unwield(player);
			} else if player.check_object(obj_id, Object::is_being_worn) {
				if player.check_object(obj_id, Object::is_cursed) {
					message(CURSE_MESSAGE, 0);
					return;
				}
				mv_aquatars(mash, player, level);
				unwear(player);
				print_stats(STAT_ARMOR, player);
			} else if let Some(hand) = player.ring_hand(obj_id) {
				if player.check_ring(hand, Object::is_cursed) {
					message(CURSE_MESSAGE, 0);
					return;
				}
				un_put_hand(hand, mash, player, level);
			}
			let place_obj = if let Some(obj) = player.pack_mut().object_if_mut(obj_id, |obj| obj.quantity > 1 && obj.what_is != Weapon) {
				obj.quantity -= 1;
				let mut new = obj.clone_with_new_id();
				new.quantity = 1;
				new
			} else {
				let mut obj = take_from_pack(obj_id, &mut player.rogue.pack).expect("take from pack");
				obj.ichar = 'L';
				obj
			};
			let obj_desc = get_obj_desc(&place_obj, player.settings.fruit.to_string(), &player.notes);
			place_at(place_obj, player.rogue.row, player.rogue.col, level);
			message(&format!("dropped {}", obj_desc), 0);
			reg_move(mash, player, level);
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

pub unsafe fn wait_for_ack() {
	while rgetchar() != ' ' {}
}

pub unsafe fn pack_letter(prompt: &str, filter: PackFilter, player: &Player) -> char {
	if !mask_pack(&player.rogue.pack, filter.clone()) {
		message("nothing appropriate", 0);
		return CANCEL;
	}

	loop {
		message(prompt, 0);
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
		check_message();
		match pack_op {
			PackOp::List(filter) => {
				inventory(filter, player);
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

pub unsafe fn take_off(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	if let Some(armor_id) = player.armor_id() {
		if player.pack().check_object(armor_id, Object::is_cursed) {
			message(CURSE_MESSAGE, 0);
		} else {
			mv_aquatars(mash, player, level);
			if let Some(armor) = unwear(player) {
				let armor_id = armor.id();
				let obj_desc = player.get_obj_desc(armor_id);
				let msg = format!("was wearing {}", obj_desc);
				message(&msg, 0);
			}
			print_stats(STAT_ARMOR, player);
			reg_move(mash, player, level);
		}
	} else {
		message("not wearing any", 0);
	}
}

pub unsafe fn wear(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	if player.armor_id().is_some() {
		message("your already wearing some", 0);
		return;
	}
	let ch = pack_letter("wear what?", Armors, player);
	if ch == CANCEL {
		return;
	}
	match player.object_with_letter_mut(ch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj) => {
			if obj.what_is != Armor {
				message("you can't wear that", 0);
				return;
			}
			obj.identified = true;
			let obj_id = obj.id();
			let obj_desc = player.get_obj_desc(obj_id);
			message(&format!("wearing {}", obj_desc), 0);
			do_wear(obj_id, player);
			print_stats(STAT_ARMOR, player);
			reg_move(mash, player, level);
		}
	};
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


pub unsafe fn wield(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	if player.wields_cursed_weapon() {
		message(CURSE_MESSAGE, 0);
		return;
	}
	let ch = pack_letter("wield what?", Weapons, player);
	if ch == CANCEL {
		return;
	}
	match player.object_with_letter_mut(ch) {
		None => {
			message("No such item.", 0);
			return;
		}
		Some(obj) => {
			if obj.what_is == Armor || obj.what_is == Ring {
				let item_name = if obj.what_is == Armor { "armor" } else { "rings" };
				let msg = format!("you can't wield {}", item_name);
				message(&msg, 0);
				return;
			}
			if obj.is_being_wielded() {
				message("in use", 0);
			} else {
				let obj_id = obj.id();
				let obj_desc = player.get_obj_desc(obj_id);
				player.unwield_weapon();
				message(&format!("wielding {}", obj_desc), 0);
				do_wield(obj_id, player);
				reg_move(mash, player, level);
			}
		}
	}
}

pub fn do_wield(obj_id: ObjectId, player: &mut Player) {
	player.rogue.weapon = Some(obj_id);
	let obj = player.object_mut(obj_id).expect("wield obj in pack");
	obj.in_use_flags |= BEING_WIELDED;
}

pub fn unwield(player: &mut Player) {
	player.unwield_weapon();
}

pub unsafe fn call_it(player: &mut Player) {
	let ch = pack_letter("call what?", AnyFrom(vec![Scroll, Potion, Wand, Ring]), player);
	if ch == CANCEL {
		return;
	}
	match player.object_id_with_letter(ch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			let what = player.object_what(obj_id);
			match what {
				Scroll | Potion | Wand | Ring => {
					let kind = player.object_kind(obj_id);
					let new_name = get_input_line::<String>(
						"call it:",
						None,
						Some(player.notes.title(what, kind as usize).as_str()),
						true,
						true);
					if !new_name.is_empty() {
						let id = player.notes.note_mut(what, kind as usize);
						id.status = Called;
						id.title = Title::UserString(new_name);
					}
				}
				_ => {
					message("surely you already know what that's called", 0);
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

pub unsafe fn has_amulet(player: &Player) -> bool {
	mask_pack(&player.rogue.pack, Amulets)
}

pub unsafe fn kick_into_pack(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	if !level.dungeon[player.rogue.row as usize][player.rogue.col as usize].is_object() {
		message("nothing here", 0);
	} else {
		let settings = player.settings.clone();
		match pick_up(player.rogue.row, player.rogue.col, player, level) {
			PickUpResult::TurnedToDust => {
				reg_move(mash, player, level);
			}
			PickUpResult::AddedToGold(obj) => {
				let msg = get_obj_desc(&obj, settings.fruit.to_string(), &player.notes);
				message(&msg, 0);
			}
			PickUpResult::AddedToPack { added_id: obj_id, .. } => {
				let msg = player.get_obj_desc(obj_id);
				message(&msg, 0);
			}
			PickUpResult::PackTooFull => {
				// No message, pick_up displays a message
			}
		}
	}
}
