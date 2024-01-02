#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_int, c_short};
use ncurses::chtype;
use crate::message::message;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::gr_row_col;

pub mod flags;

use crate::prelude::*;
pub use flags::MonsterFlags;
use SpotFlag::{Door, Monster};
use crate::{objects, odds, pack};
use crate::prelude::flags::MONSTERS;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::ObjectWhat::Scroll;
use crate::prelude::SpotFlag::{Floor, Object, Stairs, Tunnel};

#[derive(Clone)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: isize,
	pub hp_max: isize,
	pub str_current: isize,
	pub str_max: isize,
	pub pack: object,
	pub gold: isize,
	pub exp: isize,
	pub exp_points: isize,
	pub row: i64,
	pub col: i64,
	pub fchar: char,
	pub moves_left: i16,
}

pub type fighter = fight;

pub static mut level_monsters: object = obj::default();
pub static mut mon_disappeared: bool = false;
pub static mut m_names: [&'static str; 26] = [
	"aquator",
	"bat",
	"centaur",
	"dragon",
	"emu",
	"venus fly-trap",
	"griffin",
	"hobgoblin",
	"ice monster",
	"jabberwock",
	"kestrel",
	"leprechaun",
	"medusa",
	"nymph",
	"orc",
	"phantom",
	"quagga",
	"rattlesnake",
	"snake",
	"troll",
	"black unicorn",
	"vampire",
	"wraith",
	"xeroc",
	"yeti",
	"zombie",
];
#[no_mangle]
pub static mut mon_tab: [object; 26] = [
	{
		let mut init = obj {
			m_flags: MonsterFlags::a(),
			damage: "0d0".to_string(),
			quantity: 25,
			ichar: 'A',
			kill_exp: 20,
			is_protected: 9,
			is_cursed: 18,
			class: 100,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::b(),
			damage: "1d3".to_string(),
			quantity: 10,
			ichar: 'B',
			kill_exp: 2,
			is_protected: 1,
			is_cursed: 8,
			class: 60,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::c(),
			damage: "3d3/2d5".to_string(),
			quantity: 32,
			ichar: 'C',
			kill_exp: 15,
			is_protected: 7,
			is_cursed: 16,
			class: 85,
			identified: false,
			stationary_damage: 0,
			which_kind: 10,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::d(),
			damage: "4d6/4d9".to_string(),
			quantity: 145,
			ichar: 'D',
			kill_exp: 5000,
			is_protected: 21,
			is_cursed: 126,
			class: 100,
			identified: false,
			stationary_damage: 0,
			which_kind:
			90,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::e(),
			damage: "1d3".to_string(),
			quantity: 11,
			ichar: 'E',
			kill_exp: 2,
			is_protected: 1,
			is_cursed: 7,
			class: 65,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::f(),
			damage: "5d5".to_string(),
			quantity: 73,
			ichar: 'F',
			kill_exp: 91,
			is_protected: 12,
			is_cursed: 126,
			class: 80,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::g(),
			damage: "5d5/5d5".to_string(),
			quantity: 115,
			ichar: 'G',
			kill_exp: 2000,
			is_protected: 20,
			is_cursed: 126,
			class: 85,
			identified: false,
			stationary_damage: 0,
			which_kind:
			10,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::h(),
			damage: "1d3/1d2".to_string(),
			quantity: 15,
			ichar: 'H',
			kill_exp: 3,
			is_protected: 1,
			is_cursed: 10,
			class: 67,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::i(),
			damage: "0d0".to_string(),
			quantity: 15,
			ichar: 'I',
			kill_exp: 5,
			is_protected: 2,
			is_cursed: 11,
			class: 68,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::j(),
			damage: "3d10/4d5".to_string(),
			quantity: 132,
			ichar: 'J',
			kill_exp: 3000,
			is_protected: 21,
			is_cursed: 126,
			class: 100,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::k(),
			damage: "1d4".to_string(),
			quantity: 10,
			ichar: 'K',
			kill_exp: 2,
			is_protected: 1,
			is_cursed: 6,
			class: 60,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::l(),
			damage: "0d0".to_string(),
			quantity: 25,
			ichar: 'L',
			kill_exp: 21,
			is_protected: 6,
			is_cursed: 16,
			class: 75,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::m(),
			damage: "4d4/3d7".to_string(),
			quantity: 97,
			ichar: 'M',
			kill_exp: 250,
			is_protected: 18,
			is_cursed: 126,
			class: 85,
			identified: false,
			stationary_damage: 0,
			which_kind:
			25,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::n(),
			damage: "0d0".to_string(),
			quantity: 25,
			ichar: 'N',
			kill_exp: 39,
			is_protected: 10,
			is_cursed: 19,
			class: 75,
			identified: false,
			stationary_damage: 0,
			which_kind:
			100,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::o(),
			damage: "1d6".to_string(),
			quantity: 25,
			ichar: 'O',
			kill_exp: 5,
			is_protected: 4,
			is_cursed: 13,
			class: 70,
			identified: false,
			stationary_damage: 0,
			which_kind:
			10,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::p(),
			damage: "5d4".to_string(),
			quantity: 76,
			ichar: 'P',
			kill_exp: 120,
			is_protected: 15,
			is_cursed: 24,
			class: 80,
			identified: false,
			stationary_damage: 0,
			which_kind:
			50,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::q(),
			damage: "3d5".to_string(),
			quantity: 30,
			ichar: 'Q',
			kill_exp: 20,
			is_protected: 8,
			is_cursed: 17,
			class: 78,
			identified: false,
			stationary_damage: 0,
			which_kind:
			20,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::r(),
			damage: "2d5".to_string(),
			quantity: 19,
			ichar: 'R',
			kill_exp: 10,
			is_protected: 3,
			is_cursed: 12,
			class: 70,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::s(),
			damage: "1d3".to_string(),
			quantity: 8,
			ichar: 'S',
			kill_exp: 2,
			is_protected: 1,
			is_cursed: 9,
			class: 50,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::t(),
			damage: "4d6/1d4".to_string(),
			quantity: 75,
			ichar: 'T',
			kill_exp: 125,
			is_protected: 13,
			is_cursed: 22,
			class: 75,
			identified: false,
			stationary_damage: 0,
			which_kind:
			33,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::u(),
			damage: "4d10".to_string(),
			quantity: 90,
			ichar: 'U',
			kill_exp: 200,
			is_protected: 17,
			is_cursed: 26,
			class: 85,
			identified: false,
			stationary_damage: 0,
			which_kind:
			33,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::v(),
			damage: "1d14/1d4".to_string(),
			quantity: 55,
			ichar: 'V',
			kill_exp: 350,
			is_protected: 19,
			is_cursed: 126,
			class: 85,
			identified: false,
			stationary_damage: 0,
			which_kind:
			18,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::w(),
			damage: "2d8".to_string(),
			quantity: 45,
			ichar: 'W',
			kill_exp: 55,
			is_protected: 14,
			is_cursed: 23,
			class: 75,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::x(),
			damage: "4d6".to_string(),
			quantity: 42,
			ichar: 'X',
			kill_exp: 110,
			is_protected: 16,
			is_cursed: 25,
			class: 75,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::y(),
			damage: "3d6".to_string(),
			quantity: 35,
			ichar: 'Y',
			kill_exp: 50,
			is_protected: 11,
			is_cursed: 20,
			class: 80,
			identified: false,
			stationary_damage: 0,
			which_kind:
			20,
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
	{
		let mut init = obj {
			m_flags: MonsterFlags::z(),
			damage: "1d7".to_string(),
			quantity: 21,
			ichar: 'Z',
			kill_exp: 8,
			is_protected: 5,
			is_cursed: 14,
			class: 69,
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
];

#[no_mangle]
pub unsafe extern "C" fn put_mons() {
	let n = get_rand(4, 6);
	for _i in 0..n {
		let mut monster = gr_monster(0 as *mut object, 0);
		if (*monster).m_flags.wanders && coin_toss() {
			wake_up(&mut *monster);
		}
		let mut row = 0;
		let mut col = 0;
		gr_row_col(&mut row, &mut col, vec![Floor, Tunnel, Stairs, Object]);
		put_m_at(row, col, &mut *monster);
	}
}

#[no_mangle]
pub unsafe extern "C" fn gr_monster(
	mut monster: *mut object,
	mut mn: c_int,
) -> *mut object {
	if monster.is_null() {
		monster = alloc_object();
		loop {
			mn = get_rand(0 as c_int, 26 as c_int - 1 as c_int);
			if cur_level as c_int
				>= mon_tab[mn as usize].is_protected as c_int
				&& cur_level as c_int
				<= mon_tab[mn as usize].is_cursed as c_int
			{
				break;
			}
		}
	}
	*monster = mon_tab[mn as usize];
	if (*monster).m_flags.imitates {
		(*monster).disguise = gr_obj_char() as libc::c_ushort;
	}
	if cur_level as c_int > 26 as c_int + 2 as c_int {
		(*monster).m_flags.hasted = true;
	}
	(*monster).trow = -1;
	return monster;
}

#[no_mangle]
pub unsafe extern "C" fn mv_mons() {
	if haste_self as c_int % 2 as c_int != 0 {
		return;
	}
	let mut monster: *mut object = level_monsters.next_object;
	while !monster.is_null() {
		let mut done_with_monster = false;
		let next_monster = (*monster).next_object;
		if (*monster).m_flags.hasted {
			mon_disappeared = false;
			mv_monster(&mut *monster, rogue.row, rogue.col);
			if mon_disappeared {
				done_with_monster = true;
			}
		} else if (*monster).m_flags.slowed {
			(*monster).flip_slowed_toggle();
			if (*monster).slowed_toggle() {
				done_with_monster = true;
			}
		}
		if !done_with_monster && (*monster).m_flags.confused && move_confused(&mut *monster) {
			done_with_monster = true;
		}
		if !done_with_monster {
			let mut flew = false;
			if (*monster).m_flags.flies && !(*monster).m_flags.napping && !mon_can_go(&*monster, rogue.row, rogue.col) {
				flew = true;
				mv_monster(&mut *monster, rogue.row, rogue.col);
			}
			if !flew || !mon_can_go(&*monster, rogue.row, rogue.col) {
				mv_monster(&mut *monster, rogue.row, rogue.col);
			}
		}
		monster = next_monster;
	}
}

#[no_mangle]
pub unsafe extern "C" fn party_monsters(rn: i64, n: i64) {
	for i in 0..MONSTERS {
		mon_tab[i].set_first_level(mon_tab[i].first_level() - (cur_level % 3))
	}
	let n = n + n;
	for _i in 0..n {
		if no_room_for_monster(rn as usize) {
			break;
		}
		let mut found: Option<(i64, i64)> = None;
		for _j in 0..250 {
			let row = get_rand(rooms[rn as usize].top_row + 1, rooms[rn as usize].bottom_row - 1);
			let col = get_rand(rooms[rn as usize].left_col + 1, rooms[rn as usize].right_col - 1);
			let dungeon_spot = dungeon[row as usize][col as usize];
			if !Monster.is_set(dungeon_spot) && SpotFlag::is_any_set(&vec![Floor, Tunnel], dungeon_spot) {
				found = Some((row, col));
				break;
			}
		}
		if let Some((row, col)) = found {
			let monster = gr_monster(0 as *mut object, 0);
			if !(*monster).m_flags.imitates {
				(*monster).m_flags.wakens = true;
			}
			put_m_at(row, col, &mut *monster);
		}
	}
	for i in 0..MONSTERS {
		mon_tab[i].set_first_level(mon_tab[i].first_level() + (cur_level % 3))
	}
}

#[no_mangle]
pub unsafe extern "C" fn gmc_row_col(row: i64, col: i64) -> chtype {
	let monster = objects::object_at(&level_monsters, row, col);
	if !monster.is_null() {
		let invisible = (*monster).m_flags.invisible;
		let bypass_invisible = detect_monster || see_invisible || r_see_invisible;
		if (invisible && !bypass_invisible) || (blind != 0) {
			(*monster).trail_char()
		} else {
			if (*monster).m_flags.imitates {
				(*monster).disguise()
			} else {
				(*monster).m_char()
			}
		}
	} else {
		ncurses::chtype::from('&')
	}
}

#[no_mangle]
pub unsafe extern "C" fn gmc(mut monster: *mut object) -> chtype {
	let defeat_invisibility = detect_monster || see_invisible || r_see_invisible;
	if ((*monster).m_flags.invisible && !defeat_invisibility) || (blind != 0) {
		(*monster).trail_char()
	} else if (*monster).m_flags.imitates {
		(*monster).disguise()
	} else {
		(*monster).m_char()
	}
}

pub unsafe fn mv_monster(monster: &mut object, row: i64, col: i64) {
	if monster.m_flags.asleep {
		if monster.m_flags.napping {
			monster.decrement_nap();
			return;
		}
		if (monster.m_flags.wakens)
			&& rogue_is_around(monster.row, monster.col)
			&& rand_percent(if stealthy > 0 { odds::WAKE_PERCENT / (odds::STEALTH_FACTOR + (stealthy as usize)) } else { odds::WAKE_PERCENT })
		{
			wake_up(monster);
		}
		return;
	} else if monster.m_flags.already_moved {
		monster.m_flags.already_moved = false;
		return;
	}
	if monster.m_flags.flits && flit(monster) {
		return;
	}
	if monster.m_flags.stationary && !mon_can_go(monster, rogue.row, rogue.col) {
		return;
	}
	if monster.m_flags.freezing_rogue {
		return;
	}
	if monster.m_flags.confuses && m_confuse(monster) {
		return;
	}
	if mon_can_go(monster, rogue.row, rogue.col) {
		mon_hit(monster, None, false);
		return;
	}
	if monster.m_flags.flames && flame_broil(monster) {
		return;
	}
	if monster.m_flags.seeks_gold && seek_gold(monster) {
		return;
	}

	let (mut row, mut col) = (row, col);
	if monster.trow == monster.row && monster.tcol == monster.col {
		monster.trow = NO_ROOM;
	} else if monster.trow != NO_ROOM {
		row = monster.trow;
		col = monster.tcol;
	}
	if monster.row > row {
		row = monster.row - 1;
	} else if monster.row < row {
		row = monster.row + 1;
	}
	if Door.is_set(dungeon[row as usize][monster.col as usize]) && mtry(monster, row, monster.col) {
		return;
	}
	if monster.col > col {
		col = monster.col - 1;
	} else if monster.col < col {
		col = monster.col + 1;
	}
	if Door.is_set(dungeon[monster.row as usize][col as usize]) && mtry(monster, monster.row, col) {
		return;
	}
	if mtry(monster, row, col) {
		return;
	}

	{
		let mut tried: [bool; 6] = [false; 6];
		'moved: for _i in 0..6 {
			loop {
				let n = get_rand(0, 5) as usize;
				if !tried[n] {
					match n {
						0 => if mtry(monster, row, monster.col - 1) { break 'moved; }
						1 => if mtry(monster, row, monster.col) { break 'moved; }
						2 => if mtry(monster, row, monster.col + 1) { break 'moved; }
						3 => if mtry(monster, monster.row - 1, col) { break 'moved; }
						4 => if mtry(monster, monster.row, col) { break 'moved; }
						5 => if mtry(monster, monster.row + 1, col) { break 'moved; }
						_ => unreachable!("0 <= n  <= 5")
					}
					tried[n] = true;
					break;
				} else {
					// Repeat until we find an untried n.
					// FUTURE This code is silly. Should generate a random order ahead of time instead of looping.
				}
			}
		}
	}

	if monster.row == monster.o_row && monster.col == monster.o_col {
		monster.o += 1;
		if monster.o > 4 {
			if monster.trow == NO_ROOM && !mon_sees(monster, rogue.row, rogue.col) {
				monster.trow = get_rand(1, (DROWS - 2) as i64);
				monster.tcol = get_rand(0, (DCOLS - 1) as i64);
			} else {
				monster.trow = NO_ROOM;
				monster.o = 0;
			}
		}
	} else {
		monster.o_row = monster.row;
		monster.o_col = monster.col;
		monster.o = 0;
	}
}

pub unsafe fn mtry(monster: &mut object, row: i64, col: i64) -> bool {
	if mon_can_go(monster, row, col) {
		move_mon_to(monster, row, col);
		return true;
	}
	return false;
}

pub unsafe fn move_mon_to(monster: &mut object, row: i64, col: i64) {
	Monster.clear(&mut dungeon[monster.row as usize][monster.col as usize]);
	Monster.set(&mut dungeon[row as usize][col as usize]);
	let c = ncurses::mvinch((monster.row as usize) as i32, (monster.col as usize) as i32);
	if (c >= chtype::from('A')) && (c <= chtype::from('Z'))
	{
		let (mrow, mcol) = ((monster.row as usize) as i32, (monster.col as usize) as i32);
		let no_detect_monster = !detect_monster;
		if no_detect_monster {
			ncurses::mvaddch(mrow, mcol, monster.trail_char());
		} else {
			if rogue_can_see(mrow as i64, mcol as i64) {
				ncurses::mvaddch(mrow, mcol, monster.trail_char());
			} else {
				if monster.trail_char() == chtype::from('.') {
					monster.set_trail_char(chtype::from(' '));
				}
				ncurses::mvaddch(mrow, mcol, monster.trail_char());
			}
		}
	}
	monster.set_trail_char(ncurses::mvinch(row as i32, col as i32));
	if blind == 0 && ((detect_monster) || rogue_can_see(row, col)) {
		let bypass_invisibility = (detect_monster) || (see_invisible) || (r_see_invisible);
		if !monster.m_flags.invisible || bypass_invisibility {
			ncurses::mvaddch(row as i32, col as i32, gmc(monster));
		}
	}
	if Door.is_set(dungeon[row as usize][col as usize])
		&& (get_room_number(row, col) != cur_room)
		&& Floor.is_only(dungeon[monster.row as usize][monster.col as usize])
		&& blind == 0 {
		ncurses::mvaddch((monster.row as usize) as i32, (monster.col as usize) as i32, chtype::from(' '));
	}
	if Door.is_set(dungeon[row as usize][col as usize]) {
		let entering = Tunnel.is_set(dungeon[monster.row as usize][monster.col as usize]);
		dr_course(monster, entering, row, col);
	} else {
		monster.row = row;
		monster.col = col;
	}
}

pub unsafe fn mon_can_go(monster: &obj, row: i64, col: i64) -> bool {
	let dr = monster.row as isize - row as isize;        /* check if move distance > 1 */
	if (dr >= 2) || (dr <= -2) {
		return false;
	}
	let dc = monster.col as isize - col as isize;
	if (dc >= 2) || (dc <= -2) {
		return false;
	}
	if SpotFlag::Nothing.is_set(dungeon[monster.row as usize][col as usize]) || SpotFlag::Nothing.is_set(dungeon[row as usize][monster.col as usize]) {
		return false;
	}
	if !is_passable(row as c_int, col as c_int) || Monster.is_set(dungeon[row as usize][col as usize]) {
		return false;
	}
	if (monster.row != row) && (monster.col != col)
		&& (Door.is_set(dungeon[row as usize][col as usize]) || Door.is_set(dungeon[monster.row as usize][monster.col as usize])) {
		return false;
	}
	if (monster.trow == NO_ROOM)
		&& !monster.m_flags.flits
		&& !monster.m_flags.confused
		&& !monster.m_flags.can_flit
	{
		if (monster.row < rogue.row) && (row < monster.row) { return false; }
		if (monster.row > rogue.row) && (row > monster.row) { return false; }
		if (monster.col < rogue.col) && (col < monster.col) { return false; }
		if (monster.col > rogue.col) && (col > monster.col) { return false; }
	}
	if Object.is_set(dungeon[row as usize][col as usize]) {
		let obj = objects::object_at(&level_objects, row, col);
		if (*obj).what_is == Scroll && (*obj).which_kind == scroll_kind::SCARE_MONSTER {
			return false;
		}
	}
	return true;
}

pub fn wake_up(monster: &mut object) {
	monster.m_flags.wake_up();
}

#[no_mangle]
pub unsafe extern "C" fn wake_room(rn: i64, entering: bool, row: i64, col: i64) {
	let wake_percent = {
		let wake_percent = if rn == party_room { odds::PARTY_WAKE_PERCENT } else { odds::WAKE_PERCENT };
		if stealthy > 0 {
			wake_percent / (odds::STEALTH_FACTOR + stealthy as usize)
		} else {
			wake_percent
		}
	};
	let mut monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).in_room(rn) {
			if entering {
				(*monster).trow = NO_ROOM;
			} else {
				(*monster).trow = row;
				(*monster).tcol = col;
			}
		}
		if (*monster).m_flags.wakens && (*monster).in_room(rn) {
			if rand_percent(wake_percent) {
				wake_up(&mut *monster);
			}
		}
		monster = (*monster).next_object;
	}
}

pub unsafe fn mon_name(monster: &object) -> &'static str {
	if player_is_blind() || (monster.m_flags.invisible && !bypass_invisibility()) {
		"something"
	} else if player_hallucinating() {
		m_names[get_rand(0, m_names.len() - 1)]
	} else {
		mon_real_name(monster)
	}
}

pub unsafe fn mon_real_name(monster: &obj) -> &'static str {
	let index = monster.m_char() as usize - 'A' as usize;
	m_names[index]
}

pub unsafe fn player_hallucinating() -> bool { halluc != 0 }

pub unsafe fn player_is_blind() -> bool { blind != 0 }

pub unsafe fn bypass_invisibility() -> bool { detect_monster || see_invisible || r_see_invisible }

pub unsafe fn rogue_is_around(row: i64, col: i64) -> bool {
	let rdif = row - rogue.row;
	let cdif = col - rogue.col;
	(rdif >= -1) && (rdif <= 1) && (cdif >= -1) && (cdif <= 1)
}

#[no_mangle]
pub unsafe extern "C" fn wanderer() {
	let mut monster: *mut object = 0 as *mut object;
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	let mut found = false;
	{
		let mut i: c_short = 0;
		while i < 15 && !found {
			monster = gr_monster(0 as *mut object, 0 as c_int);
			let monster_wanders_or_wakens = (*monster).m_flags.wakens || (*monster).m_flags.wanders;
			if monster_wanders_or_wakens {
				found = true;
			} else {
				free_object(monster);
			}
			i += 1;
		}
	}
	if found {
		found = false;
		wake_up(&mut *monster);
		let mut i = 0;
		while i < 25 && !found {
			gr_row_col(&mut row, &mut col, vec![Floor, Tunnel, Stairs, Object]);
			if rogue_can_see(row, col) == false {
				put_m_at(row, col, &mut *monster);
				found = true;
			}
			i += 1;
		}
		if !found {
			free_object(monster);
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn show_monsters() {
	detect_monster = true;
	if blind != 0 {
		return;
	}
	let mut monster: *mut object = level_monsters.next_object;
	while !monster.is_null() {
		ncurses::mvaddch((*monster).row as i32, (*monster).col as i32, (*monster).m_char());
		if (*monster).m_flags.imitates {
			(*monster).m_flags.imitates = false;
			(*monster).m_flags.wakens = true;
		}
		monster = (*monster).next_object;
	}
}

#[no_mangle]
pub unsafe extern "C" fn create_monster() {
	let mut found = false;
	let mut row = rogue.row;
	let mut col = rogue.col;
	for i in 0..9 {
		{
			let (r_moved, c_moved) = rand_around(i, row, col);
			row = r_moved;
			col = c_moved;
		}
		let on_rogue = row == rogue.row && col == rogue.col;
		let out_of_bounds = row < MIN_ROW || row > (DROWS - 2) as i64 || col < 0 || col > (DCOLS - 1) as i64;
		if on_rogue || out_of_bounds {
			continue;
		}
		let spot_moved = dungeon[row as usize][col as usize];
		if !Monster.is_set(spot_moved) && SpotFlag::is_any_set(&vec![Floor, Tunnel, Stairs, Door], spot_moved) {
			found = true;
			break;
		}
	}
	if found {
		let mut monster = gr_monster(0 as *mut object, 0);
		put_m_at(row, col, &mut *monster);
		ncurses::mvaddch(row as i32, col as i32, gmc(monster));
		if (*monster).m_flags.wanders || (*monster).m_flags.wakens {
			wake_up(&mut *monster);
		}
	} else {
		message("you hear a faint cry of anguish in the distance", 0);
	}
}

pub unsafe fn put_m_at(row: i64, col: i64, monster: &mut object) {
	monster.row = row;
	monster.col = col;
	Monster.set(&mut dungeon[row as usize][col as usize]);
	monster.set_trail_char(ncurses::mvinch(row as i32, col as i32));
	pack::add_to_pack(monster, &mut level_monsters, 0);
	aim_monster(monster);
}

pub unsafe fn rogue_can_see(row: i64, col: i64) -> bool {
	let in_current_room = get_room_number(row, col) == cur_room;
	let not_in_maze = rooms[cur_room as usize].room_type != RoomType::Maze;
	let is_very_close = rogue_is_around(row, col);
	blind == 0 && ((in_current_room && not_in_maze) || is_very_close)
}

pub unsafe fn move_confused(monster: &mut object) -> bool {
	if !monster.m_flags.asleep {
		monster.decrement_moves_confused();
		if monster.m_flags.stationary {
			return if coin_toss() { true } else { false };
		} else if rand_percent(15) {
			return true;
		} else {
			let mut row = monster.row;
			let mut col = monster.col;
			for i in 0..9 {
				{
					let (r_moved, c_moved) = rand_around(i, row, col);
					row = r_moved;
					col = c_moved;
				}
				let on_rogue = row == rogue.row && col == rogue.col;
				if on_rogue {
					return false;
				}
				if mtry(monster, row, col) {
					return true;
				}
			}
		}
	}
	return false;
}

pub unsafe fn flit(monster: &mut object) -> bool {
	if !rand_percent(odds::FLIT_PERCENT) {
		return false;
	}
	if rand_percent(10) {
		return true;
	}
	let mut row = monster.row;
	let mut col = monster.col;
	for i in 0..9 {
		{
			let (r_moved, c_moved) = rand_around(i, row, col);
			row = r_moved;
			col = c_moved;
		}
		let on_rogue = row == rogue.row && col == rogue.col;
		if on_rogue {
			continue;
		}
		if mtry(monster, row, col) {
			return true;
		}
	}
	return true;
}

pub unsafe fn gr_obj_char() -> u16 {
	const rs: &str = "%!?]=/):*";
	let r = get_rand(0, 8) as usize;
	rs.as_bytes()[r] as u16
}

pub unsafe fn aim_monster(monster: &mut object) {
	let rn = get_room_number(monster.row, monster.col) as usize;
	let r = get_rand(0, 12);

	for i in 0..4 {
		let d = ((r + i) % 4) as usize;
		if rooms[rn].doors[d].oth_room.is_some() {
			monster.trow = rooms[rn].doors[d].door_row;
			monster.tcol = rooms[rn].doors[d].door_col;
			break;
		}
	}
}

pub unsafe fn no_room_for_monster(rn: usize) -> bool {
	let room = &rooms[rn];
	for i in (room.top_row + 1)..room.bottom_row {
		for j in (room.left_col + 1)..room.right_col {
			if !Monster.is_set(dungeon[i as usize][j as usize]) {
				// Found a spot for the monster
				return false;
			}
		}
	}
	return true;
}

#[no_mangle]
pub unsafe extern "C" fn aggravate() {
	let mut monster: *mut object = 0 as *mut object;
	message("you hear a high pitched humming noise", 0);
	monster = level_monsters.next_object;
	while !monster.is_null() {
		wake_up(&mut *monster);
		(*monster).m_flags.imitates = false;
		if rogue_can_see((*monster).row, (*monster).col) {
			ncurses::mvaddch((*monster).row as i32, (*monster).col as i32, (*monster).m_char());
		}
		monster = (*monster).next_object;
	}
}

#[no_mangle]
pub unsafe extern "C" fn mon_sees(
	mut monster: *mut object,
	mut row: i64,
	mut col: i64,
) -> bool {
	let mut rn: i64 = 0;
	let mut rdif: c_short = 0;
	let mut cdif: c_short = 0;
	rn = get_room_number(row, col);
	if rn != -1
		&& rn == get_room_number((*monster).row, (*monster).col)
		&& (*rooms.as_mut_ptr().offset(rn as isize)).room_type as i64 & 0o4i64 == 0 {
		return true;
	}
	rdif = (row - (*monster).row) as c_short;
	cdif = (col - (*monster).col) as c_short;
	return rdif as c_int >= -(1 as c_int)
		&& rdif as c_int <= 1 as c_int
		&& cdif as c_int >= -(1 as c_int)
		&& cdif as c_int <= 1 as c_int;
}

#[no_mangle]
pub unsafe extern "C" fn mv_aquatars() {
	let mut monster: *mut object = 0 as *mut object;
	monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).m_char() == chtype::from('A') && mon_can_go(&*monster, rogue.row, rogue.col) {
			mv_monster(&mut *monster, rogue.row, rogue.col);
			(*monster).m_flags.already_moved = true;
		}
		monster = (*monster).next_object;
	}
}
