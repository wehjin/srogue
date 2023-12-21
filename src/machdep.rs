#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::{fs, process};
use std::ffi::CString;
use libc::stat;

extern "C" {
	fn localtime(_: *const time_t) -> *mut tm;
	fn gettimeofday(_: *mut timeval, _: *mut libc::c_void) -> i64;
	fn signal(
		_: i64,
		_: Option::<unsafe extern "C" fn(i64) -> ()>,
	) -> Option::<unsafe extern "C" fn(i64) -> ()>;
	fn error_save() -> i64;
	fn onintr() -> i64;
	fn byebye() -> i64;
	fn getpid() -> pid_t;
	fn getuid() -> uid_t;
	fn sleep(_: libc::c_uint) -> libc::c_uint;
	fn unlink(_: *const libc::c_char) -> i64;
	fn getpwuid(_: uid_t) -> *mut passwd;
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
	fn exit(_: i64) -> !;
}

pub type __uint16_t = libc::c_ushort;
pub type __int32_t = i64;
pub type __uint32_t = libc::c_uint;
pub type __int64_t = libc::c_longlong;
pub type __uint64_t = libc::c_ulonglong;
pub type __darwin_time_t = libc::c_long;
pub type __darwin_blkcnt_t = __int64_t;
pub type __darwin_blksize_t = __int32_t;
pub type __darwin_dev_t = __int32_t;
pub type __darwin_gid_t = __uint32_t;
pub type __darwin_ino64_t = __uint64_t;
pub type __darwin_mode_t = __uint16_t;
pub type __darwin_off_t = __int64_t;
pub type __darwin_pid_t = __int32_t;
pub type __darwin_suseconds_t = __int32_t;
pub type __darwin_uid_t = __uint32_t;
pub type time_t = __darwin_time_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
	pub tv_sec: __darwin_time_t,
	pub tv_nsec: libc::c_long,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct tm {
	pub tm_sec: i64,
	pub tm_min: i64,
	pub tm_hour: i64,
	pub tm_mday: i64,
	pub tm_mon: i64,
	pub tm_year: i64,
	pub tm_wday: i64,
	pub tm_yday: i64,
	pub tm_isdst: i64,
	pub tm_gmtoff: libc::c_long,
	pub tm_zone: *mut libc::c_char,
}

pub type off_t = __darwin_off_t;
pub type dev_t = __darwin_dev_t;
pub type blkcnt_t = __darwin_blkcnt_t;
pub type blksize_t = __darwin_blksize_t;
pub type gid_t = __darwin_gid_t;
pub type mode_t = __darwin_mode_t;
pub type nlink_t = __uint16_t;
pub type pid_t = __darwin_pid_t;
pub type uid_t = __darwin_uid_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
	pub tv_sec: __darwin_time_t,
	pub tv_usec: __darwin_suseconds_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timezone {
	pub tz_minuteswest: i64,
	pub tz_dsttime: i64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct passwd {
	pub pw_name: *mut libc::c_char,
	pub pw_passwd: *mut libc::c_char,
	pub pw_uid: uid_t,
	pub pw_gid: gid_t,
	pub pw_change: __darwin_time_t,
	pub pw_class: *mut libc::c_char,
	pub pw_gecos: *mut libc::c_char,
	pub pw_dir: *mut libc::c_char,
	pub pw_shell: *mut libc::c_char,
	pub pw_expire: __darwin_time_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct rogue_time {
	pub year: libc::c_short,
	pub month: libc::c_short,
	pub day: libc::c_short,
	pub hour: libc::c_short,
	pub minute: libc::c_short,
	pub second: libc::c_short,
}

pub type tcflag_t = libc::c_ulong;
pub type cc_t = libc::c_uchar;
pub type speed_t = libc::c_ulong;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct termios {
	pub c_iflag: tcflag_t,
	pub c_oflag: tcflag_t,
	pub c_cflag: tcflag_t,
	pub c_lflag: tcflag_t,
	pub c_cc: [cc_t; 20],
	pub c_ispeed: speed_t,
	pub c_ospeed: speed_t,
}

#[no_mangle]
pub unsafe extern "C" fn md_slurp() -> i64 {
	panic!("Reached end of non-void function without returning");
}

pub fn md_control_keybord(mut mode: libc::c_short) {
	// Stubbing this out allows tty driver so steal some commands like ^Y.
	// See machdep.c for more details
}

#[no_mangle]
pub unsafe extern "C" fn md_heed_signals() -> i64 {
	signal(
		2 as i64,
		::core::mem::transmute::<
			Option::<unsafe extern "C" fn() -> i64>,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(
			Some(
				::core::mem::transmute::<
					unsafe extern "C" fn() -> i64,
					unsafe extern "C" fn() -> i64,
				>(onintr),
			),
		),
	);
	signal(
		3 as i64,
		::core::mem::transmute::<
			Option::<unsafe extern "C" fn() -> i64>,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(
			Some(
				::core::mem::transmute::<
					unsafe extern "C" fn() -> i64,
					unsafe extern "C" fn() -> i64,
				>(byebye),
			),
		),
	);
	signal(
		1,
		::core::mem::transmute::<
			Option::<unsafe extern "C" fn() -> i64>,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(
			Some(
				::core::mem::transmute::<
					unsafe extern "C" fn() -> i64,
					unsafe extern "C" fn() -> i64,
				>(error_save),
			),
		),
	);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_ignore_signals() -> i64 {
	signal(
		3 as i64,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(1 as libc::intptr_t),
	);
	signal(
		2 as i64,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(1 as libc::intptr_t),
	);
	signal(
		1,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(1 as libc::intptr_t),
	);
	signal(
		18 as i64,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(i64) -> ()>,
		>(1 as libc::intptr_t),
	);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_get_file_id(fname: &str) -> i64 {
	let mut sbuf = libc::stat {
		st_dev: 0,
		st_mode: 0,
		st_nlink: 0,
		st_ino: 0,
		st_uid: 0,
		st_gid: 0,
		st_rdev: 0,
		st_atime: 0,
		st_atime_nsec: 0,
		st_mtime: 0,
		st_mtime_nsec: 0,
		st_ctime: 0,
		st_ctime_nsec: 0,
		st_birthtime: 0,
		st_size: 0,
		st_blocks: 0,
		st_blksize: 0,
		st_flags: 0,
		st_gen: 0,
		st_lspare: 0,
		st_qspare: [0; 2],
		st_birthtime_nsec: 0,
	};
	let fname = CString::new(fname).unwrap();
	if stat(fname.as_ptr(), &mut sbuf) == 0 {
		sbuf.st_ino as i64
	} else {
		-(1)
	}
}

#[no_mangle]
pub unsafe extern "C" fn md_link_count(mut fname: *mut libc::c_char) -> i64 {
	let mut sbuf = stat {
		st_dev: 0,
		st_mode: 0,
		st_nlink: 0,
		st_ino: 0,
		st_uid: 0,
		st_gid: 0,
		st_rdev: 0,
		st_atime: 0,
		st_atime_nsec: 0,
		st_mtime: 0,
		st_mtime_nsec: 0,
		st_ctime: 0,
		st_ctime_nsec: 0,
		st_birthtime: 0,
		st_birthtime_nsec: 0,
		st_size: 0,
		st_blocks: 0,
		st_blksize: 0,
		st_flags: 0,
		st_gen: 0,
		st_lspare: 0,
		st_qspare: [0; 2],
	};
	stat(fname, &mut sbuf);
	return sbuf.st_nlink as i64;
}

#[no_mangle]
pub unsafe extern "C" fn md_gct(mut rt_buf: *mut rogue_time) -> i64 {
	let mut tv: timeval = timeval { tv_sec: 0, tv_usec: 0 };
	let mut tzp: timezone = timezone {
		tz_minuteswest: 0,
		tz_dsttime: 0,
	};
	let mut t: *mut tm = 0 as *mut tm;
	let mut seconds: libc::c_long = 0;
	gettimeofday(&mut tv, &mut tzp as *mut timezone as *mut libc::c_void);
	seconds = tv.tv_sec;
	t = localtime(&mut seconds);
	(*rt_buf).year = (*t).tm_year as libc::c_short;
	(*rt_buf).month = ((*t).tm_mon + 1) as libc::c_short;
	(*rt_buf).day = (*t).tm_mday as libc::c_short;
	(*rt_buf).hour = (*t).tm_hour as libc::c_short;
	(*rt_buf).minute = (*t).tm_min as libc::c_short;
	(*rt_buf).second = (*t).tm_sec as libc::c_short;
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_gfmt(
	mut fname: *mut libc::c_char,
	mut rt_buf: *mut rogue_time,
) -> i64 {
	let mut sbuf: stat = stat::default();
	let mut t: *mut tm = 0 as *mut tm;
	stat(fname, &mut sbuf);
	let mut seconds = sbuf.st_mtimespec.tv_sec;
	t = localtime(&mut seconds);
	(*rt_buf).year = (*t).tm_year as libc::c_short;
	(*rt_buf).month = ((*t).tm_mon + 1) as libc::c_short;
	(*rt_buf).day = (*t).tm_mday as libc::c_short;
	(*rt_buf).hour = (*t).tm_hour as libc::c_short;
	(*rt_buf).minute = (*t).tm_min as libc::c_short;
	(*rt_buf).second = (*t).tm_sec as libc::c_short;
	panic!("Reached end of non-void function without returning");
}

pub fn md_df(file_name: &str) -> bool {
	let result = fs::remove_file(file_name);
	result.is_ok()
}


pub fn md_get_login_name() -> Option<String> {
	let username = whoami::username();
	if username.is_empty() {
		None
	} else {
		Some(username)
	}
}


#[no_mangle]
pub unsafe extern "C" fn md_sleep(mut nsecs: i64) -> i64 {
	sleep(nsecs as libc::c_uint);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_getenv(mut name: *mut libc::c_char) -> *mut libc::c_char {
	let mut value: *mut libc::c_char = 0 as *mut libc::c_char;
	value = libc::getenv(name);
	return value;
}

#[no_mangle]
pub unsafe extern "C" fn md_malloc(mut n: i64) -> *mut libc::c_char {
	let t = libc::malloc(n as libc::size_t);
	return t as *mut libc::c_char;
}

pub fn md_get_seed() -> u32 {
	process::id()
}

pub fn md_exit(status: i64) {
	process::exit(status)
}
