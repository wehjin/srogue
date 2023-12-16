#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

static mut rntb: [libc::c_long; 32] = [
	3 as libc::c_int as libc::c_long,
	0x9a319039 as libc::c_uint as libc::c_long,
	0x32d9c024 as libc::c_int as libc::c_long,
	0x9b663182 as libc::c_uint as libc::c_long,
	0x5da1f342 as libc::c_int as libc::c_long,
	0xde3b81e0 as libc::c_uint as libc::c_long,
	0xdf0a6fb5 as libc::c_uint as libc::c_long,
	0xf103bc02 as libc::c_uint as libc::c_long,
	0x48f340fb as libc::c_int as libc::c_long,
	0x7449e56b as libc::c_int as libc::c_long,
	0xbeb1dbb0 as libc::c_uint as libc::c_long,
	0xab5c5918 as libc::c_uint as libc::c_long,
	0x946554fd as libc::c_uint as libc::c_long,
	0x8c2e680f as libc::c_uint as libc::c_long,
	0xeb3d799f as libc::c_uint as libc::c_long,
	0xb11ee0b7 as libc::c_uint as libc::c_long,
	0x2d436b86 as libc::c_int as libc::c_long,
	0xda672e2a as libc::c_uint as libc::c_long,
	0x1588ca88 as libc::c_int as libc::c_long,
	0xe369735d as libc::c_uint as libc::c_long,
	0x904f35f7 as libc::c_uint as libc::c_long,
	0xd7158fd6 as libc::c_uint as libc::c_long,
	0x6fa6f051 as libc::c_int as libc::c_long,
	0x616e6b96 as libc::c_int as libc::c_long,
	0xac94efdc as libc::c_uint as libc::c_long,
	0x36413f93 as libc::c_int as libc::c_long,
	0xc622c298 as libc::c_uint as libc::c_long,
	0xf5a42ab8 as libc::c_uint as libc::c_long,
	0x8a88d77b as libc::c_uint as libc::c_long,
	0xf5ad9d0e as libc::c_uint as libc::c_long,
	0x8999220b as libc::c_uint as libc::c_long,
	0x27fb47b9 as libc::c_int as libc::c_long,
];
static mut fptr: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;
static mut rptr: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;
static mut state: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;
static mut rand_type: libc::c_int = 3 as libc::c_int;
static mut rand_deg: libc::c_int = 31 as libc::c_int;
static mut rand_sep: libc::c_int = 3 as libc::c_int;
static mut end_ptr: *mut libc::c_long = 0 as *const libc::c_long as *mut libc::c_long;

#[no_mangle]
pub unsafe extern "C" fn srrandom(mut x: libc::c_int) -> libc::c_int {
	let mut i: libc::c_int = 0;
	*state.offset(0 as libc::c_int as isize) = x as libc::c_long;
	if rand_type != 0 as libc::c_int {
		i = 1 as libc::c_int;
		while i < rand_deg {
			*state
				.offset(
					i as isize,
				) = 1103515245 as libc::c_int as libc::c_long
				* *state.offset((i - 1 as libc::c_int) as isize)
				+ 12345 as libc::c_int as libc::c_long;
			i += 1;
			i;
		}
		fptr = &mut *state.offset(rand_sep as isize) as *mut libc::c_long;
		rptr = &mut *state.offset(0 as libc::c_int as isize) as *mut libc::c_long;
		i = 0 as libc::c_int;
		while i < 10 as libc::c_int * rand_deg {
			rrandom();
			i += 1;
			i;
		}
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe extern "C" fn rrandom() -> libc::c_long {
	let mut i_0: libc::c_long = 0;
	if rand_type == 0 as libc::c_int {
		let ref mut fresh0 = *state.offset(0 as libc::c_int as isize);
		*fresh0 = *state.offset(0 as libc::c_int as isize)
			* 1103515245 as libc::c_int as libc::c_long
			+ 12345 as libc::c_int as libc::c_long
			& 0x7fffffff as libc::c_int as libc::c_long;
		i_0 = *fresh0;
	} else {
		*fptr += *rptr;
		i_0 = *fptr >> 1 as libc::c_int & 0x7fffffff as libc::c_int as libc::c_long;
		fptr = fptr.offset(1);
		if fptr >= end_ptr {
			fptr = state;
			rptr = rptr.offset(1);
			rptr;
		} else {
			rptr = rptr.offset(1);
			if rptr >= end_ptr {
				rptr = state;
			}
		}
	}
	return i_0;
}

#[no_mangle]
pub unsafe extern "C" fn get_rand(
	mut x: libc::c_int,
	mut y: libc::c_int,
) -> libc::c_int {
	let mut r: libc::c_int = 0;
	let mut t: libc::c_int = 0;
	let mut lr: libc::c_long = 0;
	if x > y {
		t = y;
		y = x;
		x = t;
	}
	lr = rrandom();
	lr &= 0x7fff as libc::c_int as libc::c_long;
	r = lr as libc::c_int;
	r = r % (y - x + 1 as libc::c_int) + x;
	return r;
}

#[no_mangle]
pub unsafe extern "C" fn rand_percent(mut percentage: libc::c_int) -> bool {
	get_rand(1 as libc::c_int, 100 as libc::c_int) <= percentage
}

#[no_mangle]
pub unsafe extern "C" fn coin_toss() -> libc::c_int {
	return if rrandom() & 0o1 as libc::c_int as libc::c_long != 0 {
		1 as libc::c_int
	} else {
		0 as libc::c_int
	};
}

unsafe extern "C" fn run_static_initializers() {
	fptr = &mut *rntb.as_mut_ptr().offset(4 as libc::c_int as isize)
		as *mut libc::c_long;
	rptr = &mut *rntb.as_mut_ptr().offset(1 as libc::c_int as isize)
		as *mut libc::c_long;
	state = &mut *rntb.as_mut_ptr().offset(1 as libc::c_int as isize)
		as *mut libc::c_long;
	end_ptr = &mut *rntb.as_mut_ptr().offset(32 as libc::c_int as isize)
		as *mut libc::c_long;
}

#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
