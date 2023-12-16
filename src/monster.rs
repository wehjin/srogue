#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use crate::message::message;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::gr_row_col;

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
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

pub type chtype = libc::c_uint;

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
pub struct obj {
	pub m_flags: libc::c_ulong,
	pub damage: *mut libc::c_char,
	pub quantity: libc::c_short,
	pub ichar: libc::c_short,
	pub kill_exp: libc::c_short,
	pub is_protected: libc::c_short,
	pub is_cursed: libc::c_short,
	pub class: libc::c_short,
	pub identified: libc::c_short,
	pub which_kind: libc::c_ushort,
	pub o_row: libc::c_short,
	pub o_col: libc::c_short,
	pub o: libc::c_short,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub d_enchant: libc::c_short,
	pub quiver: libc::c_short,
	pub trow: libc::c_short,
	pub tcol: libc::c_short,
	pub hit_enchant: libc::c_short,
	pub what_is: libc::c_ushort,
	pub picked_up: libc::c_short,
	pub in_use_flags: libc::c_ushort,
	pub next_object: *mut obj,
}

pub type object = obj;

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dr {
	pub oth_room: libc::c_short,
	pub oth_row: libc::c_short,
	pub oth_col: libc::c_short,
	pub door_row: libc::c_short,
	pub door_col: libc::c_short,
}

pub type door = dr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct rm {
	pub bottom_row: libc::c_char,
	pub right_col: libc::c_char,
	pub left_col: libc::c_char,
	pub top_row: libc::c_char,
	pub doors: [door; 4],
	pub is_room: libc::c_ushort,
}

pub type room = rm;

#[no_mangle]
pub static mut level_monsters: object = obj {
	m_flags: 0,
	damage: 0 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long
				| 0o2000 as libc::c_long) as libc::c_ulong,
			damage: b"0d0\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40 as libc::c_long
				| 0o200 as libc::c_long) as libc::c_ulong,
			damage: b"1d3\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40 as libc::c_long) as libc::c_ulong,
			damage: b"3d3/2d5\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long
				| 0o40000000 as libc::c_long) as libc::c_ulong,
			damage: b"4d6/4d9\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long) as libc::c_ulong,
			damage: b"1d3\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o4000 as libc::c_long | 0o100000000 as libc::c_long)
				as libc::c_ulong,
			damage: b"5d5\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long
				| 0o100 as libc::c_long) as libc::c_ulong,
			damage: b"5d5/5d5\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long)
				as libc::c_ulong,
			damage: b"1d3/1d2\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o10000 as libc::c_long) as libc::c_ulong,
			damage: b"0d0\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40 as libc::c_long) as libc::c_ulong,
			damage: b"3d10/4d5\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long
				| 0o100 as libc::c_long) as libc::c_ulong,
			damage: b"1d4\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20000 as libc::c_long) as libc::c_ulong,
			damage: b"0d0\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long
				| 0o10000000 as libc::c_long) as libc::c_ulong,
			damage: b"4d4/3d7\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40000 as libc::c_long) as libc::c_ulong,
			damage: b"0d0\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40 as libc::c_long | 0o20 as libc::c_long
				| 0o1000000 as libc::c_long) as libc::c_ulong,
			damage: b"1d6\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o4 as libc::c_long | 0o40 as libc::c_long
				| 0o200 as libc::c_long) as libc::c_ulong,
			damage: b"5d4\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long)
				as libc::c_ulong,
			damage: b"3d5\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long
				| 0o100000 as libc::c_long) as libc::c_ulong,
			damage: b"2d5\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long)
				as libc::c_ulong,
			damage: b"1d3\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long)
				as libc::c_ulong,
			damage: b"4d6/1d4\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long)
				as libc::c_ulong,
			damage: b"4d10\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long
				| 0o200000 as libc::c_long) as libc::c_ulong,
			damage: b"1d14/1d4\0" as *const u8 as *const libc::c_char
				as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40 as libc::c_long
				| 0o400000 as libc::c_long) as libc::c_ulong,
			damage: b"2d8\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20000000 as libc::c_long)
				as libc::c_ulong,
			damage: b"4d6\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o40 as libc::c_long) as libc::c_ulong,
			damage: b"3d6\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
			m_flags: (0o10 as libc::c_long | 0o20 as libc::c_long | 0o40 as libc::c_long)
				as libc::c_ulong,
			damage: b"1d7\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
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
pub unsafe extern "C" fn put_mons() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut monster: *mut object = 0 as *mut object;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	n = get_rand(4 as libc::c_int, 6 as libc::c_int) as libc::c_short;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < n as libc::c_int {
		monster = gr_monster(0 as *mut object, 0 as libc::c_int);
		if (*monster).m_flags & 0o40 as libc::c_long as libc::c_ulong != 0
			&& coin_toss() != 0
		{
			wake_up(monster);
		}
		gr_row_col(
			&mut row,
			&mut col,
			0o100 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o200 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o1 as libc::c_int as libc::c_ushort as libc::c_int,
		);
		put_m_at(row as libc::c_int, col as libc::c_int, monster);
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
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
	if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
		(*monster).what_is = gr_obj_char() as libc::c_ushort;
	}
	if cur_level as libc::c_int > 26 as libc::c_int + 2 as libc::c_int {
		(*monster).m_flags |= 0o1 as libc::c_long as libc::c_ulong;
	}
	(*monster).trow = -(1 as libc::c_int) as libc::c_short;
	return monster;
}

#[no_mangle]
pub unsafe extern "C" fn mv_mons() -> libc::c_int {
	let mut current_block: u64;
	let mut monster: *mut object = 0 as *mut object;
	let mut next_object: *mut object = 0 as *mut object;
	let mut flew: libc::c_char = 0;
	if haste_self as libc::c_int % 2 as libc::c_int != 0 {
		return;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		next_object = (*monster).next_object;
		if (*monster).m_flags & 0o1 as libc::c_long as libc::c_ulong != 0 {
			mon_disappeared = 0 as libc::c_int as libc::c_char;
			mv_monster(monster, rogue.row as libc::c_int, rogue.col as libc::c_int);
			if mon_disappeared != 0 {
				current_block = 17110343724390633633;
			} else {
				current_block = 5399440093318478209;
			}
		} else if (*monster).m_flags & 0o2 as libc::c_long as libc::c_ulong != 0 {
			(*monster).quiver = ((*monster).quiver == 0) as libc::c_int as libc::c_short;
			if (*monster).quiver != 0 {
				current_block = 17110343724390633633;
			} else {
				current_block = 5399440093318478209;
			}
		} else {
			current_block = 5399440093318478209;
		}
		match current_block {
			5399440093318478209 => {
				if !((*monster).m_flags & 0o1000 as libc::c_long as libc::c_ulong != 0
					&& move_confused(monster) != 0)
				{
					flew = 0 as libc::c_int as libc::c_char;
					if (*monster).m_flags & 0o100 as libc::c_long as libc::c_ulong != 0
						&& (*monster).m_flags
						& 0o200000000 as libc::c_long as libc::c_ulong == 0
						&& mon_can_go(
						monster,
						rogue.row as libc::c_int,
						rogue.col as libc::c_int,
					) == 0
					{
						flew = 1 as libc::c_int as libc::c_char;
						mv_monster(
							monster,
							rogue.row as libc::c_int,
							rogue.col as libc::c_int,
						);
					}
					if !(flew as libc::c_int != 0
						&& mon_can_go(
						monster,
						rogue.row as libc::c_int,
						rogue.col as libc::c_int,
					) != 0)
					{
						mv_monster(
							monster,
							rogue.row as libc::c_int,
							rogue.col as libc::c_int,
						);
					}
				}
			}
			_ => {}
		}
		monster = next_object;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn party_monsters(
	mut rn: libc::c_int,
	mut n: libc::c_int,
) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut monster: *mut object = 0 as *mut object;
	let mut found: libc::c_char = 0;
	n += n;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 26 as libc::c_int {
		mon_tab[i as usize]
			.is_protected = (mon_tab[i as usize].is_protected as libc::c_int
			- cur_level as libc::c_int % 3 as libc::c_int) as libc::c_short;
		i += 1;
		i;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < n {
		if no_room_for_monster(rn) != 0 {
			break;
		}
		found = 0 as libc::c_int as libc::c_char;
		j = found as libc::c_short;
		while found == 0 && (j as libc::c_int) < 250 as libc::c_int {
			row = get_rand(
				(*rooms.as_mut_ptr().offset(rn as isize)).top_row as libc::c_int
					+ 1 as libc::c_int,
				(*rooms.as_mut_ptr().offset(rn as isize)).bottom_row as libc::c_int
					- 1 as libc::c_int,
			) as libc::c_short;
			col = get_rand(
				(*rooms.as_mut_ptr().offset(rn as isize)).left_col as libc::c_int
					+ 1 as libc::c_int,
				(*rooms.as_mut_ptr().offset(rn as isize)).right_col as libc::c_int
					- 1 as libc::c_int,
			) as libc::c_short;
			if dungeon[row as usize][col as usize] as libc::c_int
				& 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
				&& dungeon[row as usize][col as usize] as libc::c_int
				& (0o100 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o200 as libc::c_int as libc::c_ushort as libc::c_int) != 0
			{
				found = 1 as libc::c_int as libc::c_char;
			}
			j += 1;
			j;
		}
		if found != 0 {
			monster = gr_monster(0 as *mut object, 0 as libc::c_int);
			if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong == 0 {
				(*monster).m_flags |= 0o20 as libc::c_long as libc::c_ulong;
			}
			put_m_at(row as libc::c_int, col as libc::c_int, monster);
		}
		i += 1;
		i;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 26 as libc::c_int {
		mon_tab[i as usize]
			.is_protected = (mon_tab[i as usize].is_protected as libc::c_int
			+ cur_level as libc::c_int % 3 as libc::c_int) as libc::c_short;
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn gmc_row_col(
	mut row: libc::c_int,
	mut col: libc::c_int,
) -> libc::c_int {
	let mut monster: *mut object = 0 as *mut object;
	let mut retval: libc::c_short = 0;
	monster = object_at(&mut level_monsters, row, col);
	if !monster.is_null() {
		if !(detect_monster as libc::c_int != 0 || see_invisible as libc::c_int != 0
			|| r_see_invisible as libc::c_int != 0)
			&& (*monster).m_flags & 0o4 as libc::c_long as libc::c_ulong != 0
			|| blind as libc::c_int != 0
		{
			retval = (*monster).d_enchant;
			return retval as libc::c_int;
		}
		if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
			return (*monster).what_is as libc::c_int;
		}
		return (*monster).ichar as libc::c_int;
	} else {
		return '&' as i32;
	};
}

#[no_mangle]
pub unsafe extern "C" fn gmc(mut monster: *mut object) -> libc::c_int {
	if !(detect_monster as libc::c_int != 0 || see_invisible as libc::c_int != 0
		|| r_see_invisible as libc::c_int != 0)
		&& (*monster).m_flags & 0o4 as libc::c_long as libc::c_ulong != 0
		|| blind as libc::c_int != 0
	{
		return (*monster).d_enchant as libc::c_int;
	}
	if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
		return (*monster).what_is as libc::c_int;
	}
	return (*monster).ichar as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn wake_room(
	mut rn: libc::c_short,
	mut entering: libc::c_char,
	mut row: libc::c_short,
	mut col: libc::c_short,
) -> libc::c_int {
	let mut monster: *mut object = 0 as *mut object;
	let mut wake_percent: libc::c_short = 0;
	let mut in_room: libc::c_char = 0;
	wake_percent = (if rn as libc::c_int == party_room as libc::c_int {
		75 as libc::c_int
	} else {
		45 as libc::c_int
	}) as libc::c_short;
	if stealthy as libc::c_int > 0 as libc::c_int {
		wake_percent = (wake_percent as libc::c_int
			/ (3 as libc::c_int + stealthy as libc::c_int)) as libc::c_short;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		in_room = (rn as libc::c_int
			== get_room_number(
			(*monster).row as libc::c_int,
			(*monster).col as libc::c_int,
		)) as libc::c_int as libc::c_char;
		if in_room != 0 {
			if entering != 0 {
				(*monster).trow = -(1 as libc::c_int) as libc::c_short;
			} else {
				(*monster).trow = row;
				(*monster).tcol = col;
			}
		}
		if (*monster).m_flags & 0o20 as libc::c_long as libc::c_ulong != 0
			&& rn as libc::c_int
			== get_room_number(
			(*monster).row as libc::c_int,
			(*monster).col as libc::c_int,
		)
		{
			if rand_percent(wake_percent as libc::c_int) != 0 {
				wake_up(monster);
			}
		}
		monster = (*monster).next_object;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn mon_name(mut monster: *mut object) -> *mut libc::c_char {
	let mut ch: libc::c_short = 0;
	if blind as libc::c_int != 0
		|| (*monster).m_flags & 0o4 as libc::c_long as libc::c_ulong != 0
		&& !(detect_monster as libc::c_int != 0 || see_invisible as libc::c_int != 0
		|| r_see_invisible as libc::c_int != 0)
	{
		return b"something\0" as *const u8 as *const libc::c_char as *mut libc::c_char;
	}
	if halluc != 0 {
		ch = (get_rand('A' as i32, 'Z' as i32) - 'A' as i32) as libc::c_short;
		return m_names[ch as usize];
	}
	ch = ((*monster).ichar as libc::c_int - 'A' as i32) as libc::c_short;
	return m_names[ch as usize];
}

#[no_mangle]
pub unsafe extern "C" fn wanderer() -> libc::c_int {
	let mut monster: *mut object = 0 as *mut object;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut found: libc::c_char = 0 as libc::c_int as libc::c_char;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 15 as libc::c_int && found == 0 {
		monster = gr_monster(0 as *mut object, 0 as libc::c_int);
		if (*monster).m_flags
			& (0o20 as libc::c_long | 0o40 as libc::c_long) as libc::c_ulong == 0
		{
			free_object(monster);
		} else {
			found = 1 as libc::c_int as libc::c_char;
		}
		i += 1;
		i;
	}
	if found != 0 {
		found = 0 as libc::c_int as libc::c_char;
		wake_up(monster);
		i = 0 as libc::c_int as libc::c_short;
		while (i as libc::c_int) < 25 as libc::c_int && found == 0 {
			gr_row_col(
				&mut row,
				&mut col,
				0o100 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o200 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o1 as libc::c_int as libc::c_ushort as libc::c_int,
			);
			if rogue_can_see(row as libc::c_int, col as libc::c_int) == 0 {
				put_m_at(row as libc::c_int, col as libc::c_int, monster);
				found = 1 as libc::c_int as libc::c_char;
			}
			i += 1;
			i;
		}
		if found == 0 {
			free_object(monster);
		}
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn show_monsters() -> libc::c_int {
	let mut monster: *mut object = 0 as *mut object;
	detect_monster = 1 as libc::c_int as libc::c_char;
	if blind != 0 {
		return;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		if wmove(stdscr, (*monster).row as libc::c_int, (*monster).col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int);
		} else {
			waddch(stdscr, (*monster).ichar as chtype);
		};
		if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
			(*monster).m_flags &= !(0o20000000 as libc::c_long) as libc::c_ulong;
			(*monster).m_flags |= 0o20 as libc::c_long as libc::c_ulong;
		}
		monster = (*monster).next_object;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn create_monster() -> libc::c_int {
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut found: libc::c_char = 0 as libc::c_int as libc::c_char;
	let mut monster: *mut object = 0 as *mut object;
	row = rogue.row;
	col = rogue.col;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 9 as libc::c_int {
		rand_around(i as libc::c_int, &mut row, &mut col);
		if !(row as libc::c_int == rogue.row as libc::c_int
			&& {
			col = rogue.col;
			col as libc::c_int != 0
		} || (row as libc::c_int) < 1 as libc::c_int
			|| row as libc::c_int > 24 as libc::c_int - 2 as libc::c_int
			|| (col as libc::c_int) < 0 as libc::c_int
			|| col as libc::c_int > 80 as libc::c_int - 1 as libc::c_int)
		{
			if dungeon[row as usize][col as usize] as libc::c_int
				& 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
				&& dungeon[row as usize][col as usize] as libc::c_int
				& (0o100 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o200 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o40 as libc::c_int as libc::c_ushort as libc::c_int) != 0
			{
				found = 1 as libc::c_int as libc::c_char;
				break;
			}
		}
		i += 1;
		i;
	}
	if found != 0 {
		monster = gr_monster(0 as *mut object, 0 as libc::c_int);
		put_m_at(row as libc::c_int, col as libc::c_int, monster);
		if wmove(stdscr, row as libc::c_int, col as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddch(stdscr, gmc(monster) as chtype);
		};
		if (*monster).m_flags
			& (0o40 as libc::c_long | 0o20 as libc::c_long) as libc::c_ulong != 0
		{
			wake_up(monster);
		}
	} else {
		message(
			b"you hear a faint cry of anguish in the distance\0" as *const u8
				as *const libc::c_char,
			0 as libc::c_int,
		);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn aggravate() -> libc::c_int {
	let mut monster: *mut object = 0 as *mut object;
	message(
		b"you hear a high pitched humming noise\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
	);
	monster = level_monsters.next_object;
	while !monster.is_null() {
		wake_up(monster);
		(*monster).m_flags &= !(0o20000000 as libc::c_long) as libc::c_ulong;
		if rogue_can_see((*monster).row as libc::c_int, (*monster).col as libc::c_int)
			!= 0
		{
			if wmove(
				stdscr,
				(*monster).row as libc::c_int,
				(*monster).col as libc::c_int,
			) == -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				waddch(stdscr, (*monster).ichar as chtype);
			};
		}
		monster = (*monster).next_object;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn mon_sees(
	mut monster: *mut object,
	mut row: libc::c_int,
	mut col: libc::c_int,
) -> libc::c_char {
	let mut rn: libc::c_short = 0;
	let mut rdif: libc::c_short = 0;
	let mut cdif: libc::c_short = 0;
	let mut retval: libc::c_short = 0;
	rn = get_room_number(row, col) as libc::c_short;
	if rn as libc::c_int != -(1 as libc::c_int)
		&& rn as libc::c_int
		== get_room_number(
		(*monster).row as libc::c_int,
		(*monster).col as libc::c_int,
	)
		&& (*rooms.as_mut_ptr().offset(rn as isize)).is_room as libc::c_int
		& 0o4 as libc::c_int as libc::c_ushort as libc::c_int == 0
	{
		return 1 as libc::c_int as libc::c_char;
	}
	rdif = (row - (*monster).row as libc::c_int) as libc::c_short;
	cdif = (col - (*monster).col as libc::c_int) as libc::c_short;
	retval = (rdif as libc::c_int >= -(1 as libc::c_int)
		&& rdif as libc::c_int <= 1 as libc::c_int
		&& cdif as libc::c_int >= -(1 as libc::c_int)
		&& cdif as libc::c_int <= 1 as libc::c_int) as libc::c_int as libc::c_short;
	return retval as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn mv_aquatars() -> libc::c_int {
	let mut monster: *mut object = 0 as *mut object;
	monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).ichar as libc::c_int == 'A' as i32
			&& mon_can_go(monster, rogue.row as libc::c_int, rogue.col as libc::c_int)
			!= 0
		{
			mv_monster(monster, rogue.row as libc::c_int, rogue.col as libc::c_int);
			(*monster).m_flags |= 0o400000000 as libc::c_long as libc::c_ulong;
		}
		monster = (*monster).next_object;
	}
	panic!("Reached end of non-void function without returning");
}
