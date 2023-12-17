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
use crate::prelude::SpotFlag::{HorWall, Tunnel, VertWall};
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
pub const COL1: libc::c_int = 26;
pub const COL2: libc::c_int = 52;
pub const ROW1: libc::c_int = 7;
pub const ROW2: libc::c_int = 15;
pub const HIDE_PERCENT: c_int = 12;

#[derive(Copy, Clone)]
pub struct DungeonSpot {
	pub col: usize,
	pub row: usize,
}

#[derive(Copy, Clone)]
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
	pub fn is_horwall_or_tunnel(value: c_ushort) -> bool {
		(value & (HorWall as i32 | Tunnel as i32)) != 0
	}
	pub fn is_vertwall_or_tunnel(value: c_ushort) -> bool {
		(value & (VertWall as i32 | Tunnel as i32)) != 0
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
