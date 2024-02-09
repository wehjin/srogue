#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::clone::Clone;
use std::string::ToString;
use ncurses::{chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};
use ObjectWhat::{Armor, Potion, Scroll, Weapon};
use crate::level::constants::MAX_ROOM;
use crate::odds::GOLD_PERCENT;
use crate::prelude::*;
use crate::prelude::armor_kind::{ARMORS, PLATE, SPLINT};
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::item_usage::{being_wielded, being_worn, on_either_hand, on_left_hand};
use crate::prelude::object_what::{ObjectWhat};
use crate::prelude::object_what::ObjectWhat::{Amulet, Food, Gold, Ring, Wand};
use crate::prelude::potion_kind::PotionKind::{Blindness, Confusion, DetectMonster, DetectObjects, ExtraHealing, Hallucination, Healing, IncreaseStrength, Levitation, Poison, RaiseLevel, RestoreStrength, SeeInvisible};
use crate::prelude::potion_kind::{PotionKind, POTIONS};
use crate::prelude::ring_kind::RINGS;
use crate::prelude::scroll_kind::ScrollKind::{AggravateMonster, CreateMonster, EnchArmor, EnchWeapon, HoldMonster, Identify, MagicMapping, ProtectArmor, RemoveCurse, ScareMonster, Sleep, Teleport};
use crate::prelude::scroll_kind::SCROLLS;
use crate::prelude::wand_kind::{CANCELLATION, MAGIC_MISSILE, MAX_WAND};
use crate::prelude::weapon_kind::{ARROW, DAGGER, DART, SHURIKEN, WEAPONS};
use crate::settings::fruit;

#[derive(Clone, Serialize, Deserialize)]
pub struct id {
	pub value: i16,
	pub title: Option<String>,
	pub id_status: IdStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum IdStatus {
	Unidentified,
	Identified,
	Called,
}

#[derive(Serialize, Deserialize)]
pub struct SaveObj {
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

impl SaveObj {
	pub unsafe fn option_save_obj(obj: *const object) -> Option<SaveObj> {
		if obj.is_null() {
			None
		} else {
			Some(Self::from_obj(&*obj))
		}
	}
	pub unsafe fn to_obj(&self, is_rogue: bool) -> *mut obj {
		let heap_obj = alloc_object();
		*heap_obj = obj {
			quantity: self.quantity,
			ichar: self.ichar,
			kill_exp: self.kill_exp,
			is_protected: self.is_protected,
			is_cursed: self.is_cursed,
			class: self.class,
			identified: self.identified,
			which_kind: self.which_kind,
			o_row: self.o_row,
			o_col: self.o_col,
			o: self.o,
			row: self.row,
			col: self.col,
			d_enchant: self.d_enchant,
			quiver: self.quiver,
			trow: self.trow,
			tcol: self.tcol,
			hit_enchant: self.hit_enchant,
			what_is: self.what_is,
			picked_up: self.picked_up,
			in_use_flags: self.in_use_flags,
			next_object: 0 as *mut obj,
		};
		if is_rogue {
			if being_worn(&*heap_obj) {
				do_wear(&mut *heap_obj);
			} else if being_wielded(&*heap_obj) {
				do_wield(&mut *heap_obj);
			} else if on_either_hand(&*heap_obj) {
				do_put_on(&mut *heap_obj, on_left_hand(&*heap_obj));
			}
		}
		heap_obj
	}
	pub fn from_obj(obj: &obj) -> Self {
		Self {
			quantity: obj.quantity,
			ichar: obj.ichar,
			kill_exp: obj.kill_exp,
			is_protected: obj.is_protected,
			is_cursed: obj.is_cursed,
			class: obj.class,
			identified: obj.identified,
			which_kind: obj.which_kind,
			o_row: obj.o_row,
			o_col: obj.o_col,
			o: obj.o,
			row: obj.row,
			col: obj.col,
			d_enchant: obj.d_enchant,
			quiver: obj.quiver,
			trow: obj.trow,
			tcol: obj.tcol,
			hit_enchant: obj.hit_enchant,
			what_is: obj.what_is,
			picked_up: obj.picked_up,
			in_use_flags: obj.in_use_flags,
		}
	}
}

#[derive(Clone)]
pub struct obj {
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
	pub next_object: *mut obj,
}

pub const fn empty_obj() -> obj {
	obj {
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
		next_object: 0 as *mut obj,
	}
}

impl obj {
	pub fn next_monster(&self) -> *mut obj {
		self.next_object
	}
	pub fn set_next_monster(&mut self, value: *mut obj) {
		self.next_object = value;
	}
}

pub type object = obj;

pub static mut level_objects: object = empty_obj();
pub static mut foods: i16 = 0;
pub static mut party_counter: usize = 0;
pub static mut free_list: *mut object = 0 as *mut object;
pub static mut rogue: Fighter = Fighter {
	armor: 0 as *mut object,
	weapon: 0 as *mut object,
	left_ring: 0 as *mut object,
	right_ring: 0 as *mut object,
	hp_current: 12,
	hp_max: 12,
	str_current: 16,
	str_max: 16,
	pack: empty_obj(),
	gold: 0,
	exp: 1,
	exp_points: 0,
	row: 0,
	col: 0,
	fchar: '@',
	moves_left: 1250,
};
pub static mut id_potions: [id; POTIONS] = {
	[
		{
			let mut init = id {
				value: 100,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 250,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 200,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 10,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 300,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 10,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 10,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 80,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 150,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 145,
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
			let mut init = id {
				value: 505,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 200,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 235,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 235,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 175,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 190,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 610,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 210,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 180,
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
			let mut init = id {
				value: 150,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 8,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 15,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 27,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 35,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 360,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 470,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 580,
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
			let mut init = id {
				value: 300,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 300,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 400,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 500,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 600,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 600,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 700,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_wands: [id; MAX_WAND] = {
	[
		{
			let mut init = id {
				value: 25,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 50,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 45,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 8,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 55,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 2,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 20,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 20,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 0,
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
			let mut init = id {
				value: 250,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 255,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 295,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 200,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 250,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 250,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 300,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 290,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 270,
				title: None,
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};

pub unsafe fn put_objects(depth: &RogueDepth, level: &mut Level) {
	if depth.cur < depth.max {
		return;
	}

	let mut n = if coin_toss() { get_rand(2, 4) } else { get_rand(3, 5) };
	while rand_percent(33) {
		n += 1;
	}
	if depth.cur == party_counter {
		make_party(depth.cur, level);
		party_counter = next_party(depth.cur);
	}
	for _i in 0..n {
		let obj = gr_object(depth.cur);
		rand_place(&mut *obj, level);
	}
	put_gold(depth.cur, level);
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
	let obj = alloc_object();
	(*obj).row = row;
	(*obj).col = col;
	(*obj).what_is = Gold;
	(*obj).quantity = get_rand((2 * cur_level) as i16, (16 * cur_level) as i16);
	if is_maze {
		(*obj).quantity += (*obj).quantity / 2;
	}
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Object);
	add_to_pack(obj, &mut level_objects, 0);
}


pub unsafe fn place_at(obj: &mut object, row: i64, col: i64, level: &mut Level) {
	obj.row = row;
	obj.col = col;
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Object);
	add_to_pack(obj, &mut level_objects, 0);
}

pub unsafe fn object_at(pack: &object, row: i64, col: i64) -> *mut object {
	let mut obj = pack.next_object;
	while !obj.is_null() && ((*obj).row != row || (*obj).col != col) {
		obj = (*obj).next_object;
	}
	obj
}

pub unsafe fn get_letter_object(ch: char) -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = rogue.pack.next_object;
	while !obj.is_null() && (*obj).ichar != ch {
		obj = (*obj).next_object;
	}
	return obj;
}

pub unsafe fn free_stuff(mut obj_list: *mut object) {
	while !(*obj_list).next_object.is_null() {
		let obj = (*obj_list).next_object;
		(*obj_list).next_object = (*(*obj_list).next_object).next_object;
		free_object(obj);
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

pub unsafe fn gr_object(cur_level: usize) -> *mut object {
	let mut obj = alloc_object();
	if foods < (cur_level / 2) as i16 {
		(*obj).what_is = Food;
		foods += 1;
	} else {
		(*obj).what_is = gr_what_is();
	}
	match (*obj).what_is {
		Scroll => {
			gr_scroll(&mut *obj);
		}
		Potion => {
			gr_potion(&mut *obj);
		}
		Weapon => {
			gr_weapon(&mut *obj, true);
		}
		Armor => {
			gr_armor(&mut *obj);
		}
		Wand => {
			gr_wand(&mut *obj);
		}
		Food => {
			get_food(&mut *obj, false);
		}
		Ring => {
			gr_ring(&mut *obj, true);
		}
		_ => {}
	}
	return obj;
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

pub fn weapon_damage(weapon: &obj) -> &'static str {
	weapon_kind::damage(weapon.which_kind)
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
	(*obj).which_kind = get_rand(0, (MAX_WAND - 1) as u16);
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

pub unsafe fn put_stairs(level: &mut Level) {
	let mut row = 0;
	let mut col = 0;
	gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Tunnel], level);
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Stairs);
}

pub unsafe fn get_armor_class(obj: *const object) -> isize {
	(*obj).class + (*obj).d_enchant
}

pub unsafe fn alloc_object() -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	if !free_list.is_null() {
		obj = free_list;
		free_list = (*free_list).next_object;
	} else {
		obj = md_malloc(core::mem::size_of::<object>() as i64) as *mut object;
		if obj.is_null() {
			message("cannot allocate object, saving game", 0);
			save_into_file(ERROR_FILE, unimplemented!("Acquire game state or move error handling to higher level"));
		}
	}
	(*obj).quantity = 1;
	(*obj).ichar = 'L';
	(*obj).is_cursed = 0;
	(*obj).picked_up = (*obj).is_cursed;
	(*obj).in_use_flags = 0;
	(*obj).identified = false;
	return obj;
}

pub unsafe fn free_object(obj: *mut object) {
	(*obj).next_object = free_list;
	free_list = obj;
}

pub unsafe fn make_party(level_depth: usize, level: &mut Level) {
	party_room = Some(gr_room(level));
	let cur_party_room = party_room.expect("some party room");
	let n = if rand_percent(99) { party_objects(cur_party_room, level_depth, level) } else { 11 };
	if rand_percent(99) {
		party_monsters(cur_party_room, n, level_depth, level);
	}
}

pub unsafe fn show_objects(level: &Level) {
	let mut obj = level_objects.next_object;
	while !obj.is_null() {
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
		if (mc < 'A' as chtype || mc > 'Z' as chtype) && (row != rogue.row || col != rogue.col) {
			mvaddch(row as i32, col as i32, rc);
		}
		obj = (*obj).next_object;
	}
	for monster in &MASH.monsters {
		if monster.m_flags.imitates {
			mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.disguise_char);
		}
	}
}

pub unsafe fn put_amulet(level: &mut Level) {
	let mut obj = alloc_object();
	(*obj).what_is = Amulet;
	rand_place(&mut *obj, level);
}

pub unsafe fn rand_place(obj: &mut obj, level: &mut Level) {
	let mut row = 0;
	let mut col = 0;
	gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Tunnel], level);
	place_at(obj, row, col, level);
}

pub unsafe fn new_object_for_wizard() {
	if pack_count(0 as *mut object) >= MAX_PACK_COUNT {
		message("pack full", 0);
		return;
	}
	message("type of object?", 0);
	let ch = {
		const CHOICES: &'static str = "!?:)]=/,\x1B";
		let mut ch: char = char::default();
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
	let obj = alloc_object();
	let max_kind = match ch {
		'!' => {
			(*obj).what_is = Potion;
			Some(POTIONS - 1)
		}
		'?' => {
			(*obj).what_is = Scroll;
			Some(SCROLLS - 1)
		}
		',' => {
			(*obj).what_is = Amulet;
			None
		}
		':' => {
			get_food(&mut *obj, false);
			None
		}
		')' => {
			gr_weapon(&mut *obj, false);
			Some(WEAPONS - 1)
		}
		']' => {
			gr_armor(&mut *obj);
			Some(ARMORS - 1)
		}
		'/' => {
			gr_wand(&mut *obj);
			Some(MAX_WAND - 1)
		}
		'=' => {
			(*obj).what_is = Ring;
			Some(RINGS - 1)
		}
		_ => None
	};
	if let Some(max_kind) = max_kind {
		if let Some(kind) = get_kind(max_kind) {
			(*obj).which_kind = kind as u16;
			if (*obj).what_is == Ring {
				gr_ring(&mut *obj, false);
			}
		} else {
			free_object(obj);
			return;
		}
	}
	message(&get_desc(&*obj), 0);
	add_to_pack(obj, &mut rogue.pack, 1);
}

unsafe fn get_kind(max_kind: usize) -> Option<usize> {
	let good_kind = {
		let mut good_kind = None;
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

pub unsafe fn next_party(cur_level: usize) -> usize {
	let mut n = cur_level;
	while (n % PARTY_TIME) > 0 {
		n += 1;
	}
	return get_rand(n + 1, n + PARTY_TIME);
}