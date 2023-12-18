#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rooms: [room; 0];
	static mut level_monsters: object;
	fn md_malloc() -> *mut libc::c_char;
	fn add_to_pack() -> *mut object;
	static mut cur_level: libc::c_short;
	static mut max_level: libc::c_short;
	static mut party_room: libc::c_short;
	static mut error_file: *mut libc::c_char;
	static mut is_wood: [libc::c_char; 0];
}

use libc::{c_int, c_short};
use crate::prelude::*;

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
pub struct id {
	pub value: libc::c_short,
	pub title: [libc::c_char; 128],
	pub real: [libc::c_char; 128],
	pub id_status: libc::c_ushort,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct obj {
	pub m_flags: MonsterFlags,
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

impl obj {
	pub fn m_char(&self) -> chtype {
		self.ichar as chtype
	}
	pub fn first_level(&self) -> c_short {
		self.is_protected
	}
	pub fn set_first_level(&mut self, value: c_short) {
		self.is_protected = value;
	}
	pub fn set_trail_char(&mut self, ch: ncurses::chtype) {
		self.d_enchant = ch as c_short;
	}
	pub fn trail_char(&self) -> chtype {
		self.d_enchant as chtype
	}
	pub fn disguise(&self) -> chtype {
		self.what_is as chtype
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
	pub fn in_room(&self, rn: usize) -> bool {
		let object_rn = get_room_number(self.row as c_int, self.col as c_int);
		object_rn != NO_ROOM && object_rn as usize == rn
	}
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

#[no_mangle]
pub static mut level_objects: object = obj {
	m_flags: MonsterFlags::default(),
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
pub static mut dungeon: [[libc::c_ushort; 80]; 24] = [[0; 80]; 24];
#[no_mangle]
pub static mut foods: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut party_counter: libc::c_short = 0;
#[no_mangle]
pub static mut free_list: *mut object = 0 as *const object as *mut object;
#[no_mangle]
pub static mut fruit: *mut libc::c_char = b"slime-mold \0" as *const u8
	as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut rogue: fighter = {
	let mut init = fight {
		armor: 0 as *const object as *mut object,
		weapon: 0 as *const object as *mut object,
		left_ring: 0 as *const object as *mut object,
		right_ring: 0 as *const object as *mut object,
		hp_current: 12 as libc::c_int as libc::c_short,
		hp_max: 12 as libc::c_int as libc::c_short,
		str_current: 16 as libc::c_int as libc::c_short,
		str_max: 16 as libc::c_int as libc::c_short,
		pack: {
			let mut init = obj {
				m_flags: MonsterFlags::default(),
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
			init
		},
		gold: 0 as libc::c_int as libc::c_long,
		exp: 1 as libc::c_int as libc::c_short,
		exp_points: 0 as libc::c_int as libc::c_long,
		row: 0 as libc::c_int as libc::c_short,
		col: 0 as libc::c_int as libc::c_short,
		fchar: '@' as i32 as libc::c_short,
		moves_left: 1250 as libc::c_int as libc::c_short,
	};
	init
};
#[no_mangle]
pub static mut id_potions: [id; 14] = unsafe {
	[
		{
			let mut init = id {
				value: 100 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"blue \0                           \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of increase strength \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 250 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"red \0                            \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of restore strength \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 100 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"green \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of healing \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 200 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"grey \0                           \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of extra healing \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 10 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"brown \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of poison \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 300 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"clear \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of raise level \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 10 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"pink \0                           \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of blindness \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"white \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of hallucination \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 100 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"purple \0                         \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of detect monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 100 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"black \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of detect things \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 10 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"yellow \0                         \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of confusion \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 80 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"plaid \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of levitation \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 150 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"burgundy \0                       \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of haste self \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 145 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"beige \0                          \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of see invisible \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
	]
};
#[no_mangle]
pub static mut id_scrolls: [id; 12] = unsafe {
	[
		{
			let mut init = id {
				value: 505 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of protect armor \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 200 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of hold monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 235 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of enchant weapon \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 235 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of enchant armor \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 175 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of identify \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 190 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of teleportation \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of sleep \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 610 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of scare monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 210 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of remove curse \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 100 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of create monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of aggravate monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 180 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                   \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of magic mapping \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
	]
};
#[no_mangle]
pub static mut id_weapons: [id; 8] = unsafe {
	[
		{
			let mut init = id {
				value: 150 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"short bow \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 8 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"darts \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 15 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"arrows \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 27 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"daggers \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 35 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"shurikens \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 360 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"mace \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 470 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"long sword \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 580 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"two-handed sword \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
	]
};
#[no_mangle]
pub static mut id_armors: [id; 7] = unsafe {
	[
		{
			let mut init = id {
				value: 300 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"leather armor \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 300 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"ring mail \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 400 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"scale mail \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 500 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"chain mail \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 600 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"banded mail \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 600 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"splint mail \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 700 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"plate mail \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
	]
};
#[no_mangle]
pub static mut id_wands: [id; 10] = unsafe {
	[
		{
			let mut init = id {
				value: 25 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of teleport away \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 50 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of slow monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 45 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of confuse monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 8 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of invisibility \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 55 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of polymorph \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 2 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of haste monster \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of sleep \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 20 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of magic missile \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 20 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of cancellation \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 0 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of do nothing \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
	]
};
#[no_mangle]
pub static mut id_rings: [id; 11] = unsafe {
	[
		{
			let mut init = id {
				value: 250 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of stealth \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 100 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of teleportation \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 255 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of regeneration \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 295 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of slow digestion \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 200 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of add strength \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 250 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of sustain strength \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 250 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of dexterity \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 25 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of adornment \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 300 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of see invisible \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 290 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of maintain armor \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
		{
			let mut init = id {
				value: 270 as libc::c_int as libc::c_short,
				title: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"                                 \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				real: *::core::mem::transmute::<
					&[u8; 128],
					&mut [libc::c_char; 128],
				>(
					b"of searching \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
				),
				id_status: 0 as libc::c_int as libc::c_ushort,
			};
			init
		},
	]
};

#[no_mangle]
pub unsafe extern "C" fn put_objects() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	if (cur_level as libc::c_int) < max_level as libc::c_int {
		return;
	}
	n = (if coin_toss() != 0 {
		get_rand(2 as libc::c_int, 4 as libc::c_int)
	} else {
		get_rand(3 as libc::c_int, 5 as libc::c_int)
	}) as libc::c_short;
	while rand_percent(33 as libc::c_int) != 0 {
		n += 1;
		n;
	}
	if cur_level as libc::c_int == party_counter as libc::c_int {
		make_party();
		party_counter = next_party() as libc::c_short;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < n as libc::c_int {
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
	mut row: libc::c_int,
	mut col: libc::c_int,
) -> libc::c_int {
	(*obj).row = row as libc::c_short;
	(*obj).col = col as libc::c_short;
	dungeon[row
		as usize][col
		as usize] = (dungeon[row as usize][col as usize] as libc::c_int
		| 0o1 as libc::c_int as libc::c_ushort as libc::c_int) as libc::c_ushort;
	add_to_pack(obj, &mut level_objects, 0 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn object_at(
	mut pack: *mut object,
	mut row: libc::c_short,
	mut col: libc::c_short,
) -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = (*pack).next_object;
	while !obj.is_null()
		&& ((*obj).row as libc::c_int != row as libc::c_int
		|| (*obj).col as libc::c_int != col as libc::c_int)
	{
		obj = (*obj).next_object;
	}
	return obj;
}

#[no_mangle]
pub unsafe extern "C" fn get_letter_object(mut ch: libc::c_int) -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = rogue.pack.next_object;
	while !obj.is_null() && (*obj).ichar as libc::c_int != ch {
		obj = (*obj).next_object;
	}
	return obj;
}

#[no_mangle]
pub unsafe extern "C" fn free_stuff(mut objlist: *mut object) -> libc::c_int {
	let mut obj: *mut object = 0 as *mut object;
	while !((*objlist).next_object).is_null() {
		obj = (*objlist).next_object;
		(*objlist).next_object = (*(*objlist).next_object).next_object;
		free_object(obj);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn name_of(mut obj: *mut object) -> *mut libc::c_char {
	let mut retstring: *mut libc::c_char = 0 as *mut libc::c_char;
	match (*obj).what_is as libc::c_int {
		4 => {
			retstring = (if (*obj).quantity as libc::c_int > 1 as libc::c_int {
				b"scrolls \0" as *const u8 as *const libc::c_char
			} else {
				b"scroll \0" as *const u8 as *const libc::c_char
			}) as *mut libc::c_char;
		}
		8 => {
			retstring = (if (*obj).quantity as libc::c_int > 1 as libc::c_int {
				b"potions \0" as *const u8 as *const libc::c_char
			} else {
				b"potion \0" as *const u8 as *const libc::c_char
			}) as *mut libc::c_char;
		}
		32 => {
			if (*obj).which_kind as libc::c_int == 0 as libc::c_int {
				retstring = b"food \0" as *const u8 as *const libc::c_char
					as *mut libc::c_char;
			} else {
				retstring = fruit;
			}
		}
		64 => {
			retstring = (if *is_wood.as_mut_ptr().offset((*obj).which_kind as isize)
				as libc::c_int != 0
			{
				b"staff \0" as *const u8 as *const libc::c_char
			} else {
				b"wand \0" as *const u8 as *const libc::c_char
			}) as *mut libc::c_char;
		}
		2 => {
			match (*obj).which_kind as libc::c_int {
				1 => {
					retstring = (if (*obj).quantity as libc::c_int > 1 as libc::c_int {
						b"darts \0" as *const u8 as *const libc::c_char
					} else {
						b"dart \0" as *const u8 as *const libc::c_char
					}) as *mut libc::c_char;
				}
				2 => {
					retstring = (if (*obj).quantity as libc::c_int > 1 as libc::c_int {
						b"arrows \0" as *const u8 as *const libc::c_char
					} else {
						b"arrow \0" as *const u8 as *const libc::c_char
					}) as *mut libc::c_char;
				}
				3 => {
					retstring = (if (*obj).quantity as libc::c_int > 1 as libc::c_int {
						b"daggers \0" as *const u8 as *const libc::c_char
					} else {
						b"dagger \0" as *const u8 as *const libc::c_char
					}) as *mut libc::c_char;
				}
				4 => {
					retstring = (if (*obj).quantity as libc::c_int > 1 as libc::c_int {
						b"shurikens \0" as *const u8 as *const libc::c_char
					} else {
						b"shuriken \0" as *const u8 as *const libc::c_char
					}) as *mut libc::c_char;
				}
				_ => {
					retstring = (id_weapons[(*obj).which_kind as usize].title)
						.as_mut_ptr();
				}
			}
		}
		1 => {
			retstring = b"armor \0" as *const u8 as *const libc::c_char
				as *mut libc::c_char;
		}
		128 => {
			retstring = b"ring \0" as *const u8 as *const libc::c_char
				as *mut libc::c_char;
		}
		256 => {
			retstring = b"amulet \0" as *const u8 as *const libc::c_char
				as *mut libc::c_char;
		}
		_ => {
			retstring = b"unknown \0" as *const u8 as *const libc::c_char
				as *mut libc::c_char;
		}
	}
	return retstring;
}

#[no_mangle]
pub unsafe extern "C" fn gr_object() -> *mut object {
	let mut obj: *mut object = 0 as *mut object;
	obj = alloc_object();
	if (foods as libc::c_int) < cur_level as libc::c_int / 2 as libc::c_int {
		(*obj).what_is = 0o40 as libc::c_int as libc::c_ushort;
		foods += 1;
		foods;
	} else {
		(*obj).what_is = gr_what_is();
	}
	match (*obj).what_is as libc::c_int {
		4 => {
			gr_scroll(obj);
		}
		8 => {
			gr_potion(obj);
		}
		2 => {
			gr_weapon(obj, 1 as libc::c_int);
		}
		1 => {
			gr_armor(obj);
		}
		64 => {
			gr_wand(obj);
		}
		32 => {
			get_food(obj, 0 as libc::c_int);
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

#[no_mangle]
pub unsafe extern "C" fn get_armor_class(mut obj: *mut obj) -> libc::c_int {
	if !obj.is_null() {
		return (*obj).class as libc::c_int + (*obj).d_enchant as libc::c_int;
	}
	return 0 as libc::c_int;
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
			message(
				b"cannot allocate object, saving game\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
			save_into_file(error_file);
		}
	}
	(*obj).quantity = 1 as libc::c_int as libc::c_short;
	(*obj).ichar = 'L' as i32 as libc::c_short;
	(*obj).is_cursed = 0 as libc::c_int as libc::c_short;
	(*obj).picked_up = (*obj).is_cursed;
	(*obj).in_use_flags = 0 as libc::c_int as libc::c_ushort;
	(*obj).identified = 0 as libc::c_int as libc::c_ushort as libc::c_short;
	(*obj).damage = b"1d1\0" as *const u8 as *const libc::c_char as *mut libc::c_char;
	return obj;
}

pub unsafe fn free_object(obj: *mut object) {
	(*obj).next_object = free_list;
	free_list = obj;
}

#[no_mangle]
pub unsafe extern "C" fn show_objects() -> libc::c_int {
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
		mc = (if wmove(stdscr, row as libc::c_int, col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int) as chtype
		} else {
			winch(stdscr)
		}) as libc::c_short;
		if ((mc as libc::c_int) < 'A' as i32 || mc as libc::c_int > 'Z' as i32)
			&& (row as libc::c_int != rogue.row as libc::c_int
			|| col as libc::c_int != rogue.col as libc::c_int)
		{
			if wmove(stdscr, row as libc::c_int, col as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				waddch(stdscr, rc as chtype);
			};
		}
		obj = (*obj).next_object;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
			if wmove(
				stdscr,
				(*monster).row as libc::c_int,
				(*monster).col as libc::c_int,
			) == -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				waddch(stdscr, (*monster).what_is as libc::c_int as chtype);
			};
		}
		monster = (*monster).next_object;
	}
	panic!("Reached end of non-void function without returning");
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
			get_food(obj, 0 as libc::c_int);
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
