#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn cbreak() -> libc::c_int;
	fn endwin() -> libc::c_int;
	fn initscr() -> *mut WINDOW;
	fn noecho() -> libc::c_int;
	fn nonl() -> libc::c_int;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn wrefresh(_: *mut WINDOW) -> libc::c_int;
	fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
	static mut __stdoutp: *mut FILE;
	fn fflush(_: *mut FILE) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut COLS: libc::c_int;
	static mut LINES: libc::c_int;
	static mut rogue: fighter;
	static mut level_objects: object;
	static mut level_monsters: object;
	fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn strncpy(
		_: *mut libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> *mut libc::c_char;
	fn md_gln() -> *mut libc::c_char;
	fn md_getenv() -> *mut libc::c_char;
	fn md_malloc() -> *mut libc::c_char;
	fn add_to_pack() -> *mut object;
	fn alloc_object() -> *mut object;
	static mut fruit: *mut libc::c_char;
	static mut save_file: *mut libc::c_char;
	static mut party_counter: libc::c_short;
	static mut jump: libc::c_char;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> libc::c_int;
}

use crate::prelude::*;

pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
pub type chtype = libc::c_uint;
pub type fpos_t = __darwin_off_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
	pub _base: *mut libc::c_uchar,
	pub _size: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: libc::c_int,
	pub _w: libc::c_int,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
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
		unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
	>,
	pub _write: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*const libc::c_char,
			libc::c_int,
		) -> libc::c_int,
	>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: libc::c_int,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: libc::c_int,
	pub _offset: fpos_t,
}

pub type FILE = __sFILE;

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
pub static mut login_name: [libc::c_char; 30] = [0; 30];
#[no_mangle]
pub static mut nick_name: *mut libc::c_char = b"\0" as *const u8 as *const libc::c_char
	as *mut libc::c_char;
#[no_mangle]
pub static mut rest_file: *mut libc::c_char = 0 as *const libc::c_char
	as *mut libc::c_char;
#[no_mangle]
pub static mut cant_int: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut did_int: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut score_only: libc::c_char = 0;
#[no_mangle]
pub static mut init_curses: bool = false;
#[no_mangle]
pub static mut save_is_interactive: bool = true;
#[no_mangle]
pub static mut ask_quit: libc::c_char = 1 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut show_skull: libc::c_char = 1 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut error_file: *mut libc::c_char = b"rogue.esave\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut byebye_string: *mut libc::c_char = b"Okay, bye bye!\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn init(
	mut argc: libc::c_int,
	mut argv: *mut *mut libc::c_char,
) -> libc::c_int {
	let mut pn: *mut libc::c_char = 0 as *mut libc::c_char;
	let mut seed: libc::c_int = 0;
	do_args(argc, argv);
	do_opts();
	pn = md_gln();
	if pn.is_null() || strlen(pn) >= 30 as libc::c_int as libc::c_ulong {
		clean_up(b"Hey!  Who are you?\0" as *const u8 as *const libc::c_char);
	}
	strcpy(login_name.as_mut_ptr(), pn);
	if score_only == 0 && rest_file.is_null() {
		printf(
			b"Hello %s, just a moment while I dig the dungeon...\0" as *const u8
				as *const libc::c_char,
			if *nick_name.offset(0 as libc::c_int as isize) as libc::c_int != 0 {
				nick_name
			} else {
				login_name.as_mut_ptr()
			},
		);
		fflush(__stdoutp);
	}
	initscr();
	if LINES < 24 as libc::c_int || COLS < 80 as libc::c_int {
		clean_up(
			b"must be played on 24 x 80 or better screen\0" as *const u8
				as *const libc::c_char,
		);
	}
	start_window();
	init_curses = 1 as libc::c_int as libc::c_char;
	md_heed_signals();
	if score_only != 0 {
		put_scores(0 as *mut object, 0 as libc::c_int);
	}
	seed = md_gseed();
	srrandom(seed);
	if !rest_file.is_null() {
		restore(rest_file);
		return 1 as libc::c_int;
	}
	mix_colors();
	get_wand_and_ring_materials();
	make_scroll_titles();
	level_objects.next_object = 0 as *mut obj;
	level_monsters.next_object = 0 as *mut obj;
	player_init();
	party_counter = get_rand(1 as libc::c_int, 10 as libc::c_int) as libc::c_short;
	ring_stats(0 as libc::c_int);
	return 0 as libc::c_int;
}

pub unsafe fn clean_up(estr: *const libc::c_char) {
	if save_is_interactive {
		if init_curses {
			wmove(stdscr, DROWS - 1, 0);
			wrefresh(stdscr);
			stop_window();
		}
		printf(b"\n%s\n" as *const u8 as *const libc::c_char, estr);
	}
	endwin();
	md_exit(0);
}

pub unsafe fn stop_window() {
	endwin();
	md_control_keybord(1);
}

#[no_mangle]
pub unsafe extern "C" fn byebye() -> libc::c_int {
	md_ignore_signals();
	if ask_quit != 0 {
		quit(1 as libc::c_char);
	} else {
		clean_up(byebye_string);
	}
	md_heed_signals();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn onintr() -> libc::c_int {
	md_ignore_signals();
	if cant_int != 0 {
		did_int = 1 as libc::c_int as libc::c_char;
	} else {
		check_message();
		message(b"interrupt\0" as *const u8 as *const libc::c_char, 1 as libc::c_int);
	}
	md_heed_signals();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn error_save() -> libc::c_int {
	save_is_interactive = 0 as libc::c_int as libc::c_char;
	save_into_file(error_file);
	clean_up(b"\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
	panic!("Reached end of non-void function without returning");
}
