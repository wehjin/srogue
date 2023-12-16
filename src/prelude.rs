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


pub const DROWS: libc::c_int = 80;
pub const DCOLS: libc::c_int = 24;

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
