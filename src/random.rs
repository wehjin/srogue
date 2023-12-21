#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use rand::{random, Rng, thread_rng};

static mut rntb: [libc::c_long; 32] = [
	3 as i64 as libc::c_long,
	0x9a319039 as libc::c_uint as libc::c_long,
	0x32d9c024 as i64 as libc::c_long,
	0x9b663182 as libc::c_uint as libc::c_long,
	0x5da1f342 as i64 as libc::c_long,
	0xde3b81e0 as libc::c_uint as libc::c_long,
	0xdf0a6fb5 as libc::c_uint as libc::c_long,
	0xf103bc02 as libc::c_uint as libc::c_long,
	0x48f340fb as i64 as libc::c_long,
	0x7449e56b as i64 as libc::c_long,
	0xbeb1dbb0 as libc::c_uint as libc::c_long,
	0xab5c5918 as libc::c_uint as libc::c_long,
	0x946554fd as libc::c_uint as libc::c_long,
	0x8c2e680f as libc::c_uint as libc::c_long,
	0xeb3d799f as libc::c_uint as libc::c_long,
	0xb11ee0b7 as libc::c_uint as libc::c_long,
	0x2d436b86 as i64 as libc::c_long,
	0xda672e2a as libc::c_uint as libc::c_long,
	0x1588ca88 as i64 as libc::c_long,
	0xe369735d as libc::c_uint as libc::c_long,
	0x904f35f7 as libc::c_uint as libc::c_long,
	0xd7158fd6 as libc::c_uint as libc::c_long,
	0x6fa6f051 as libc::c_long,
	0x616e6b96 as i64 as libc::c_long,
	0xac94efdc as libc::c_uint as libc::c_long,
	0x36413f93 as i64 as libc::c_long,
	0xc622c298 as libc::c_uint as libc::c_long,
	0xf5a42ab8 as libc::c_uint as libc::c_long,
	0x8a88d77b as libc::c_uint as libc::c_long,
	0xf5ad9d0e as libc::c_uint as libc::c_long,
	0x8999220b as libc::c_uint as libc::c_long,
	0x27fb47b9 as i64 as libc::c_long,
];
static mut fptr: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;
static mut rptr: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;
static mut state: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;
static mut rand_type: i64 = 3 as i64;
static mut rand_deg: i64 = 31;
static mut rand_sep: i64 = 3 as i64;
static mut end_ptr: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;

pub fn rrandom() -> libc::c_long { random() }

pub fn get_rand<T>(x: T, y: T) -> T { thread_rng().gen_range(x..=y) }

pub fn rand_percent<T>(percentage: T) -> bool { get_rand(1, 100) <= percentage }

pub fn coin_toss() -> bool { random() }

unsafe extern "C" fn run_static_initializers() {
	fptr = &mut *rntb.as_mut_ptr().offset(4 as i64 as isize)
		as *mut libc::c_long;
	rptr = &mut *rntb.as_mut_ptr().offset(1 as isize)
		as *mut libc::c_long;
	state = &mut *rntb.as_mut_ptr().offset(1 as isize)
		as *mut libc::c_long;
	end_ptr = &mut *rntb.as_mut_ptr().offset(32 as i64 as isize)
		as *mut libc::c_long;
}

#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
