use libc::{c_int, c_ushort};
pub use crate::message::*;
pub use crate::level::*;
pub use crate::monster::*;
pub use crate::hit::*;
pub use crate::init::*;
pub use crate::instruct::*;
pub use crate::inventory::*;
pub use crate::machdep::*;
pub use crate::r#move::*;
pub use crate::objects::*;
pub use crate::pack::*;
pub use crate::play::*;
pub use crate::random::*;
pub use crate::ring::*;
pub use crate::room::*;
pub use crate::save::*;
pub use crate::score::*;
pub use crate::spec_hit::*;
pub use crate::throw::*;
pub use crate::trap::*;
pub use crate::r#use::*;
pub use crate::zap::*;


pub const MAXROOMS: c_int = 9;
pub const NO_ROOM: c_int = -1;
pub const BIG_ROOM: usize = 10;
pub const R_ROOM: c_ushort = 2;
pub const MIN_ROW: libc::c_int = 1;
pub const DCOLS: libc::c_int = 80;
pub const DROWS: libc::c_int = 24;
pub const MAX_TITLE_LENGTH: usize = 30;
pub const MORE: &'static str = "-more-";
pub const COL1: libc::c_int = 26;
pub const COL2: libc::c_int = 52;
pub const ROW1: libc::c_int = 7;
pub const ROW2: libc::c_int = 15;
pub const HIDE_PERCENT: c_int = 12;


pub const MAX_EXP_LEVEL: usize = 21;
pub const MAX_EXP: usize = 10000000;
pub const MAX_GOLD: usize = 900000;
pub const MAX_ARMOR: usize = 99;
pub const MAX_HP: usize = 800;
pub const MAX_STRENGTH: usize = 99;
pub const LAST_DUNGEON: usize = 99;
pub const INIT_HP: usize = 12;

pub type chtype = ncurses::chtype;

#[derive(Copy, Clone)]
pub struct DungeonSpot {
	pub col: usize,
	pub row: usize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SpotFlag {
	Nothing = 0x0,
	Object = 0o1,
	Monster = 0o2,
	Stairs = 0o4,
	HorWall = 0o10,
	VertWall = 0o20,
	Door = 0o40,
	Floor = 0o100,
	Tunnel = 0o200,
	Trap = 0o400,
	Hidden = 0o1000,
}

impl SpotFlag {
	pub fn union(flags: &Vec<SpotFlag>) -> c_ushort {
		flags.iter().fold(0, |it, more| it & more.code())
	}
	pub fn is_any_set(flags: &Vec<SpotFlag>, value: c_ushort) -> bool {
		for flag in flags {
			if flag.is_set(value) {
				return true;
			}
		}
		return false;
	}

	pub fn is_set(&self, value: c_ushort) -> bool {
		match self {
			SpotFlag::Nothing => value == 0,
			_ => (value & self.code()) != 0,
		}
	}
	pub fn is_only(&self, value: c_ushort) -> bool {
		value == self.code()
	}
	pub fn clear(&self, value: &mut c_ushort) {
		let code = self.code();
		*value &= !code;
	}
	pub fn set(&self, value: &mut c_ushort) {
		let code = self.code();
		*value |= code;
	}
	pub fn code(&self) -> c_ushort {
		match self {
			SpotFlag::Nothing => 0o0,
			SpotFlag::Object => 0o1,
			SpotFlag::Monster => 0o2,
			SpotFlag::Stairs => 0o4,
			SpotFlag::HorWall => 0o10,
			SpotFlag::VertWall => 0o20,
			SpotFlag::Door => 0o40,
			SpotFlag::Floor => 0o100,
			SpotFlag::Tunnel => 0o200,
			SpotFlag::Trap => 0o400,
			SpotFlag::Hidden => 0o1000,
		}
	}
}

pub mod item_usage {
	pub const NOT_USED: u16 = 0o0;
	pub const BEING_WIELDED: u16 = 0o1;
	pub const BEING_WORN: u16 = 0o2;
	pub const ON_LEFT_HAND: u16 = 0o4;
	pub const ON_RIGHT_HAND: u16 = 0o10;
	pub const ON_EITHER_HAND: u16 = 0o14;
	pub const BEING_USED: u16 = 0o17;
}

pub mod object_what {
	pub const ARMOR: u16 = 0o1;
	pub const WEAPON: u16 = 0o2;
	pub const SCROLL: u16 = 0o4;
	pub const POTION: u16 = 0o10;
	pub const GOLD: u16 = 0o20;
	pub const FOOD: u16 = 0o40;
	pub const WAND: u16 = 0o100;
	pub const RING: u16 = 0o200;
	pub const AMULET: u16 = 0o400;
	pub const ALL_OBJECTS: u16 = 0o777;
}

pub mod object_kind {
	pub const PROTECT_ARMOR: u16 = 0;
	pub const HOLD_MONSTER: u16 = 1;
	pub const ENCH_WEAPON: u16 = 2;
	pub const ENCH_ARMOR: u16 = 3;
	pub const IDENTIFY: u16 = 4;
	pub const TELEPORT: u16 = 5;
	pub const SLEEP: u16 = 6;
	pub const SCARE_MONSTER: u16 = 7;
	pub const REMOVE_CURSE: u16 = 8;
	pub const CREATE_MONSTER: u16 = 9;
	pub const AGGRAVATE_MONSTER: u16 = 10;
	pub const MAGIC_MAPPING: u16 = 11;
	pub const SCROLLS: u16 = 12;
}

pub mod food_kind {
	pub const RATION: u16 = 0;
	pub const FRUIT: u16 = 1;
}

pub mod weapon_kind {
	pub const BOW: u16 = 0;
	pub const DART: u16 = 1;
	pub const ARROW: u16 = 2;
	pub const DAGGER: u16 = 3;
	pub const SHURIKEN: u16 = 4;
	pub const MACE: u16 = 5;
	pub const LONG_SWORD: u16 = 6;
	pub const TWO_HANDED_SWORD: u16 = 7;
	pub const WEAPONS: u16 = 8;
}

pub mod stat_const {
	pub const STAT_LEVEL: libc::c_int = 0o1;
	pub const STAT_GOLD: libc::c_int = 0o2;
	pub const STAT_HP: libc::c_int = 0o4;
	pub const STAT_STRENGTH: libc::c_int = 0o10;
	pub const STAT_ARMOR: libc::c_int = 0o20;
	pub const STAT_EXP: libc::c_int = 0o40;
	pub const STAT_HUNGER: libc::c_int = 0o100;
	pub const STAT_LABEL: libc::c_int = 0o200;
	pub const STAT_ALL: libc::c_int = 0o377;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: libc::c_int,
	pub _w: libc::c_int,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: crate::message::__sbuf,
	pub _lbfsize: libc::c_int,
	pub _cookie: *mut libc::c_void,
	pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
	pub _read: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*mut libc::c_char,
			libc::c_int,
		) -> libc::c_int,
	>,
	pub _seek: Option::<
		unsafe extern "C" fn(*mut libc::c_void, crate::message::fpos_t, libc::c_int) -> crate::message::fpos_t,
	>,
	pub _write: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*const libc::c_char,
			libc::c_int,
		) -> libc::c_int,
	>,
	pub _ub: crate::message::__sbuf,
	pub _extra: *mut crate::message::__sFILEX,
	pub _ur: libc::c_int,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: crate::message::__sbuf,
	pub _blksize: libc::c_int,
	pub _offset: crate::message::fpos_t,
}

pub type FILE = __sFILE;
