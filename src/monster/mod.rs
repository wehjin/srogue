#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_int, c_short};
use crate::message::message;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::gr_row_col;

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: ncurses::chtype) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut rooms: [room; 0];
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	fn add_to_pack() -> *mut object;
	fn alloc_object() -> *mut object;
	fn object_at() -> *mut object;
	static mut cur_level: libc::c_short;
	static mut cur_room: libc::c_short;
	static mut party_room: libc::c_short;
	static mut blind: libc::c_short;
	static mut halluc: libc::c_short;
	static mut haste_self: libc::c_short;
	static mut detect_monster: libc::c_char;
	static mut see_invisible: libc::c_char;
	static mut r_see_invisible: libc::c_char;
	static mut stealthy: libc::c_short;
}

pub mod flags;

use crate::prelude::*;
pub use flags::MonsterFlags;
use SpotFlag::{Door, Monster};
use crate::{objects, odds, pack};
use crate::prelude::flags::MONSTERS;
use crate::prelude::SpotFlag::{Floor, Object, Stairs, Tunnel};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: libc::c_short,
	pub _curx: libc::c_short,
	pub _maxy: libc::c_short,
	pub _maxx: libc::c_short,
	pub _begy: libc::c_short,
	pub _begx: libc::c_short,
	pub _flags: libc::c_short,
	pub _attrs: attr_t,
	pub _bkgd: chtype,
	pub _notimeout: libc::c_int,
	pub _clear: libc::c_int,
	pub _leaveok: libc::c_int,
	pub _scroll: libc::c_int,
	pub _idlok: libc::c_int,
	pub _idcok: libc::c_int,
	pub _immed: libc::c_int,
	pub _sync: libc::c_int,
	pub _use_keypad: libc::c_int,
	pub _delay: libc::c_int,
	pub _line: *mut ldat,
	pub _regtop: libc::c_short,
	pub _regbottom: libc::c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: libc::c_short,
}

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
pub type attr_t = chtype;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: libc::c_short,
	pub hp_max: libc::c_short,
	pub str_current: libc::c_short,
	pub str_max: libc::c_short,
	pub pack: object,
	pub gold: libc::c_long,
	pub exp: libc::c_short,
	pub exp_points: libc::c_long,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub fchar: libc::c_short,
	pub moves_left: libc::c_short,
}

pub type fighter = fight;

#[no_mangle]
pub static mut level_monsters: object = obj {
	m_flags: MonsterFlags {
		hasted: false,
		slowed: false,
		invisible: false,
		asleep: false,
		wakens: false,
		wanders: false,
		flies: false,
		flits: false,
		can_flit: false,
		confused: false,
		rusts: false,
		holds: false,
		freezes: false,
		steals_gold: false,
		steals_item: false,
		stings: false,
		drains_life: false,
		drops_level: false,
		seeks_gold: false,
		freezing_rogue: false,
		RUST_VANISHED: false,
		confuses: false,
		imitates: false,
		flames: false,
		stationary: false,
		napping: false,
		already_moved: false,
	},
	damage: "",
	quantity: 0,
	ichar: 0,
	kill_exp: 0,
	is_protected: 0,
	is_cursed: 0,
	class: 0,
	identified: 0,
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
	what_is: 0,
	picked_up: 0,
	in_use_flags: 0,
	next_object: 0 as *const obj as *mut obj,
};
#[no_mangle]
pub static mut mon_disappeared: libc::c_char = 0;
#[no_mangle]
pub static mut m_names: [*mut libc::c_char; 26] = [
	b"aquator\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"bat\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"centaur\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"dragon\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"emu\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"venus fly-trap\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"griffin\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"hobgoblin\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"ice monster\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"jabberwock\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"kestrel\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"leprechaun\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"medusa\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"nymph\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"orc\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"phantom\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"quagga\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"rattlesnake\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"snake\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"troll\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"black unicorn\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"vampire\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"wraith\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"xeroc\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"yeti\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
	b"zombie\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
];
#[no_mangle]
pub static mut mon_tab: [object; 26] = [
	{
		let mut init = obj {
			m_flags: MonsterFlags::a(),
			damage: "0d0",
			quantity: 25 as libc::c_int as libc::c_short,
			ichar: 'A' as i32 as libc::c_short,
			kill_exp: 20 as libc::c_int as libc::c_short,
			is_protected: 9 as libc::c_int as libc::c_short,
			is_cursed: 18 as libc::c_int as libc::c_short,
			class: 100 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::b(),
			damage: "1d3",
			quantity: 10 as libc::c_int as libc::c_short,
			ichar: 'B' as i32 as libc::c_short,
			kill_exp: 2 as libc::c_int as libc::c_short,
			is_protected: 1 as libc::c_int as libc::c_short,
			is_cursed: 8 as libc::c_int as libc::c_short,
			class: 60 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::c(),
			damage: "3d3/2d5",
			quantity: 32 as libc::c_int as libc::c_short,
			ichar: 'C' as i32 as libc::c_short,
			kill_exp: 15 as libc::c_int as libc::c_short,
			is_protected: 7 as libc::c_int as libc::c_short,
			is_cursed: 16 as libc::c_int as libc::c_short,
			class: 85 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 10 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::d(),
			damage: "4d6/4d9",
			quantity: 145 as libc::c_int as libc::c_short,
			ichar: 'D' as i32 as libc::c_short,
			kill_exp: 5000 as libc::c_int as libc::c_short,
			is_protected: 21 as libc::c_int as libc::c_short,
			is_cursed: 126 as libc::c_int as libc::c_short,
			class: 100 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 90 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::e(),
			damage: "1d3",
			quantity: 11 as libc::c_int as libc::c_short,
			ichar: 'E' as i32 as libc::c_short,
			kill_exp: 2 as libc::c_int as libc::c_short,
			is_protected: 1 as libc::c_int as libc::c_short,
			is_cursed: 7 as libc::c_int as libc::c_short,
			class: 65 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::f(),
			damage: "5d5",
			quantity: 73 as libc::c_int as libc::c_short,
			ichar: 'F' as i32 as libc::c_short,
			kill_exp: 91 as libc::c_int as libc::c_short,
			is_protected: 12 as libc::c_int as libc::c_short,
			is_cursed: 126 as libc::c_int as libc::c_short,
			class: 80 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::g(),
			damage: "5d5/5d5",
			quantity: 115 as libc::c_int as libc::c_short,
			ichar: 'G' as i32 as libc::c_short,
			kill_exp: 2000 as libc::c_int as libc::c_short,
			is_protected: 20 as libc::c_int as libc::c_short,
			is_cursed: 126 as libc::c_int as libc::c_short,
			class: 85 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 10 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::h(),
			damage: "1d3/1d2",
			quantity: 15 as libc::c_int as libc::c_short,
			ichar: 'H' as i32 as libc::c_short,
			kill_exp: 3 as libc::c_int as libc::c_short,
			is_protected: 1 as libc::c_int as libc::c_short,
			is_cursed: 10 as libc::c_int as libc::c_short,
			class: 67 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::i(),
			damage: "0d0",
			quantity: 15 as libc::c_int as libc::c_short,
			ichar: 'I' as i32 as libc::c_short,
			kill_exp: 5 as libc::c_int as libc::c_short,
			is_protected: 2 as libc::c_int as libc::c_short,
			is_cursed: 11 as libc::c_int as libc::c_short,
			class: 68 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::j(),
			damage: "3d10/4d5",
			quantity: 132 as libc::c_int as libc::c_short,
			ichar: 'J' as i32 as libc::c_short,
			kill_exp: 3000 as libc::c_int as libc::c_short,
			is_protected: 21 as libc::c_int as libc::c_short,
			is_cursed: 126 as libc::c_int as libc::c_short,
			class: 100 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::k(),
			damage: "1d4",
			quantity: 10 as libc::c_int as libc::c_short,
			ichar: 'K' as i32 as libc::c_short,
			kill_exp: 2 as libc::c_int as libc::c_short,
			is_protected: 1 as libc::c_int as libc::c_short,
			is_cursed: 6 as libc::c_int as libc::c_short,
			class: 60 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::l(),
			damage: "0d0",
			quantity: 25 as libc::c_int as libc::c_short,
			ichar: 'L' as i32 as libc::c_short,
			kill_exp: 21 as libc::c_int as libc::c_short,
			is_protected: 6 as libc::c_int as libc::c_short,
			is_cursed: 16 as libc::c_int as libc::c_short,
			class: 75 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::m(),
			damage: "4d4/3d7",
			quantity: 97 as libc::c_int as libc::c_short,
			ichar: 'M' as i32 as libc::c_short,
			kill_exp: 250 as libc::c_int as libc::c_short,
			is_protected: 18 as libc::c_int as libc::c_short,
			is_cursed: 126 as libc::c_int as libc::c_short,
			class: 85 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 25 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::n(),
			damage: "0d0",
			quantity: 25 as libc::c_int as libc::c_short,
			ichar: 'N' as i32 as libc::c_short,
			kill_exp: 39 as libc::c_int as libc::c_short,
			is_protected: 10 as libc::c_int as libc::c_short,
			is_cursed: 19 as libc::c_int as libc::c_short,
			class: 75 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 100 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::o(),
			damage: "1d6",
			quantity: 25 as libc::c_int as libc::c_short,
			ichar: 'O' as i32 as libc::c_short,
			kill_exp: 5 as libc::c_int as libc::c_short,
			is_protected: 4 as libc::c_int as libc::c_short,
			is_cursed: 13 as libc::c_int as libc::c_short,
			class: 70 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 10 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::p(),
			damage: "5d4",
			quantity: 76 as libc::c_int as libc::c_short,
			ichar: 'P' as i32 as libc::c_short,
			kill_exp: 120 as libc::c_int as libc::c_short,
			is_protected: 15 as libc::c_int as libc::c_short,
			is_cursed: 24 as libc::c_int as libc::c_short,
			class: 80 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 50 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::q(),
			damage: "3d5",
			quantity: 30 as libc::c_int as libc::c_short,
			ichar: 'Q' as i32 as libc::c_short,
			kill_exp: 20 as libc::c_int as libc::c_short,
			is_protected: 8 as libc::c_int as libc::c_short,
			is_cursed: 17 as libc::c_int as libc::c_short,
			class: 78 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 20 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::r(),
			damage: "2d5",
			quantity: 19 as libc::c_int as libc::c_short,
			ichar: 'R' as i32 as libc::c_short,
			kill_exp: 10 as libc::c_int as libc::c_short,
			is_protected: 3 as libc::c_int as libc::c_short,
			is_cursed: 12 as libc::c_int as libc::c_short,
			class: 70 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::s(),
			damage: "1d3",
			quantity: 8 as libc::c_int as libc::c_short,
			ichar: 'S' as i32 as libc::c_short,
			kill_exp: 2 as libc::c_int as libc::c_short,
			is_protected: 1 as libc::c_int as libc::c_short,
			is_cursed: 9 as libc::c_int as libc::c_short,
			class: 50 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::t(),
			damage: "4d6/1d4",
			quantity: 75 as libc::c_int as libc::c_short,
			ichar: 'T' as i32 as libc::c_short,
			kill_exp: 125 as libc::c_int as libc::c_short,
			is_protected: 13 as libc::c_int as libc::c_short,
			is_cursed: 22 as libc::c_int as libc::c_short,
			class: 75 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 33 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::u(),
			damage: "4d10",
			quantity: 90 as libc::c_int as libc::c_short,
			ichar: 'U' as i32 as libc::c_short,
			kill_exp: 200 as libc::c_int as libc::c_short,
			is_protected: 17 as libc::c_int as libc::c_short,
			is_cursed: 26 as libc::c_int as libc::c_short,
			class: 85 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 33 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::v(),
			damage: "1d14/1d4",
			quantity: 55 as libc::c_int as libc::c_short,
			ichar: 'V' as i32 as libc::c_short,
			kill_exp: 350 as libc::c_int as libc::c_short,
			is_protected: 19 as libc::c_int as libc::c_short,
			is_cursed: 126 as libc::c_int as libc::c_short,
			class: 85 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 18 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::w(),
			damage: "2d8",
			quantity: 45 as libc::c_int as libc::c_short,
			ichar: 'W' as i32 as libc::c_short,
			kill_exp: 55 as libc::c_int as libc::c_short,
			is_protected: 14 as libc::c_int as libc::c_short,
			is_cursed: 23 as libc::c_int as libc::c_short,
			class: 75 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::x(),
			damage: "4d6",
			quantity: 42 as libc::c_int as libc::c_short,
			ichar: 'X' as i32 as libc::c_short,
			kill_exp: 110 as libc::c_int as libc::c_short,
			is_protected: 16 as libc::c_int as libc::c_short,
			is_cursed: 25 as libc::c_int as libc::c_short,
			class: 75 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::y(),
			damage: "3d6",
			quantity: 35 as libc::c_int as libc::c_short,
			ichar: 'Y' as i32 as libc::c_short,
			kill_exp: 50 as libc::c_int as libc::c_short,
			is_protected: 11 as libc::c_int as libc::c_short,
			is_cursed: 20 as libc::c_int as libc::c_short,
			class: 80 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 20 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
			picked_up: 0,
			in_use_flags: 0,
			next_object: 0 as *const obj as *mut obj,
		};
		init
	},
	{
		let mut init = obj {
			m_flags: MonsterFlags::z(),
			damage: "1d7",
			quantity: 21 as libc::c_int as libc::c_short,
			ichar: 'Z' as i32 as libc::c_short,
			kill_exp: 8 as libc::c_int as libc::c_short,
			is_protected: 5 as libc::c_int as libc::c_short,
			is_cursed: 14 as libc::c_int as libc::c_short,
			class: 69 as libc::c_int as libc::c_short,
			identified: 0 as libc::c_int as libc::c_short,
			which_kind: 0 as libc::c_int as libc::c_ushort,
			o_row: 0 as libc::c_int as libc::c_short,
			o_col: 0 as libc::c_int as libc::c_short,
			o: 0 as libc::c_int as libc::c_short,
			row: 0,
			col: 0,
			d_enchant: 0,
			quiver: 0,
			trow: 0,
			tcol: 0,
			hit_enchant: 0,
			what_is: 0,
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
		let mut row: libc::c_short = 0;
		let mut col: libc::c_short = 0;
		gr_row_col(
			&mut row,
			&mut col,
			0o100 as libc::c_int as libc::c_ushort
				| 0o200 as libc::c_int as libc::c_ushort
				| 0o4 as libc::c_int as libc::c_ushort
				| 0o1 as libc::c_int as libc::c_ushort,
		);
		put_m_at(row, col, &mut *monster);
	}
}

#[no_mangle]
pub unsafe extern "C" fn gr_monster(
	mut monster: *mut object,
	mut mn: libc::c_int,
) -> *mut object {
	if monster.is_null() {
		monster = alloc_object();
		loop {
			mn = get_rand(0 as libc::c_int, 26 as libc::c_int - 1 as libc::c_int);
			if cur_level as libc::c_int
				>= mon_tab[mn as usize].is_protected as libc::c_int
				&& cur_level as libc::c_int
				<= mon_tab[mn as usize].is_cursed as libc::c_int
			{
				break;
			}
		}
	}
	*monster = mon_tab[mn as usize];
	if (*monster).m_flags.imitates {
		(*monster).what_is = gr_obj_char() as libc::c_ushort;
	}
	if cur_level as libc::c_int > 26 as libc::c_int + 2 as libc::c_int {
		(*monster).m_flags.hasted = true;
	}
	(*monster).trow = -(1 as libc::c_int) as libc::c_short;
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
			mon_disappeared = 0;
			mv_monster(&mut *monster, rogue.row as isize, rogue.col as isize);
			if mon_disappeared != 0 {
				done_with_monster = true;
			}
		} else if (*monster).m_flags.slowed {
			(*monster).flip_slowed_toggle();
			if (*monster).slowed_toggled() {
				done_with_monster = true;
			}
		}
		if !done_with_monster && (*monster).m_flags.confused && move_confused(&mut *monster) {
			done_with_monster = true;
		}
		if !done_with_monster {
			let mut flew = false;
			if (*monster).m_flags.flies && !(*monster).m_flags.napping && !mon_can_go(&*monster, rogue.row as usize, rogue.col as usize) {
				flew = true;
				mv_monster(&mut *monster, rogue.row as isize, rogue.col as isize);
			}
			if !flew || !mon_can_go(&*monster, rogue.row as usize, rogue.col as usize) {
				mv_monster(&mut *monster, rogue.row as isize, rogue.col as isize);
			}
		}
		monster = next_monster;
	}
}

#[no_mangle]
pub unsafe extern "C" fn party_monsters(rn: usize, n: usize) {
	for i in 0..MONSTERS {
		mon_tab[i].set_first_level(mon_tab[i].first_level() - (cur_level % 3))
	}
	let n = n + n;
	for _i in 0..n {
		if no_room_for_monster(rn as usize) {
			break;
		}
		let mut found: Option<(c_int, c_int)> = None;
		for _j in 0..250 {
			let row = get_rand(rooms[rn].top_row + 1, rooms[rn].bottom_row - 1);
			let col = get_rand(rooms[rn].left_col + 1, rooms[rn].right_col - 1);
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
			put_m_at(row as c_short, col as c_short, &mut *monster);
		}
	}
	for i in 0..MONSTERS {
		mon_tab[i].set_first_level(mon_tab[i].first_level() + (cur_level % 3))
	}
}

#[no_mangle]
pub unsafe extern "C" fn gmc_row_col(row: usize, col: usize) -> ncurses::chtype {
	let monster = objects::object_at(&mut level_monsters, row as c_short, col as c_short);
	if !monster.is_null() {
		let invisible = (*monster).m_flags.invisible;
		let bypass_invisible = detect_monster != 0 || see_invisible != 0 || r_see_invisible != 0;
		let is_blind = blind != 0;
		if (invisible && !bypass_invisible) || is_blind {
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
	let defeat_invisibility = detect_monster != 0 || see_invisible != 0 || r_see_invisible != 0;
	if ((*monster).m_flags.invisible && !defeat_invisibility) || (blind != 0) {
		(*monster).trail_char()
	} else if (*monster).m_flags.imitates {
		(*monster).disguise()
	} else {
		(*monster).m_char()
	}
}

pub unsafe fn mv_monster(monster: &mut object, row: isize, col: isize) {
	if monster.m_flags.asleep {
		if monster.m_flags.napping {
			monster.decrement_nap();
			return;
		}
		if (monster.m_flags.wakens)
			&& rogue_is_around(monster.row, monster.col)
			&& rand_percent(if stealthy > 0 { odds::WAKE_PERCENT / (odds::STEALTH_FACTOR + (stealthy as c_int)) } else { odds::WAKE_PERCENT })
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
	if monster.m_flags.stationary && !mon_can_go(monster, rogue.row as usize, rogue.col as usize) {
		return;
	}
	if monster.m_flags.freezing_rogue {
		return;
	}
	if monster.m_flags.confuses && m_confuse(monster) {
		return;
	}
	if mon_can_go(monster, rogue.row as usize, rogue.col as usize) {
		mon_hit(monster, 0 as *mut libc::c_char, 0);
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
		monster.trow = NO_ROOM as c_short;
	} else if monster.trow != NO_ROOM as i16 {
		row = monster.trow as isize;
		col = monster.tcol as isize;
	}
	if monster.row > row as i16 {
		row = (monster.row - 1) as isize;
	} else if monster.row < row as i16 {
		row = (monster.row + 1) as isize;
	}
	if Door.is_set(dungeon[row as usize][monster.col as usize]) && mtry(monster, row as usize, monster.col as usize) {
		return;
	}
	if monster.col > col as i16 {
		col = (monster.col - 1) as isize;
	} else if monster.col < col as i16 {
		col = (monster.col + 1) as isize;
	}
	if Door.is_set(dungeon[monster.row as usize][col as usize]) && mtry(monster, monster.row as usize, col as usize) {
		return;
	}
	if mtry(monster, row as usize, col as usize) {
		return;
	}

	{
		let mut tried: [bool; 6] = [false; 6];
		'moved: for _i in 0..6 {
			loop {
				let n = get_rand(0, 5) as usize;
				if !tried[n] {
					match n {
						0 => if mtry(monster, row as usize, (monster.col - 1) as usize) { break 'moved; }
						1 => if mtry(monster, row as usize, monster.col as usize) { break 'moved; }
						2 => if mtry(monster, row as usize, (monster.col + 1) as usize) { break 'moved; }
						3 => if mtry(monster, (monster.row - 1) as usize, col as usize) { break 'moved; }
						4 => if mtry(monster, monster.row as usize, col as usize) { break 'moved; }
						5 => if mtry(monster, (monster.row + 1) as usize, col as usize) { break 'moved; }
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
			if monster.trow == NO_ROOM as i16 && !mon_sees(monster, rogue.row as c_int, rogue.col as c_int) {
				monster.trow = get_rand(1, DROWS - 2) as c_short;
				monster.tcol = get_rand(0, DCOLS - 1) as c_short;
			} else {
				monster.trow = NO_ROOM as c_short;
				monster.o = 0;
			}
		}
	} else {
		monster.o_row = monster.row;
		monster.o_col = monster.col;
		monster.o = 0;
	}
}

pub unsafe fn mtry(monster: &mut object, row: usize, col: usize) -> bool {
	if mon_can_go(monster, row, col) {
		move_mon_to(monster, row, col);
		return true;
	}
	return false;
}

pub unsafe fn move_mon_to(monster: &mut object, row: usize, col: usize) {
	let mrow = monster.row as usize;
	let mcol = monster.col as usize;

	Monster.clear(&mut dungeon[mrow][mcol]);
	Monster.set(&mut dungeon[row][col]);
	let c = ncurses::mvinch(mrow as i32, mcol as i32);
	if (c >= chtype::from('A')) && (c <= chtype::from('Z'))
	{
		let (mrow, mcol) = (mrow as i32, mcol as i32);
		let no_detect_monster = detect_monster == 0;
		if no_detect_monster {
			ncurses::mvaddch(mrow, mcol, monster.trail_char());
		} else {
			if rogue_can_see(mrow as usize, mcol as usize) {
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
	let not_blind = blind == 0;
	if not_blind && ((detect_monster != 0) || rogue_can_see(row, col)) {
		let bypass_invisibility = (detect_monster != 0) || (see_invisible != 0) || (r_see_invisible != 0);
		if !monster.m_flags.invisible || bypass_invisibility {
			ncurses::mvaddch(row as i32, col as i32, gmc(monster));
		}
	}
	if Door.is_set(dungeon[row][col])
		&& (get_room_number(row as c_int, col as c_int) != cur_room as i32)
		&& Floor.is_only(dungeon[mrow][mcol])
		&& (blind == 0) {
		ncurses::mvaddch(mrow as i32, mcol as i32, chtype::from(' '));
	}
	if Door.is_set(dungeon[row][col]) {
		let entering = Tunnel.is_set(dungeon[mrow][mcol]);
		dr_course(monster, entering, row as c_short, col as c_short);
	} else {
		monster.row = row as c_short;
		monster.col = col as c_short;
	}
}

pub unsafe fn mon_can_go(monster: &obj, row: usize, col: usize) -> bool {
	let dr = monster.row as isize - row as isize;        /* check if move distance > 1 */
	if (dr >= 2) || (dr <= -2) {
		return false;
	}
	let dc = monster.col as isize - col as isize;
	if (dc >= 2) || (dc <= -2) {
		return false;
	}
	if SpotFlag::Nothing.is_set(dungeon[monster.row as usize][col as usize]) || SpotFlag::Nothing.is_set(dungeon[row][monster.col as usize]) {
		return false;
	}
	if !is_passable(row as c_int, col as c_int) || Monster.is_set(dungeon[row][col]) {
		return false;
	}
	if (monster.row != row as i16) && (monster.col != col as i16)
		&& (Door.is_set(dungeon[row][col]) || Door.is_set(dungeon[monster.row as usize][monster.col as usize])) {
		return false;
	}
	if (monster.trow == NO_ROOM as i16)
		&& !monster.m_flags.flits
		&& !monster.m_flags.confused
		&& !monster.m_flags.can_flit
	{
		if (monster.row < rogue.row) && (row < monster.row as usize) { return false; }
		if (monster.row > rogue.row) && (row > monster.row as usize) { return false; }
		if (monster.col < rogue.col) && (col < monster.col as usize) { return false; }
		if (monster.col > rogue.col) && (col > monster.col as usize) { return false; }
	}
	if SpotFlag::Object.is_set(dungeon[row][col]) {
		let obj = objects::object_at(&mut level_objects, row as c_short, col as c_short);
		if (*obj).what_is == object_what::SCROLL && (*obj).which_kind == object_kind::SCARE_MONSTER {
			return false;
		}
	}
	return true;
}

pub fn wake_up(monster: &mut object) {
	monster.m_flags.wake_up();
}

#[no_mangle]
pub unsafe extern "C" fn wake_room(rn: usize, entering: bool, row: usize, col: usize) {
	let wake_percent = {
		let wake_percent = if rn == party_room as usize { odds::PARTY_WAKE_PERCENT } else { odds::WAKE_PERCENT };
		if stealthy > 0 {
			wake_percent / (odds::STEALTH_FACTOR + stealthy as c_int)
		} else {
			wake_percent
		}
	};
	let mut monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).in_room(rn) {
			if entering {
				(*monster).trow = NO_ROOM as c_short;
			} else {
				(*monster).trow = row as c_short;
				(*monster).tcol = col as c_short;
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

#[no_mangle]
pub unsafe extern "C" fn mon_name(mut monster: *mut object) -> *mut libc::c_char {
	if blind != 0
		&& ((*monster).m_flags.invisible && !(detect_monster != 0 || see_invisible != 0 || r_see_invisible != 0)) {
		return b"something\0" as *const u8 as *const libc::c_char as *mut libc::c_char;
	}
	let mut ch: libc::c_short = 0;
	if halluc != 0 {
		ch = (get_rand('A' as i32, 'Z' as i32) - 'A' as i32) as libc::c_short;
		return m_names[ch as usize];
	}
	ch = ((*monster).ichar as libc::c_int - 'A' as i32) as libc::c_short;
	return m_names[ch as usize];
}

pub unsafe fn rogue_is_around(row: c_short, col: c_short) -> bool {
	let rdif = row - rogue.row;
	let cdif = col - rogue.col;
	(rdif >= -1) && (rdif <= 1) && (cdif >= -1) && (cdif <= 1)
}

#[no_mangle]
pub unsafe extern "C" fn wanderer() {
	let mut monster: *mut object = 0 as *mut object;
	let mut row: c_short = 0;
	let mut col: c_short = 0;
	let mut found = false;
	{
		let mut i: c_short = 0;
		while i < 15 && !found {
			monster = gr_monster(0 as *mut object, 0 as libc::c_int);
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
			gr_row_col(&mut row, &mut col, SpotFlag::union(&vec![Floor, Tunnel, Stairs, Object]));
			if rogue_can_see(row as usize, col as usize) == false {
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
	detect_monster = 1;
	let is_blind = blind != 0;
	if is_blind {
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
	let mut row = rogue.row as isize;
	let mut col = rogue.col as isize;
	for i in 0..9 {
		{
			let (r_moved, c_moved) = rand_around(i, row, col);
			row = r_moved;
			col = c_moved;
		}
		let on_rogue = row == rogue.row as isize && col == rogue.col as isize;
		let out_of_bounds = row < MIN_ROW as isize || row > (DROWS - 2) as isize || col < 0 || col > (DCOLS - 1) as isize;
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
		put_m_at(row as c_short, col as c_short, &mut *monster);
		ncurses::mvaddch(row as i32, col as i32, gmc(monster));
		if (*monster).m_flags.wanders || (*monster).m_flags.wakens {
			wake_up(&mut *monster);
		}
	} else {
		message(b"you hear a faint cry of anguish in the distance\0" as *const u8 as *const libc::c_char, 0);
	}
}

pub unsafe fn put_m_at(row: c_short, col: c_short, monster: &mut object) {
	monster.row = row;
	monster.col = col;
	Monster.set(&mut dungeon[row as usize][col as usize]);
	monster.set_trail_char(ncurses::mvinch(row as i32, col as i32));
	pack::add_to_pack(monster, &mut level_monsters, 0);
	aim_monster(monster);
}

pub unsafe fn rogue_can_see(row: usize, col: usize) -> bool {
	let not_blind = blind == 0;
	let in_current_room = get_room_number(row as c_int, col as c_int) == cur_room as i32;
	let not_in_maze = rooms[cur_room as usize].room_type != RoomType::Maze;
	let is_very_close = rogue_is_around(row as c_short, col as c_short);
	not_blind && ((in_current_room && not_in_maze) || is_very_close)
}

pub unsafe fn move_confused(monster: &mut object) -> bool {
	if !monster.m_flags.asleep {
		monster.decrement_moves_confused();
		if monster.m_flags.stationary {
			return if coin_toss() { true } else { false };
		} else if rand_percent(15) {
			return true;
		} else {
			let mut row = monster.row as isize;
			let mut col = monster.col as isize;
			for i in 0..9 {
				{
					let (r_moved, c_moved) = rand_around(i, row, col);
					row = r_moved;
					col = c_moved;
				}
				let on_rogue = row == rogue.row as isize && col == rogue.col as isize;
				if on_rogue {
					return false;
				}
				if mtry(monster, row as usize, col as usize) {
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
	let mut row = monster.row as isize;
	let mut col = monster.col as isize;
	for i in 0..9 {
		{
			let (r_moved, c_moved) = rand_around(i, row, col);
			row = r_moved;
			col = c_moved;
		}
		let on_rogue = row == rogue.row as isize && col == rogue.col as isize;
		if on_rogue {
			continue;
		}
		if mtry(monster, row as usize, col as usize) {
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
	let rn = get_room_number(monster.row as libc::c_int, monster.col as libc::c_int) as usize;
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
	message(
		b"you hear a high pitched humming noise\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
	);
	monster = level_monsters.next_object;
	while !monster.is_null() {
		wake_up(&mut *monster);
		(*monster).m_flags.imitates = false;
		if rogue_can_see((*monster).row as usize, (*monster).col as usize) {
			ncurses::mvaddch((*monster).row as i32, (*monster).col as i32, (*monster).m_char());
		}
		monster = (*monster).next_object;
	}
}

#[no_mangle]
pub unsafe extern "C" fn mon_sees(
	mut monster: *mut object,
	mut row: libc::c_int,
	mut col: libc::c_int,
) -> bool {
	let mut rn: libc::c_short = 0;
	let mut rdif: libc::c_short = 0;
	let mut cdif: libc::c_short = 0;
	rn = get_room_number(row, col) as libc::c_short;
	if rn as libc::c_int != -(1 as libc::c_int)
		&& rn as libc::c_int
		== get_room_number(
		(*monster).row as libc::c_int,
		(*monster).col as libc::c_int,
	)
		&& (*rooms.as_mut_ptr().offset(rn as isize)).room_type as libc::c_int
		& 0o4 as libc::c_int as libc::c_ushort as libc::c_int == 0
	{
		return true;
	}
	rdif = (row - (*monster).row as libc::c_int) as libc::c_short;
	cdif = (col - (*monster).col as libc::c_int) as libc::c_short;
	return rdif as libc::c_int >= -(1 as libc::c_int)
		&& rdif as libc::c_int <= 1 as libc::c_int
		&& cdif as libc::c_int >= -(1 as libc::c_int)
		&& cdif as libc::c_int <= 1 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn mv_aquatars() {
	let mut monster: *mut object = 0 as *mut object;
	monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).m_char() == chtype::from('A') && mon_can_go(&*monster, rogue.row as usize, rogue.col as usize) {
			mv_monster(&mut *monster, rogue.row as isize, rogue.col as isize);
			(*monster).m_flags.already_moved = true;
		}
		monster = (*monster).next_object;
	}
}
