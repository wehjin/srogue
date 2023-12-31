#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;

	fn md_malloc() -> *mut libc::c_char;
	static mut error_file: *mut libc::c_char;
}

use libc::{c_int, c_short};
use ncurses::{addch, chtype};
use serde::Serialize;
use crate::prelude::*;
use crate::prelude::armor_kind::ARMORS;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::{ObjectWhat};
use crate::prelude::object_what::ObjectWhat::Food;
use crate::prelude::potion_kind::POTIONS;
use crate::prelude::ring_kind::RINGS;
use crate::prelude::scroll_kind::SCROLLS;
use crate::prelude::wand_kind::WANDS;
use crate::prelude::weapon_kind::WEAPONS;
use crate::settings::fruit;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: libc::c_short,
	pub _pad_x: libc::c_short,
	pub _pad_top: libc::c_short,
	pub _pad_left: libc::c_short,
	pub _pad_bottom: libc::c_short,
	pub _pad_right: libc::c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = ncurses::chtype;

#[derive(Clone, Serialize)]
pub struct id {
	pub value: i16,
	pub title: String,
	pub real: String,
	pub id_status: IdStatus,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub enum IdStatus {
	Unidentified,
	Identified,
	Called,
}

#[derive(Serialize)]
pub struct SaveObj {
	pub m_flags: MonsterFlags,
	pub damage: String,
	pub quantity: i16,
	pub ichar: char,
	pub kill_exp: isize,
	pub is_protected: i16,
	pub is_cursed: i16,
	pub class: isize,
	pub identified: bool,
	pub stationary_damage: isize,
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
	pub disguise: u16,
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
	pub fn from_obj(obj: &obj) -> Self {
		Self {
			m_flags: obj.m_flags,
			damage: obj.damage.to_string(),
			quantity: obj.quantity,
			ichar: obj.ichar,
			kill_exp: obj.kill_exp,
			is_protected: obj.is_protected,
			is_cursed: obj.is_cursed,
			class: obj.class,
			identified: obj.identified,
			stationary_damage: obj.stationary_damage,
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
			disguise: obj.disguise,
			picked_up: obj.picked_up,
			in_use_flags: obj.in_use_flags,
		}
	}
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct obj {
	pub m_flags: MonsterFlags,
	pub damage: &'static str,
	pub quantity: i16,
	pub ichar: char,
	pub kill_exp: isize,
	pub is_protected: i16,
	pub is_cursed: i16,
	pub class: isize,
	pub identified: bool,
	pub stationary_damage: isize,
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
	pub disguise: u16,
	pub picked_up: i16,
	pub in_use_flags: u16,
	pub next_object: *mut obj,
}

impl obj {
	pub fn m_hit_chance(&self) -> usize {
		self.class as usize
	}
	pub fn hp_to_kill(&self) -> c_short { self.quantity }
	pub fn set_hp_to_kill(&mut self, value: c_short) { self.quantity = value }
	pub fn m_char(&self) -> chtype {
		self.ichar as ncurses::chtype
	}
	pub fn stationary_damage(&self) -> isize { self.stationary_damage }

	pub fn set_stationary_damage(&mut self, value: isize) {
		self.stationary_damage = value;
	}
	pub fn first_level(&self) -> isize {
		self.is_protected as isize
	}
	pub fn set_first_level(&mut self, value: isize) {
		self.is_protected = value as i16;
	}
	pub fn set_trail_char(&mut self, ch: chtype) { self.d_enchant = ch as isize; }
	pub fn trail_char(&self) -> chtype {
		self.d_enchant as chtype
	}
	pub fn disguise(&self) -> chtype {
		self.disguise as chtype
	}
	pub fn nap_length(&self) -> c_short {
		self.picked_up
	}
	pub fn set_nap_length(&mut self, value: c_short) {
		self.picked_up = value;
	}
	pub fn decrement_nap(&mut self) {
		self.set_nap_length(self.nap_length() - 1);
		if self.nap_length() <= 0 {
			self.m_flags.napping = false;
			self.m_flags.asleep = false;
		}
	}
	pub fn moves_confused(&self) -> c_short {
		self.hit_enchant
	}
	pub fn decrement_moves_confused(&mut self) {
		self.hit_enchant -= 1;
		if self.hit_enchant <= 0 {
			self.m_flags.confuses = false;
		}
	}

	pub fn slowed_toggled(&self) -> bool {
		self.quiver == 1
	}

	pub fn flip_slowed_toggle(&mut self) {
		if self.quiver == 1 {
			self.quiver = 0;
		} else {
			self.quiver = 1;
		}
	}
	pub fn in_room(&self, rn: i64) -> bool {
		let object_rn = get_room_number(self.row as i64, self.col as i64);
		object_rn != NO_ROOM && object_rn == rn
	}
}

pub type object = obj;

#[no_mangle]
pub static mut level_objects: object = obj {
	m_flags: MonsterFlags::default(),
	damage: "",
	quantity: 0,
	ichar: '\u{00}',
	kill_exp: 0,
	is_protected: 0,
	is_cursed: 0,
	class: 0,
	identified: false,
	stationary_damage: 0,
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
	disguise: 0,
	picked_up: 0,
	in_use_flags: 0,
	next_object: 0 as *const obj as *mut obj,
};
pub static mut dungeon: [[u16; DCOLS]; DROWS] = [[0; DCOLS]; DROWS];
pub static mut foods: i16 = 0;
pub static mut party_counter: i16 = 0;
#[no_mangle]
pub static mut free_list: *mut object = 0 as *const object as *mut object;
#[no_mangle]
pub static mut rogue: fighter = {
	let mut init = fight {
		armor: 0 as *const object as *mut object,
		weapon: 0 as *const object as *mut object,
		left_ring: 0 as *const object as *mut object,
		right_ring: 0 as *const object as *mut object,
		hp_current: 12,
		hp_max: 12,
		str_current: 16,
		str_max: 16,
		pack: {
			let mut init = obj {
				m_flags: MonsterFlags::default(),
				damage: "",
				quantity: 0,
				ichar: '\u{00}',
				kill_exp: 0,
				is_protected: 0,
				is_cursed: 0,
				class: 0,
				identified: false,
				stationary_damage: 0,
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
				disguise: 0,
				picked_up: 0,
				in_use_flags: 0,
				next_object: 0 as *const obj as *mut obj,
			};
			init
		},
		gold: 0,
		exp: 1,
		exp_points: 0,
		row: 0,
		col: 0,
		fchar: '@',
		moves_left: 1250,
	};
	init
};
pub static mut id_potions: [id; POTIONS] = {
	[
		{
			let mut init = id {
				value: 100,
				title: "blue ".to_string(),
				real: "of increase strength ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 250,
				title: "red ".to_string(),
				real: "of restore strength ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: "green ".to_string(),
				real: "of healing ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 200,
				title: "grey ".to_string(),
				real: "of extra healing ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 10,
				title: "brown ".to_string(),
				real: "of poison ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 300,
				title: "clear ".to_string(),
				real: "of raise level ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 10,
				title: "pink ".to_string(),
				real: "of blindness ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as i64 as libc::c_short,
				title: "white ".to_string(),
				real: "of hallucination ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: "purple ".to_string(),
				real: "of detect monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: "black ".to_string(),
				real: "of detect things ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 10,
				title: "yellow ".to_string(),
				real: "of confusion ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 80,
				title: "plaid ".to_string(),
				real: "of levitation ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 150,
				title: "burgundy ".to_string(),
				real: "of haste self ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 145 as i64 as libc::c_short,
				title: "beige ".to_string(),
				real: "of see invisible ".to_string(),
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
				value: 505 as i64 as libc::c_short,
				title: "                                   ".to_string(),
				real: "of protect armor ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 200,
				title: "                                   ".to_string(),
				real: "of hold monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 235 as i64 as libc::c_short,
				title: "                                   ".to_string(),
				real: "of enchant weapon ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 235 as i64 as libc::c_short,
				title: "                                   ".to_string(),
				real: "of enchant armor ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 175 as i64 as libc::c_short,
				title: "                                   ".to_string(),
				real: "of identify ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 190,
				title: "                                   ".to_string(),
				real: "of teleportation ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as i64 as libc::c_short,
				title: "                                   ".to_string(),
				real: "of sleep ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 610,
				title: "                                   ".to_string(),
				real: "of scare monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 210,
				title: "                                   ".to_string(),
				real: "of remove curse ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: "                                   ".to_string(),
				real: "of create monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as i64 as libc::c_short,
				title: "                                   ".to_string(),
				real: "of aggravate monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 180,
				title: "                                   ".to_string(),
				real: "of magic mapping ".to_string(),
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
				title: "short bow ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 8 as i64 as libc::c_short,
				title: "darts ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 15 as i64 as libc::c_short,
				title: "arrows ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 27 as i64 as libc::c_short,
				title: "daggers ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 35 as i64 as libc::c_short,
				title: "shurikens ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 360,
				title: "mace ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 470,
				title: "long sword ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 580,
				title: "two-handed sword ".to_string(),
				real: "".to_string(),
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
				title: "leather armor ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 300,
				title: "ring mail ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 400,
				title: "scale mail ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 500,
				title: "chain mail ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 600,
				title: "banded mail ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 600,
				title: "splint mail ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 700,
				title: "plate mail ".to_string(),
				real: "".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};
pub static mut id_wands: [id; WANDS] = {
	[
		{
			let mut init = id {
				value: 25 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of teleport away ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 50,
				title: "                                 ".to_string(),
				real: "of slow monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 45 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of confuse monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 8 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of invisibility ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 55 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of polymorph ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 2,
				title: "                                 ".to_string(),
				real: "of haste monster ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of sleep ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 20,
				title: "                                 ".to_string(),
				real: "of magic missile ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 20,
				title: "                                 ".to_string(),
				real: "of cancellation ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 0,
				title: "                                 ".to_string(),
				real: "of do nothing ".to_string(),
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
				title: "                                 ".to_string(),
				real: "of stealth ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 100,
				title: "                                 ".to_string(),
				real: "of teleportation ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 255 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of regeneration ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 295 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of slow digestion ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 200,
				title: "                                 ".to_string(),
				real: "of add strength ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 250,
				title: "                                 ".to_string(),
				real: "of sustain strength ".to_string(),

				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 250,
				title: "                                 ".to_string(),
				real: "of dexterity ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as i64 as libc::c_short,
				title: "                                 ".to_string(),
				real: "of adornment ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 300,
				title: "                                 ".to_string(),
				real: "of see invisible ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 290,
				title: "                                 ".to_string(),
				real: "of maintain armor ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
		{
			let mut init = id {
				value: 270,
				title: "                                 ".to_string(),
				real: "of searching ".to_string(),
				id_status: IdStatus::Unidentified,
			};
			init
		},
	]
};

#[no_mangle]
pub unsafe extern "C" fn put_objects() {
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	if (cur_level as i64) < max_level as i64 {
		return;
	}
	n = (if coin_toss() {
		get_rand(2 as i64, 4 as i64)
	} else {
		get_rand(3 as i64, 5 as i64)
	}) as libc::c_short;
	while rand_percent(33) {
		n += 1;
		n;
	}
	if cur_level == party_counter {
		make_party();
		party_counter = next_party() as libc::c_short;
	}
	i = 0;
	while (i as i64) < n as i64 {
		obj = gr_object();
		rand_place(obj);
		i += 1;
		i;
	}
	put_gold();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn place_at(
	mut obj: *mut object,
	mut row: i64,
	mut col: i64,
) -> i64 {
	(*obj).row = row as libc::c_short;
	(*obj).col = col as libc::c_short;
	dungeon[row
		as usize][col
		as usize] = (dungeon[row as usize][col as usize] as i64
		| 0o1 as libc::c_ushort as i64) as libc::c_ushort;
	add_to_pack(obj, &mut level_objects, 0 as i64);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn object_at(mut pack: *mut object, mut row: i64, mut col: i64) -> *mut object {
	let mut obj: *mut object = (*pack).next_object;
	while !obj.is_null()
		&& ((*obj).row as i64 != row as i64
		|| (*obj).col as i64 != col as i64)
	{
		obj = (*obj).next_object;
	}
	return obj;
}

pub unsafe fn get_letter_object(ch: char) -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = rogue.pack.next_object;
	while !obj.is_null() && (*obj).ichar as i64 != ch {
		obj = (*obj).next_object;
	}
	return obj;
}

#[no_mangle]
pub unsafe extern "C" fn free_stuff(mut objlist: *mut object) -> i64 {
	let mut obj: *mut object = 0 as *mut object;
	while !((*objlist).next_object).is_null() {
		obj = (*objlist).next_object;
		(*objlist).next_object = (*(*objlist).next_object).next_object;
		free_object(obj);
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn name_of(obj: &object) -> String {
	match obj.what_is {
		ObjectWhat::Armor => "armor ",
		ObjectWhat::Weapon => match obj.which_kind {
			weapon_kind::DART => if obj.quantity > 1 { "darts " } else { "dart " },
			weapon_kind::ARROW => if obj.quantity > 1 { "arrows " } else { "arrow " },
			weapon_kind::DAGGER => if obj.quantity > 1 { "daggers " } else { "dagger " },
			weapon_kind::SHURIKEN => if obj.quantity > 1 { "shurikens " } else { "shuriken " },
			_ => &id_weapons[obj.which_kind].title
		},
		ObjectWhat::Scroll => if obj.quantity > 1 { "scrolls " } else { "scroll " }
		ObjectWhat::Potion => if obj.quantity > 1 { "potions " } else { "potion " }
		ObjectWhat::Food => if obj.which_kind == RATION { "food " } else { &fruit(); }
		ObjectWhat::Wand => if is_wood[obj.which_kind] { "staff " } else { "wand " },
		ObjectWhat::Ring => "ring ",
		ObjectWhat::Amulet => "amulet ",
		_ => "unknown ",
	}.to_string()
}

#[no_mangle]
pub unsafe extern "C" fn gr_object() -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = alloc_object();
	if (foods as i64) < cur_level as i64 / 2 as i64 {
		(*obj).what_is = 0o40 as i64 as libc::c_ushort;
		foods += 1;
		foods;
	} else {
		(*obj).what_is = gr_what_is();
	}
	match (*obj).what_is as i64 {
		4 => {
			gr_scroll(obj);
		}
		8 => {
			gr_potion(obj);
		}
		2 => {
			gr_weapon(obj, 1);
		}
		1 => {
			gr_armor(obj);
		}
		64 => {
			gr_wand(obj);
		}
		32 => {
			get_food(&mut *obj, false);
		}
		128 => {
			gr_ring(obj, 1 as libc::c_int);
		}
		_ => {}
	}
	return obj;
}

#[no_mangle]
pub unsafe extern "C" fn gr_what_is() -> libc::c_ushort {
	let mut percent: libc::c_short = 0;
	let mut what_is: libc::c_ushort = 0;
	percent = get_rand(1 as libc::c_int, 91 as libc::c_int) as libc::c_short;
	if percent as libc::c_int <= 30 as libc::c_int {
		what_is = 0o4 as libc::c_int as libc::c_ushort;
	} else if percent as libc::c_int <= 60 as libc::c_int {
		what_is = 0o10 as libc::c_int as libc::c_ushort;
	} else if percent as libc::c_int <= 64 as libc::c_int {
		what_is = 0o100 as libc::c_int as libc::c_ushort;
	} else if percent as libc::c_int <= 74 as libc::c_int {
		what_is = 0o2 as libc::c_int as libc::c_ushort;
	} else if percent as libc::c_int <= 83 as libc::c_int {
		what_is = 0o1 as libc::c_int as libc::c_ushort;
	} else if percent as libc::c_int <= 88 as libc::c_int {
		what_is = 0o40 as libc::c_int as libc::c_ushort;
	} else {
		what_is = 0o200 as libc::c_int as libc::c_ushort;
	}
	return what_is;
}

pub fn get_food(obj: &mut obj, force_ration: bool) {
	obj.what_is = WhatIs(Food);
	if force_ration || rand_percent(80) {
		obj.which_kind = RATION;
	} else {
		obj.which_kind = FRUIT;
	}
}

#[no_mangle]
pub unsafe extern "C" fn put_stairs() -> libc::c_int {
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	gr_row_col(
		&mut row,
		&mut col,
		0o100 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o200 as libc::c_int as libc::c_ushort as libc::c_int,
	);
	dungeon[row
		as usize][col
		as usize] = (dungeon[row as usize][col as usize] as libc::c_int
		| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) as libc::c_ushort;
	panic!("Reached end of non-void function without returning");
}

pub fn get_armor_class(obj: &obj) -> isize {
	obj.class + obj.d_enchant
}

#[no_mangle]
pub unsafe extern "C" fn alloc_object() -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	if !free_list.is_null() {
		obj = free_list;
		free_list = (*free_list).next_object;
	} else {
		obj = md_malloc(::core::mem::size_of::<object>() as libc::c_ulong)
			as *mut object;
		if obj.is_null() {
			message("cannot allocate object, saving game", 0 as libc::c_int);
			save_into_file(error_file);
		}
	}
	(*obj).quantity = 1 as libc::c_int as libc::c_short;
	(*obj).ichar = 'L' as i32 as libc::c_short;
	(*obj).is_cursed = 0 as libc::c_int as libc::c_short;
	(*obj).picked_up = (*obj).is_cursed;
	(*obj).in_use_flags = 0 as libc::c_int as libc::c_ushort;
	(*obj).identified = 0 as libc::c_int as libc::c_ushort as libc::c_short;
	(*obj).damage = "1d1";
	return obj;
}

pub unsafe fn free_object(obj: *mut object) {
	(*obj).next_object = free_list;
	free_list = obj;
}

#[no_mangle]
pub unsafe extern "C" fn show_objects() {
	let mut obj: *mut object = 0 as *mut object;
	let mut mc: libc::c_short = 0;
	let mut rc: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut monster: *mut object = 0 as *mut object;
	obj = level_objects.next_object;
	while !obj.is_null() {
		row = (*obj).row;
		col = (*obj).col;
		rc = get_mask_char((*obj).what_is as libc::c_int) as libc::c_short;
		if dungeon[row as usize][col as usize] as libc::c_int
			& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
		{
			monster = object_at(
				&mut level_monsters,
				row as libc::c_int,
				col as libc::c_int,
			);
			if !monster.is_null() {
				(*monster).d_enchant = rc;
			}
		}
		mc = (if ncurses::wmove(ncurses::stdscr(), row as libc::c_int, col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int) as ncurses::chtype
		} else {
			ncurses::winch(ncurses::stdscr())
		}) as libc::c_short;
		if ((mc as libc::c_int) < 'A' as i32 || mc as libc::c_int > 'Z' as i32)
			&& (row as libc::c_int != rogue.row as libc::c_int
			|| col as libc::c_int != rogue.col as libc::c_int)
		{
			if ncurses::wmove(ncurses::stdscr(), row as libc::c_int, col as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				addch(rc as ncurses::chtype);
			};
		}
		obj = (*obj).next_object;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
			if ncurses::wmove(
				ncurses::stdscr(),
				(*monster).row as libc::c_int,
				(*monster).col as libc::c_int,
			) == -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				addch((*monster).what_is as libc::c_int as ncurses::chtype);
			};
		}
		monster = (*monster).next_object;
	}
}

#[no_mangle]
pub unsafe extern "C" fn put_amulet() -> libc::c_int {
	let mut obj: *mut object = 0 as *mut object;
	obj = alloc_object();
	(*obj).what_is = 0o400 as libc::c_int as libc::c_ushort;
	rand_place(obj);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn new_object_for_wizard() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut max: libc::c_short = 0;
	let mut wk: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut buf: [libc::c_char; 80] = [0; 80];
	if pack_count(0 as *mut object) >= 24 as libc::c_int {
		message(b"pack full\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
		return;
	}
	message(b"type of object?\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
	loop {
		ch = rgetchar() as libc::c_short;
		if !(r_index(
			b"!?:)]=/,\x1B\0" as *const u8 as *const libc::c_char,
			ch as libc::c_int,
			0 as libc::c_int,
		) == -(1 as libc::c_int))
		{
			break;
		}
		sound_bell();
	}
	check_message();
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = alloc_object();
	match ch as libc::c_int {
		33 => {
			(*obj).what_is = 0o10 as libc::c_int as libc::c_ushort;
			max = (14 as libc::c_int - 1 as libc::c_int) as libc::c_short;
		}
		63 => {
			(*obj).what_is = 0o4 as libc::c_int as libc::c_ushort;
			max = (12 as libc::c_int - 1 as libc::c_int) as libc::c_short;
		}
		44 => {
			(*obj).what_is = 0o400 as libc::c_int as libc::c_ushort;
		}
		58 => {
			get_food(&mut *obj, false);
		}
		41 => {
			gr_weapon(obj, 0 as libc::c_int);
			max = (8 as libc::c_int - 1 as libc::c_int) as libc::c_short;
		}
		93 => {
			gr_armor(obj);
			max = (7 as libc::c_int - 1 as libc::c_int) as libc::c_short;
		}
		47 => {
			gr_wand(obj);
			max = (10 as libc::c_int - 1 as libc::c_int) as libc::c_short;
		}
		61 => {
			max = (11 as libc::c_int - 1 as libc::c_int) as libc::c_short;
			(*obj).what_is = 0o200 as libc::c_int as libc::c_ushort;
		}
		_ => {}
	}
	if ch as libc::c_int != ',' as i32 && ch as libc::c_int != ':' as i32 {
		's_185: {
			loop {
				if get_input_line(
					b"which kind?\0" as *const u8 as *const libc::c_char,
					b"\0" as *const u8 as *const libc::c_char,
					buf.as_mut_ptr(),
					b"\0" as *const u8 as *const libc::c_char,
					0 as libc::c_int,
					1 as libc::c_int,
				) != 0
				{
					wk = get_number(buf.as_mut_ptr()) as libc::c_short;
					if wk as libc::c_int >= 0 as libc::c_int
						&& wk as libc::c_int <= max as libc::c_int
					{
						(*obj).which_kind = wk as libc::c_ushort;
						if (*obj).what_is as libc::c_int
							== 0o200 as libc::c_int as libc::c_ushort as libc::c_int
						{
							gr_ring(obj, 0 as libc::c_int);
						}
						break 's_185;
					} else {
						sound_bell();
					}
				} else {
					free_object(obj);
					return;
				}
			}
		}
	}
	get_desc(obj, buf.as_mut_ptr());
	message(buf.as_mut_ptr(), 0 as libc::c_int);
	add_to_pack(obj, &mut rogue.pack, 1 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}
