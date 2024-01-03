#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_short};
use scroll_kind::SCARE_MONSTER;
use crate::{get_input_line, message, mv_aquatars, print_stats};
use crate::objects::IdStatus::{Called, Identified};
use crate::objects::place_at;

use crate::prelude::*;
use crate::prelude::food_kind::FRUIT;
use crate::prelude::IdStatus::Unidentified;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_EITHER_HAND};
use crate::prelude::object_what::{PackFilter};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Amulets, AnyFrom, Armors, Foods, Potions, Rings, Scrolls, Wands, Weapons};
use crate::prelude::SpotFlag::{Object, Stairs, Trap};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_GOLD};
use crate::prelude::weapon_kind::{ARROW, DAGGER, DART, SHURIKEN};

pub static CURSE_MESSAGE: &'static str = "you can't, it appears to be cursed";
pub const MAX_PACK_COUNT: usize = 24;

pub unsafe fn add_to_pack(obj: *mut object, pack: *mut object, condense: i64) -> *mut object {
	let mut op: *mut object = 0 as *mut object;
	if condense != 0 {
		op = check_duplicate(&*obj, &mut *pack);
		if !op.is_null() {
			free_object(obj);
			return op;
		} else {
			(*obj).ichar = next_avail_ichar();
		}
	}
	if (*pack).next_object.is_null() {
		(*pack).next_object = obj;
	} else {
		op = (*pack).next_object;
		while !(*op).next_object.is_null() {
			op = (*op).next_object;
		}
		(*op).next_object = obj;
	}
	(*obj).next_object = 0 as *mut obj;
	return obj;
}

#[no_mangle]
pub unsafe extern "C" fn take_from_pack(
	mut obj: *mut object,
	mut pack: *mut object,
) -> i64 {
	while (*pack).next_object != obj {
		pack = (*pack).next_object;
	}
	(*pack).next_object = (*(*pack).next_object).next_object;
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn pick_up(row: i64, col: i64, mut status: *mut c_short) -> *mut object {
	let obj = object_at(&level_objects, row, col);
	*status = 1;
	if (*obj).what_is == Scroll
		&& (*obj).which_kind == SCARE_MONSTER
		&& (*obj).picked_up != 0 {
		message("the scroll turns to dust as you pick it up", 0);
		dungeon[row as usize][col as usize] = dungeon[row as usize][col as usize] & !Object.code();
		vanish(&mut *obj, false, &mut level_objects);
		*status = 0;
		if id_scrolls[SCARE_MONSTER as usize].id_status == Unidentified {
			id_scrolls[SCARE_MONSTER as usize].id_status = Identified
		}
		return 0 as *mut object;
	}
	if (*obj).what_is == Gold {
		rogue.gold += (*obj).quantity as isize;
		dungeon[row as usize][col as usize] = dungeon[row as usize][col as usize] & !Object.code();
		take_from_pack(obj, &mut level_objects);
		print_stats(STAT_GOLD);
		return obj;
	}
	if pack_count(obj) >= MAX_PACK_COUNT {
		message("pack too full", 1);
		return 0 as *mut object;
	}
	dungeon[row as usize][col as usize] = dungeon[row as usize][col as usize] & !Object.code();
	take_from_pack(obj, &mut level_objects);

	let obj = add_to_pack(obj, &mut rogue.pack, 1);
	(*obj).picked_up = 1;
	return obj;
}

pub unsafe fn drop_0() {
	if SpotFlag::is_any_set(&vec![Object, Stairs, Trap], dungeon[rogue.row as usize][rogue.col as usize]) {
		message("there's already something there", 0);
		return;
	}
	if rogue.pack.next_object.is_null() {
		message("you have nothing to drop", 0);
		return;
	}
	let ch = pack_letter("drop what?", AllObjects);
	if ch == CANCEL {
		return;
	}
	let mut obj = get_letter_object(ch);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*obj).in_use_flags & BEING_WIELDED != 0 {
		if (*obj).is_cursed != 0 {
			message(CURSE_MESSAGE, 0);
			return;
		}
		unwield(rogue.weapon);
	} else if (*obj).in_use_flags & BEING_WORN != 0 {
		if (*obj).is_cursed != 0 {
			message(CURSE_MESSAGE, 0);
			return;
		}
		mv_aquatars();
		unwear(rogue.armor);
		print_stats(STAT_ARMOR);
	} else if (*obj).in_use_flags & ON_EITHER_HAND != 0 {
		if (*obj).is_cursed != 0 {
			message(CURSE_MESSAGE, 0);
			return;
		}
		un_put_on(obj);
	}
	(*obj).row = rogue.row;
	(*obj).col = rogue.col;

	if (*obj).quantity > 1 && (*obj).what_is != Weapon {
		(*obj).quantity -= 1;
		let new = alloc_object();
		*new = (*obj).clone();
		(*new).quantity = 1;
		obj = new;
	} else {
		(*obj).ichar = 'L';
		take_from_pack(obj, &mut rogue.pack);
	}
	place_at(obj, rogue.row, rogue.col);
	message(&format!("dropped {}", get_desc(&*obj)), 0);
	reg_move();
}

pub unsafe fn check_duplicate(obj: &object, pack: &mut object) -> *mut object {
	match obj.what_is {
		Weapon | Food | Scroll | Potion => {}
		_ => {
			return 0 as *mut object;
		}
	}
	if (*obj).what_is == Food && (*obj).which_kind == FRUIT {
		return 0 as *mut object;
	}
	let mut op = (*pack).next_object;
	while !op.is_null() {
		if (*op).what_is == (*obj).what_is && (*op).which_kind == (*obj).which_kind {
			if (*obj).what_is != Weapon ||
				(
					(*obj).what_is == Weapon
						&& ((*obj).which_kind == ARROW || (*obj).which_kind == DAGGER || (*obj).which_kind == DART || (*obj).which_kind == SHURIKEN)
						&& (*obj).quiver == (*op).quiver
				) {
				(*op).quantity += (*obj).quantity;
				return op;
			}
		}
		op = (*op).next_object;
	}
	return 0 as *mut object;
}

pub unsafe fn next_avail_ichar() -> char {
	let mut used = [false; 26];
	{
		let mut obj = rogue.pack.next_object;
		while !obj.is_null() {
			used[((*obj).ichar as u8 - 'a' as u8) as usize] = true;
			obj = (*obj).next_object;
		}
	}
	let unused = used.into_iter().position(|used| used == false);
	if let Some(unused) = unused {
		(unused as u8 + 'a' as u8) as char
	} else {
		'?'
	}
}

pub unsafe fn wait_for_ack() {
	while rgetchar() != ' ' {}
}

pub unsafe fn pack_letter(prompt: &str, filter: PackFilter) -> char {
	if !mask_pack(&rogue.pack, filter.clone()) {
		message("nothing appropriate", 0);
		return CANCEL;
	}

	loop {
		message(prompt, 0);
		let pack_op = {
			let mut pack_op = None;
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
				inventory(&rogue.pack, filter);
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

pub unsafe fn take_off() {
	if !rogue.armor.is_null() {
		if (*rogue.armor).is_cursed != 0 {
			message(CURSE_MESSAGE, 0);
		} else {
			mv_aquatars();
			let obj = rogue.armor;
			unwear(rogue.armor);
			message(&format!("was wearing {}", get_desc(&*obj)), 0);
			print_stats(STAT_ARMOR);
			reg_move();
		}
	} else {
		message("not wearing any", 0);
	}
}

pub unsafe fn wear() {
	if !rogue.armor.is_null() {
		message("your already wearing some", 0);
		return;
	}
	let ch = pack_letter("wear what?", Armors);
	if ch == CANCEL {
		return;
	}
	let obj = get_letter_object(ch);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*obj).what_is != Armor {
		message("you can't wear that", 0);
		return;
	}
	(*obj).identified = true;
	message(&format!("wearing {}", get_desc(&*obj)), 0);
	do_wear(&mut *obj);
	print_stats(STAT_ARMOR);
	reg_move();
}

pub unsafe fn do_wear(obj: &mut obj) {
	rogue.armor = obj;
	obj.in_use_flags |= BEING_WORN;
	obj.identified = true;
}

pub unsafe fn unwear(obj: *mut object) {
	if !obj.is_null() {
		(*obj).in_use_flags &= !BEING_WORN;
	}
	rogue.armor = 0 as *mut object;
}


pub unsafe fn wield() {
	if !rogue.weapon.is_null() && (*rogue.weapon).is_cursed != 0 {
		message(CURSE_MESSAGE, 0);
		return;
	}
	let ch = pack_letter("wield what?", Weapons);
	if ch == CANCEL {
		return;
	}
	let obj = get_letter_object(ch);
	if obj.is_null() {
		message("No such item.", 0);
		return;
	}
	if (*obj).what_is == Armor || (*obj).what_is == Ring {
		let item_name = if (*obj).what_is == Armor { "armor" } else { "rings" };
		message(&format!("you can't wield {}", item_name), 0);
		return;
	}
	if ((*obj).in_use_flags & BEING_WIELDED) != 0 {
		message("in use", 0);
	} else {
		unwield(rogue.weapon);
		message(&format!("wielding {}", get_desc(&*obj)), 0);
		do_wield(&mut *obj);
		reg_move();
	}
}

pub unsafe fn do_wield(obj: &mut obj) {
	rogue.weapon = obj;
	obj.in_use_flags |= BEING_WIELDED;
}

pub unsafe fn unwield(obj: *mut obj) {
	if !obj.is_null() {
		(*obj).in_use_flags &= !BEING_WIELDED;
	}
	rogue.weapon = 0 as *mut object;
}

pub unsafe fn call_it() {
	let ch = pack_letter("call what?", AnyFrom(vec![Scroll, Potion, Wand, Ring]));
	if ch == CANCEL {
		return;
	}

	let obj = get_letter_object(ch);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	match (*obj).what_is {
		Scroll | Potion | Wand | Ring => {}
		_ => {
			message("surely you already know what that's called", 0);
			return;
		}
	}
	let id_table = get_id_table(&*obj);
	let new_name = get_input_line("call it:", None, Some(&id_table[(*obj).which_kind as usize].title), true, true);
	if !new_name.is_empty() {
		id_table[(*obj).which_kind as usize].id_status = Called;
		id_table[(*obj).which_kind as usize].title = new_name;
	}
}

pub unsafe fn pack_count(new_obj: *const obj) -> usize {
	let mut count = 0;
	let mut obj = rogue.pack.next_object;
	while !obj.is_null() {
		if (*obj).what_is != Weapon {
			count += (*obj).quantity;
		} else if new_obj.is_null() {
			count += 1;
		} else if ((*new_obj).what_is != Weapon)
			|| (((*obj).which_kind != ARROW) && ((*obj).which_kind != DAGGER) && ((*obj).which_kind != DART) && ((*obj).which_kind != SHURIKEN))
			|| ((*new_obj).which_kind != (*obj).which_kind)
			|| ((*obj).quiver != (*new_obj).quiver) {
			count += 1;
		}
		obj = (*obj).next_object;
	}
	count as usize
}


pub unsafe fn mask_pack(pack: *const object, mask: PackFilter) -> bool {
	let mut next = (*pack).next_object;
	while !next.is_null() {
		let what = (*next).what_is;
		if mask.includes(what) {
			return true;
		}
		next = (*next).next_object;
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

pub unsafe fn has_amulet() -> bool {
	mask_pack(&rogue.pack, Amulets)
}

pub unsafe fn kick_into_pack() {
	if Object.is_set(dungeon[rogue.row as usize][rogue.col as usize]) {
		message("nothing here", 0);
	} else {
		let mut status = 0;
		let obj = pick_up(rogue.row, rogue.col, &mut status);
		if !obj.is_null() {
			let obj_desc = get_desc(&*obj);
			if (*obj).what_is == Gold {
				message(&obj_desc, 0);
				free_object(obj);
			} else {
				message(&format!("{}({})", obj_desc, (*obj).ichar), 0);
			}
		}
		if !obj.is_null() || status == 0 {
			reg_move();
		}
	}
}
