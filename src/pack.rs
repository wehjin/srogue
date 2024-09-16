use crate::init::GameState;
use crate::inventory::{get_obj_desc, inventory, ObjectSource};
use crate::message::sound_bell;
use crate::motion::reg_move;
use crate::objects::NoteStatus::{Identified, Unidentified};
use crate::objects::{Object, ObjectId, ObjectPack};
use crate::player::Player;
use crate::prelude::food_kind::FRUIT;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::ObjectWhat::{Food, Potion, Scroll, Weapon};
use crate::prelude::object_what::PackFilter;
use crate::prelude::object_what::PackFilter::{Amulets, Armors, Foods, Potions, Rings, Scrolls, Wands, Weapons};
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
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
		game.diary.add_entry("the scroll turns to dust as you pick it up");
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
		game.stats_changed = true;
		PickUpResult::AddedToGold(removed)
	} else if game.player.pack_weight_with_new_object(game.ground.object(obj_id))
		>= MAX_PACK_COUNT {
		game.player.interrupt_and_slurp();
		game.diary.add_entry("pack too full");
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
	// TODO: Slurp and defer interrupts like in original message function.
	loop {
		if rgetchar() == ' ' {
			break;
		}
	}
}

pub fn pack_letter(prompt: &str, filter: PackFilter, game: &mut GameState) -> char {
	if !mask_pack(&game.player.rogue.pack, filter.clone()) {
		game.diary.add_entry("nothing appropriate");
		return CANCEL_CHAR;
	}

	loop {
		diary::show_prompt(prompt, &mut game.diary);
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
		match pack_op {
			PackOp::List(filter) => {
				inventory(filter, ObjectSource::Player, game);
			}
			PackOp::Cancel => {
				return CANCEL_CHAR;
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
	false
}

pub enum PackOp {
	Cancel,
	Select(char),
	List(PackFilter),
}

pub fn get_pack_op(c: char, default_filter: PackFilter) -> Option<PackOp> {
	match c {
		CANCEL_CHAR => Some(PackOp::Cancel),
		'*' => Some(PackOp::List(default_filter)),
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
		game.diary.add_entry("nothing here");
	} else {
		let settings = game.player.settings.clone();
		match pick_up(game.player.rogue.row, game.player.rogue.col, game) {
			PickUpResult::TurnedToDust => {
				reg_move(game);
			}
			PickUpResult::AddedToGold(obj) => {
				let msg = get_obj_desc(&obj, settings.fruit.to_string(), &game.player);
				game.diary.add_entry(&msg);
			}
			PickUpResult::AddedToPack { added_id: obj_id, .. } => {
				let msg = game.player.get_obj_desc(obj_id);
				game.diary.add_entry(&msg);
			}
			PickUpResult::PackTooFull => {
				// No message, pick_up displays a message
			}
		}
	}
}
