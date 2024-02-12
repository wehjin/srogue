#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

mod object_id;
mod object_pack;
mod potions;
mod scrolls;
mod weapons;
mod armors;
mod kinds;

use std::clone::Clone;
use std::string::ToString;
use ncurses::{chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};
use ObjectWhat::{Armor, Potion, Scroll, Weapon};
use crate::level::constants::MAX_ROOM;
use crate::odds::GOLD_PERCENT;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::item_usage::{BEING_USED, BEING_WIELDED, BEING_WORN, NOT_USED, ON_EITHER_HAND, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::ObjectWhat::{Amulet, Food, Gold, Ring, Wand};
use crate::prelude::potion_kind::PotionKind::{Blindness, Confusion, DetectMonster, DetectObjects, ExtraHealing, Hallucination, Healing, IncreaseStrength, Levitation, Poison, RaiseLevel, RestoreStrength, SeeInvisible};
use crate::prelude::potion_kind::{PotionKind, POTIONS};
use crate::ring::constants::RINGS;
use crate::scrolls::ScrollKind::{AggravateMonster, CreateMonster, EnchArmor, EnchWeapon, HoldMonster, Identify, MagicMapping, ProtectArmor, RemoveCurse, ScareMonster, Sleep, Teleport};
use crate::scrolls::constants::SCROLLS;
pub use object_id::*;
use crate::settings::fruit;
pub use object_pack::*;
use crate::armors::constants::{ARMORS, PLATE, SPLINT};
use crate::hit::DamageStat;
use crate::inventory::{get_obj_desc, get_title, IS_WOOD};
use crate::level::{CellKind, Level};
use crate::message::{CANCEL, check_message, get_input_line, message, rgetchar, sound_bell};
use crate::monster::{MASH, party_monsters};
use crate::pack::MAX_PACK_COUNT;
use crate::player::Player;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::ring::gr_ring;
use crate::room::{get_mask_char, gr_room, gr_row_col, party_objects, RoomType};
use crate::weapons::constants::{ARROW, DAGGER, DART, SHURIKEN, WEAPONS};
use crate::zap::constants::{CANCELLATION, MAGIC_MISSILE, WANDS};


#[derive(Clone, Serialize, Deserialize)]
pub struct id {
	pub title: Option<String>,
	pub id_status: IdStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum IdStatus {
	Unidentified,
	Identified,
	Called,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct
obj {
	id: ObjectId,
	pub quantity: i16,
	pub ichar: char,
	pub kill_exp: isize,
	pub is_protected: i16,
	pub is_cursed: i16,
	pub class: isize,
	pub identified: bool,
	pub which_kind: u16,
	pub o_row: i64,
	pub o_col: i64,
	pub o: i16,
	pub row: i64,
	pub col: i64,
	pub d_enchant: isize,
	pub quiver: i16,
	pub trow: i64,
	pub tcol: i64,
	pub hit_enchant: i16,
	pub what_is: ObjectWhat,
	pub picked_up: i16,
	pub in_use_flags: u16,
}

pub fn empty_obj() -> obj {
	obj {
		id: ObjectId::random(),
		quantity: 0,
		ichar: '\x00',
		kill_exp: 0,
		is_protected: 0,
		is_cursed: 0,
		class: 0,
		identified: false,
		which_kind: 0,
		o_row: 0,
		o_col: 0,
		o: 0,
		row: 0,
		col: 0,
		d_enchant: 0,
		quiver: 0,
		trow: 0,
		tcol: 0,
		hit_enchant: 0,
		what_is: ObjectWhat::None,
		picked_up: 0,
		in_use_flags: 0,
	}
}

impl obj {
	pub fn clone_with_new_id(&self) -> Self {
		let mut new = self.clone();
		new.id = ObjectId::random();
		new
	}
	pub unsafe fn to_name_with_new_quantity(&self, quantity: i16) -> String {
		let mut temp_obj = self.clone();
		temp_obj.quantity = quantity;
		name_of(&temp_obj)
	}
	pub fn can_join_existing_pack_object(&self, existing_pack_obj: &Self) -> bool {
		self.is_same_kind(existing_pack_obj) &&
			(!self.is_weapon() || (self.is_arrow_or_throwing_weapon() && self.quiver == existing_pack_obj.quiver))
	}
	pub fn is_same_kind(&self, other: &Self) -> bool { self.what_is == other.what_is && self.which_kind == other.which_kind }
	pub fn is_cursed(&self) -> bool { self.is_cursed != 0 }
	pub fn is_being_used(&self) -> bool { self.in_use_flags & BEING_USED != 0 }
	pub fn is_being_wielded(&self) -> bool { self.in_use_flags & BEING_WIELDED != 0 }
	pub fn is_being_worn(&self) -> bool { self.in_use_flags & BEING_WORN != 0 }
	pub fn is_on_either_hand(&self) -> bool { self.in_use_flags & ON_EITHER_HAND != 0 }
	pub fn is_on_left_hand(&self) -> bool { self.in_use_flags & ON_LEFT_HAND != 0 }
	pub fn is_on_right_hand(&self) -> bool { self.in_use_flags & ON_RIGHT_HAND != 0 }
	pub fn is_at(&self, row: i64, col: i64) -> bool {
		self.row == row && self.col == col
	}
	pub fn gold_quantity(&self) -> Option<usize> {
		if self.what_is == Gold {
			Some(self.quantity as usize)
		} else {
			None
		}
	}
	pub fn base_damage(&self) -> DamageStat {
		if let Some(kind) = self.weapon_kind() {
			kind.damage()
		} else {
			DamageStat { hits: 1, damage: 1 }
		}
	}
	pub fn enhanced_damage(&self) -> DamageStat {
		let DamageStat { hits, damage } = self.base_damage();
		let hits = hits + self.hit_enchant as usize;
		let damage = damage + self.d_enchant as usize;
		DamageStat { hits, damage }
	}
	pub fn id(&self) -> ObjectId { self.id }
}

pub type object = obj;

pub static mut level_objects: ObjectPack = ObjectPack::new();
pub static mut foods: i16 = 0;
pub static mut id_potions: [id; POTIONS] = {
	[
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_scrolls: [id; SCROLLS] = {
	[
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_weapons: [id; WEAPONS] = {
	[
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_armors: [id; ARMORS] = {
	[
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_wands: [id; WANDS] = {
	[
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_rings: [id; RINGS] = {
	[
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let init = id {
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};

pub unsafe fn put_objects(player: &mut Player, level: &mut Level) {
	if player.cur_depth < player.max_depth {
		return;
	}

	let mut n = if coin_toss() { get_rand(2, 4) } else { get_rand(3, 5) };
	while rand_percent(33) {
		n += 1;
	}
	if player.cur_depth == player.party_counter {
		make_party(player.cur_depth, level);
		player.party_counter = next_party(player.cur_depth);
	}
	for _i in 0..n {
		let obj = gr_object(player.cur_depth);
		rand_place(obj, player, level);
	}
	put_gold(player.cur_depth, level);
}

pub unsafe fn put_gold(level_depth: usize, level: &mut Level) {
	for i in 0..MAX_ROOM {
		let is_maze = level.rooms[i].room_type == RoomType::Maze;
		let is_room = level.rooms[i].room_type == RoomType::Room;
		if !(is_room || is_maze) {
			continue;
		}
		if is_maze || rand_percent(GOLD_PERCENT) {
			for _j in 0..50 {
				let row = get_rand(level.rooms[i].top_row + 1, level.rooms[i].bottom_row - 1);
				let col = get_rand(level.rooms[i].left_col + 1, level.rooms[i].right_col - 1);
				if level.dungeon[row as usize][col as usize].is_only_kind(CellKind::Floor)
					|| level.dungeon[row as usize][col as usize].is_only_kind(CellKind::Tunnel) {
					plant_gold(row, col, is_maze, level_depth, level);
					break;
				}
			}
		}
	}
}

pub unsafe fn plant_gold(row: i64, col: i64, is_maze: bool, cur_level: usize, level: &mut Level) {
	let mut obj = alloc_object();
	obj.row = row;
	obj.col = col;
	obj.what_is = Gold;
	obj.quantity = get_rand((2 * cur_level) as i16, (16 * cur_level) as i16);
	if is_maze {
		obj.quantity += obj.quantity / 2;
	}
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Object);
	level_objects.add(obj);
}


pub unsafe fn place_at(mut obj: object, row: i64, col: i64, level: &mut Level) {
	obj.row = row;
	obj.col = col;
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Object);
	level_objects.add(obj);
}

impl Player {
	pub fn object_id_with_letter(&self, ch: char) -> Option<ObjectId> {
		self.obj_id_if(|obj| obj.ichar == ch)
	}
}

impl Player {
	pub fn object_what(&self, obj_id: ObjectId) -> ObjectWhat {
		if let Some(obj) = self.object(obj_id) { obj.what_is } else { ObjectWhat::None }
	}
	pub fn object_kind(&self, obj_id: ObjectId) -> u16 {
		if let Some(obj) = self.object(obj_id) { obj.which_kind } else { 0 }
	}
	pub fn check_object(&self, obj_id: ObjectId, f: impl Fn(&obj) -> bool) -> bool {
		self.pack().check_object(obj_id, f)
	}
	pub fn obj_id_if(&self, f: impl Fn(&obj) -> bool) -> Option<ObjectId> {
		self.pack().find_id(f)
	}
	pub fn pack(&self) -> &ObjectPack { &self.rogue.pack }

	pub fn object_with_letter(&self, ch: char) -> Option<&obj> {
		self.find_pack_obj(|obj| obj.ichar == ch)
	}
	pub fn object_with_letter_mut(&mut self, ch: char) -> Option<&mut obj> {
		self.find_pack_obj_mut(|obj| obj.ichar == ch)
	}
}

pub unsafe fn name_of(obj: &object) -> String {
	match obj.what_is {
		Armor => "armor ".to_string(),
		Weapon => match obj.which_kind {
			DART => if obj.quantity > 1 { "darts " } else { "dart " },
			ARROW => if obj.quantity > 1 { "arrows " } else { "arrow " },
			DAGGER => if obj.quantity > 1 { "daggers " } else { "dagger " },
			SHURIKEN => if obj.quantity > 1 { "shurikens " } else { "shuriken " },
			_ => get_title(obj),
		}.to_string(),
		Scroll => if obj.quantity > 1 { "scrolls " } else { "scroll " }.to_string(),
		Potion => if obj.quantity > 1 { "potions " } else { "potion " }.to_string(),
		Food => if obj.which_kind == RATION { "food ".to_string() } else { fruit() }
		Wand => if IS_WOOD[obj.which_kind as usize] { "staff " } else { "wand " }.to_string(),
		Ring => "ring ".to_string(),
		Amulet => "amulet ".to_string(),
		_ => "unknown ".to_string(),
	}
}

pub unsafe fn gr_object(cur_level: usize) -> object {
	let mut obj = alloc_object();
	if foods < (cur_level / 2) as i16 {
		obj.what_is = Food;
		foods += 1;
	} else {
		obj.what_is = gr_what_is();
	}
	match obj.what_is {
		Scroll => {
			gr_scroll(&mut obj);
		}
		Potion => {
			gr_potion(&mut obj);
		}
		Weapon => {
			gr_weapon(&mut obj, true);
		}
		Armor => {
			gr_armor(&mut obj);
		}
		Wand => {
			gr_wand(&mut obj);
		}
		Food => {
			get_food(&mut obj, false);
		}
		Ring => {
			gr_ring(&mut obj, true);
		}
		_ => {}
	}
	obj
}


pub unsafe fn gr_what_is() -> ObjectWhat {
	let percent = get_rand(1, 91);
	if percent <= 30 {
		Scroll
	} else if percent <= 60 {
		Potion
	} else if percent <= 64 {
		Wand
	} else if percent <= 74 {
		Weapon
	} else if percent <= 83 {
		Armor
	} else if percent <= 88 {
		Food
	} else {
		Ring
	}
}


pub fn gr_scroll(obj: &mut obj) {
	let percent = get_rand(0, 85);
	(*obj).what_is = Scroll;

	let kind = if percent <= 5 {
		ProtectArmor
	} else if percent <= 11 {
		HoldMonster
	} else if percent <= 20 {
		CreateMonster
	} else if percent <= 35 {
		Identify
	} else if percent <= 43 {
		Teleport
	} else if percent <= 50 {
		Sleep
	} else if percent <= 55 {
		ScareMonster
	} else if percent <= 64 {
		RemoveCurse
	} else if percent <= 69 {
		EnchArmor
	} else if percent <= 74 {
		EnchWeapon
	} else if percent <= 80 {
		AggravateMonster
	} else {
		MagicMapping
	};
	(*obj).which_kind = kind.to_index() as u16;
}

pub fn gr_potion(obj: &mut obj) {
	(*obj).what_is = Potion;
	(*obj).which_kind = gr_potion_kind().to_index() as u16;
}

fn gr_potion_kind() -> PotionKind {
	let percent = get_rand(1, 118);
	let kind = if percent <= 5 {
		RaiseLevel
	} else if percent <= 15 {
		DetectObjects
	} else if percent <= 25 {
		DetectMonster
	} else if percent <= 35 {
		IncreaseStrength
	} else if percent <= 45 {
		RestoreStrength
	} else if percent <= 55 {
		Healing
	} else if percent <= 65 {
		ExtraHealing
	} else if percent <= 75 {
		Blindness
	} else if percent <= 85 {
		Hallucination
	} else if percent <= 95 {
		Confusion
	} else if percent <= 105 {
		Poison
	} else if percent <= 110 {
		Levitation
	} else if percent <= 114 {
		Hallucination
	} else {
		SeeInvisible
	};
	kind
}

pub fn gr_weapon(obj: &mut obj, assign_wk: bool) {
	(*obj).what_is = Weapon;
	if assign_wk {
		(*obj).which_kind = get_rand(0, (WEAPONS - 1) as u16);
	}
	if ((*obj).which_kind == ARROW) || ((*obj).which_kind == DAGGER) || ((*obj).which_kind == SHURIKEN) | ((*obj).which_kind == DART) {
		(*obj).quantity = get_rand(3, 15);
		(*obj).quiver = get_rand(0, 126);
	} else {
		(*obj).quantity = 1;
	}
	(*obj).hit_enchant = 0;
	(*obj).d_enchant = 0;

	let percent = get_rand(1, 96);
	let blessing = get_rand(1, 3);

	let mut increment = 0;
	if percent <= 16 {
		increment = 1;
	} else if percent <= 32 {
		increment = -1;
		(*obj).is_cursed = 1;
	}
	if percent <= 32 {
		for _ in 0..blessing {
			if coin_toss() {
				(*obj).hit_enchant += increment;
			} else {
				(*obj).d_enchant += increment as isize;
			}
		}
	}
}

pub fn gr_armor(obj: &mut obj) {
	(*obj).what_is = Armor;
	(*obj).which_kind = get_rand(0, (ARMORS - 1) as u16);
	(*obj).class = ((*obj).which_kind + 2) as isize;
	if ((*obj).which_kind == PLATE) || ((*obj).which_kind == SPLINT) {
		(*obj).class -= 1;
	}
	(*obj).is_protected = 0;
	(*obj).d_enchant = 0;

	let percent = get_rand(1, 100);
	let blessing = get_rand(1, 3);

	if percent <= 16 {
		(*obj).is_cursed = 1;
		(*obj).d_enchant -= blessing;
	} else if percent <= 33 {
		(*obj).d_enchant += blessing;
	}
}

pub fn gr_wand(obj: &mut obj) {
	(*obj).what_is = Wand;
	(*obj).which_kind = get_rand(0, (WANDS - 1) as u16);
	if (*obj).which_kind == MAGIC_MISSILE {
		(*obj).class = get_rand(6, 12);
	} else if (*obj).which_kind == CANCELLATION {
		(*obj).class = get_rand(5, 9);
	} else {
		(*obj).class = get_rand(3, 6);
	}
}

pub fn get_food(obj: &mut obj, force_ration: bool) {
	obj.what_is = Food;
	if force_ration || rand_percent(80) {
		obj.which_kind = RATION;
	} else {
		obj.which_kind = FRUIT;
	}
}

pub unsafe fn put_stairs(player: &Player, level: &mut Level) {
	let mut row = 0;
	let mut col = 0;
	gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Tunnel], player, level);
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Stairs);
}

pub fn get_armor_class(obj: Option<&obj>) -> isize {
	if let Some(armor) = obj {
		armor.class + armor.d_enchant
	} else { 0 }
}

pub fn alloc_object() -> object {
	let mut obj = empty_obj();
	obj.quantity = 1;
	obj.ichar = 'L';
	obj.is_cursed = 0;
	obj.picked_up = 0;
	obj.in_use_flags = NOT_USED;
	obj.identified = false;
	return obj;
}

pub unsafe fn make_party(level_depth: usize, level: &mut Level) {
	let party_room = gr_room(level);
	level.party_room = Some(party_room);
	let n = if rand_percent(99) { party_objects(party_room, level_depth, level) } else { 11 };
	if rand_percent(99) {
		party_monsters(party_room, n, level_depth, level);
	}
}

pub unsafe fn show_objects(player: &Player, level: &Level) {
	for obj in level_objects.objects() {
		let row = (*obj).row;
		let col = (*obj).col;
		let rc = get_mask_char((*obj).what_is) as chtype;
		if level.dungeon[row as usize][col as usize].is_monster() {
			let monster = MASH.monster_at_spot_mut(row, col);
			if let Some(monster) = monster {
				monster.trail_char = rc;
			}
		}
		let mc = mvinch(row as i32, col as i32);
		if (mc < 'A' as chtype || mc > 'Z' as chtype) && (row != player.rogue.row || col != player.rogue.col) {
			mvaddch(row as i32, col as i32, rc);
		}
	}
	for monster in &MASH.monsters {
		if monster.m_flags.imitates {
			mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.disguise_char);
		}
	}
}

pub unsafe fn put_amulet(player: &Player, level: &mut Level) {
	let mut obj = alloc_object();
	obj.what_is = Amulet;
	rand_place(obj, player, level);
}

pub unsafe fn rand_place(obj: obj, player: &Player, level: &mut Level) {
	let mut row = 0;
	let mut col = 0;
	gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Tunnel], player, level);
	place_at(obj, row, col, level);
}

pub unsafe fn new_object_for_wizard(player: &mut Player) {
	if player.pack_weight_with_new_object(None) >= MAX_PACK_COUNT {
		message("pack full", 0);
		return;
	}
	message("type of object?", 0);
	let ch = {
		const CHOICES: &'static str = "!?:)]=/,\x1B";
		let mut ch: char;
		loop {
			ch = rgetchar();
			match CHOICES.find(ch) {
				None => {
					sound_bell();
				}
				Some(_) => {
					break;
				}
			}
		}
		ch
	};
	check_message();
	if ch == CANCEL {
		return;
	}
	let mut obj = alloc_object();
	let max_kind = match ch {
		'!' => {
			obj.what_is = Potion;
			Some(POTIONS - 1)
		}
		'?' => {
			obj.what_is = Scroll;
			Some(SCROLLS - 1)
		}
		',' => {
			obj.what_is = Amulet;
			None
		}
		':' => {
			get_food(&mut obj, false);
			None
		}
		')' => {
			gr_weapon(&mut obj, false);
			Some(WEAPONS - 1)
		}
		']' => {
			gr_armor(&mut obj);
			Some(ARMORS - 1)
		}
		'/' => {
			gr_wand(&mut obj);
			Some(WANDS - 1)
		}
		'=' => {
			obj.what_is = Ring;
			Some(RINGS - 1)
		}
		_ => None
	};
	if let Some(max_kind) = max_kind {
		if let Some(kind) = get_kind(max_kind) {
			obj.which_kind = kind as u16;
			if obj.what_is == Ring {
				gr_ring(&mut obj, false);
			}
		} else {
			return;
		}
	}
	message(&get_obj_desc(&obj), 0);
	player.combine_or_add_item_to_pack(obj);
}

unsafe fn get_kind(max_kind: usize) -> Option<usize> {
	let good_kind = {
		let good_kind;
		loop {
			let line = get_input_line::<String>("which kind?", None, None, false, true);
			let trimmed_line = line.trim();
			if trimmed_line.is_empty() {
				good_kind = None;
				break;
			}
			match trimmed_line.parse::<isize>() {
				Err(_) => {
					sound_bell();
				}
				Ok(kind) => {
					if kind >= 0 && kind <= max_kind as isize {
						good_kind = Some(kind as usize);
						break;
					}
				}
			}
		}
		good_kind
	};
	good_kind
}

fn next_party(cur_level: usize) -> usize {
	const PARTY_TIME: usize = 10;   /* one party somewhere in each 10 level span */
	let mut n = cur_level;
	while (n % PARTY_TIME) > 0 {
		n += 1;
	}
	get_rand(n + 1, n + PARTY_TIME)
}