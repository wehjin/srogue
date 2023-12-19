#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::process;

extern "C" {
	fn localtime(_: *const time_t) -> *mut tm;
	fn stat(_: *const libc::c_char, _: *mut stat) -> libc::c_int;
	fn gettimeofday(_: *mut timeval, _: *mut libc::c_void) -> libc::c_int;
	fn signal(
		_: libc::c_int,
		_: Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
	) -> Option::<unsafe extern "C" fn(libc::c_int) -> ()>;
	fn error_save() -> libc::c_int;
	fn onintr() -> libc::c_int;
	fn byebye() -> libc::c_int;
	fn getpid() -> pid_t;
	fn getuid() -> uid_t;
	fn sleep(_: libc::c_uint) -> libc::c_uint;
	fn unlink(_: *const libc::c_char) -> libc::c_int;
	fn getpwuid(_: uid_t) -> *mut passwd;
	fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
	fn exit(_: libc::c_int) -> !;
}

pub type __uint16_t = libc::c_ushort;
pub type __int32_t = libc::c_int;
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
	pub tm_sec: libc::c_int,
	pub tm_min: libc::c_int,
	pub tm_hour: libc::c_int,
	pub tm_mday: libc::c_int,
	pub tm_mon: libc::c_int,
	pub tm_year: libc::c_int,
	pub tm_wday: libc::c_int,
	pub tm_yday: libc::c_int,
	pub tm_isdst: libc::c_int,
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
pub struct stat {
	pub st_dev: dev_t,
	pub st_mode: mode_t,
	pub st_nlink: nlink_t,
	pub st_ino: __darwin_ino64_t,
	pub st_uid: uid_t,
	pub st_gid: gid_t,
	pub st_rdev: dev_t,
	pub st_atimespec: timespec,
	pub st_mtimespec: timespec,
	pub st_ctimespec: timespec,
	pub st_birthtimespec: timespec,
	pub st_size: off_t,
	pub st_blocks: blkcnt_t,
	pub st_blksize: blksize_t,
	pub st_flags: __uint32_t,
	pub st_gen: __uint32_t,
	pub st_lspare: __int32_t,
	pub st_qspare: [__int64_t; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timeval {
	pub tv_sec: __darwin_time_t,
	pub tv_usec: __darwin_suseconds_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct timezone {
	pub tz_minuteswest: libc::c_int,
	pub tz_dsttime: libc::c_int,
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
pub unsafe extern "C" fn md_slurp() -> libc::c_int {
	panic!("Reached end of non-void function without returning");
}

pub fn md_control_keybord(mut mode: libc::c_short) {
	// Stubbing this out allows tty driver so steal some commands like ^Y.
	// See machdep.c for more details
}

#[no_mangle]
pub unsafe extern "C" fn md_heed_signals() -> libc::c_int {
	signal(
		2 as libc::c_int,
		::core::mem::transmute::<
			Option::<unsafe extern "C" fn() -> libc::c_int>,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(
			Some(
				::core::mem::transmute::<
					unsafe extern "C" fn() -> libc::c_int,
					unsafe extern "C" fn() -> libc::c_int,
				>(onintr),
			),
		),
	);
	signal(
		3 as libc::c_int,
		::core::mem::transmute::<
			Option::<unsafe extern "C" fn() -> libc::c_int>,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(
			Some(
				::core::mem::transmute::<
					unsafe extern "C" fn() -> libc::c_int,
					unsafe extern "C" fn() -> libc::c_int,
				>(byebye),
			),
		),
	);
	signal(
		1 as libc::c_int,
		::core::mem::transmute::<
			Option::<unsafe extern "C" fn() -> libc::c_int>,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(
			Some(
				::core::mem::transmute::<
					unsafe extern "C" fn() -> libc::c_int,
					unsafe extern "C" fn() -> libc::c_int,
				>(error_save),
			),
		),
	);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_ignore_signals() -> libc::c_int {
	signal(
		3 as libc::c_int,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(1 as libc::c_int as libc::intptr_t),
	);
	signal(
		2 as libc::c_int,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(1 as libc::c_int as libc::intptr_t),
	);
	signal(
		1 as libc::c_int,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(1 as libc::c_int as libc::intptr_t),
	);
	signal(
		18 as libc::c_int,
		::core::mem::transmute::<
			libc::intptr_t,
			Option::<unsafe extern "C" fn(libc::c_int) -> ()>,
		>(1 as libc::c_int as libc::intptr_t),
	);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_get_file_id(mut fname: *mut libc::c_char) -> libc::c_int {
	let mut sbuf: stat = stat {
		st_dev: 0,
		st_mode: 0,
		st_nlink: 0,
		st_ino: 0,
		st_uid: 0,
		st_gid: 0,
		st_rdev: 0,
		st_atimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_mtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_ctimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_birthtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_size: 0,
		st_blocks: 0,
		st_blksize: 0,
		st_flags: 0,
		st_gen: 0,
		st_lspare: 0,
		st_qspare: [0; 2],
	};
	if stat(fname, &mut sbuf) != 0 {
		return -(1 as libc::c_int);
	}
	return sbuf.st_ino as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn md_link_count(mut fname: *mut libc::c_char) -> libc::c_int {
	let mut sbuf: stat = stat {
		st_dev: 0,
		st_mode: 0,
		st_nlink: 0,
		st_ino: 0,
		st_uid: 0,
		st_gid: 0,
		st_rdev: 0,
		st_atimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_mtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_ctimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_birthtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_size: 0,
		st_blocks: 0,
		st_blksize: 0,
		st_flags: 0,
		st_gen: 0,
		st_lspare: 0,
		st_qspare: [0; 2],
	};
	stat(fname, &mut sbuf);
	return sbuf.st_nlink as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn md_gct(mut rt_buf: *mut rogue_time) -> libc::c_int {
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
	(*rt_buf).month = ((*t).tm_mon + 1 as libc::c_int) as libc::c_short;
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
) -> libc::c_int {
	let mut sbuf: stat = stat {
		st_dev: 0,
		st_mode: 0,
		st_nlink: 0,
		st_ino: 0,
		st_uid: 0,
		st_gid: 0,
		st_rdev: 0,
		st_atimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_mtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_ctimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_birthtimespec: timespec { tv_sec: 0, tv_nsec: 0 },
		st_size: 0,
		st_blocks: 0,
		st_blksize: 0,
		st_flags: 0,
		st_gen: 0,
		st_lspare: 0,
		st_qspare: [0; 2],
	};
	let mut seconds: libc::c_long = 0;
	let mut t: *mut tm = 0 as *mut tm;
	stat(fname, &mut sbuf);
	seconds = sbuf.st_mtimespec.tv_sec;
	t = localtime(&mut seconds);
	(*rt_buf).year = (*t).tm_year as libc::c_short;
	(*rt_buf).month = ((*t).tm_mon + 1 as libc::c_int) as libc::c_short;
	(*rt_buf).day = (*t).tm_mday as libc::c_short;
	(*rt_buf).hour = (*t).tm_hour as libc::c_short;
	(*rt_buf).minute = (*t).tm_min as libc::c_short;
	(*rt_buf).second = (*t).tm_sec as libc::c_short;
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn md_df(mut fname: *mut libc::c_char) -> libc::c_char {
	if unlink(fname) != 0 {
		return 0 as libc::c_int as libc::c_char;
	}
	return 1 as libc::c_int as libc::c_char;
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
pub unsafe extern "C" fn md_sleep(mut nsecs: libc::c_int) -> libc::c_int {
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
pub unsafe extern "C" fn md_malloc(mut n: libc::c_int) -> *mut libc::c_char {
	let t = libc::malloc(n as libc::size_t);
	return t as *mut libc::c_char;
}

pub fn md_get_seed() -> u32 {
	process::id()
}

pub fn md_exit(status: i32) {
	process::exit(status)
}
