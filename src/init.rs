#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn cbreak() -> libc::c_int;
	fn noecho() -> libc::c_int;
	fn nonl() -> libc::c_int;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
	static mut __stdoutp: *mut FILE;
	fn fflush(_: *mut FILE) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut level_objects: object;
	static mut level_monsters: object;
	fn strncpy(
		_: *mut libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> *mut libc::c_char;
	fn md_getenv() -> *mut libc::c_char;
	fn md_malloc() -> *mut libc::c_char;
	fn alloc_object() -> *mut object;
	static mut party_counter: libc::c_short;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> libc::c_int;
}

use std::{io};
use std::io::Write;
use libc::c_short;
use settings::nick_name;
use crate::{console, settings};
use crate::prelude::*;
use crate::prelude::armor_kind::RINGMAIL;
use crate::prelude::object_what::{ARMOR, WEAPON};
use crate::prelude::weapon_kind::{ARROW, BOW, MACE};
use crate::settings::{rest_file, score_only};

pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
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
pub static mut cant_int: bool = false;
#[no_mangle]
pub static mut did_int: bool = false;
#[no_mangle]
pub static mut save_is_interactive: bool = true;
#[no_mangle]
pub static mut error_file: *mut libc::c_char = b"rogue.esave\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut byebye_string: *mut libc::c_char = b"Okay, bye bye!\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

pub struct GameState {
	seed: [u8; 32],
}

impl GameState {
	pub fn new() -> Self {
		GameState {
			seed: [1u8; 32],
		}
	}

	pub fn set_seed(&mut self, seed: u32) {
		let bytes = {
			let mut parts: [u8; 4] = [0; 4];
			for i in 0..4 {
				parts[i] = (seed >> (i * 8)) as u8;
			}
			parts
		};
		for i in 0..self.seed.len() {
			self.seed[i] = bytes[i % bytes.len()]
		}
	}
}

pub unsafe fn init() -> bool {
	match md_get_login_name() {
		None => {
			clean_up(b"Hey!  Who are you?\0" as *const u8 as *const libc::c_char);
		}
		Some(name) => {
			settings::set_login_name(&name);
		}
	}
	if !score_only() && rest_file().is_none() {
		print!("Hello {}, just a moment while I dig the dungeon...", match nick_name() {
			None => settings::login_name(),
			Some(name) => name,
		});
		io::stdout().flush().expect("flush stdout");
	}

	ncurses::initscr();
	if ncurses::LINES() < 24 || ncurses::COLS() < 80 {
		clean_up(b"must be played on 24 x 80 or better screen\0" as *const u8 as *const libc::c_char);
	}
	console::up();

	let mut game = GameState::new();
	md_heed_signals();

	if score_only() {
		put_scores(None, 0);
	}
	game.set_seed(md_get_seed());
	if let Some(rest_file) = rest_file() {
		restore(rest_file);
		return true;
	}
	mix_colors();
	get_wand_and_ring_materials();
	make_scroll_titles();
	level_objects.next_object = 0 as *mut obj;
	level_monsters.next_object = 0 as *mut obj;
	player_init();
	party_counter = get_rand(1 as libc::c_int, 10 as libc::c_int) as libc::c_short;
	ring_stats(false);
	return false;
}

unsafe fn player_init() {
	rogue.pack.next_object = 0 as *mut obj;

	let obj = alloc_object();
	get_food(obj, 1);
	add_to_pack(obj, &mut rogue.pack, 1);

	let obj = alloc_object();           /* initial armor */
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = ARMOR;
		obj.which_kind = RINGMAIL;
		obj.class = (RINGMAIL + 2) as c_short;
		obj.is_protected = 0;
		obj.d_enchant = 1;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
	do_wear(obj);

	let obj = alloc_object();           /* initial weapons */
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = WEAPON;
		obj.which_kind = MACE;
		obj.damage = "2d3";
		obj.hit_enchant = 1;
		obj.d_enchant = 1;
		obj.identified = 1;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
	do_wield(obj);

	let obj = alloc_object();
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = WEAPON;
		obj.which_kind = BOW;
		obj.damage = "1d2";
		obj.hit_enchant = 1;
		obj.d_enchant = 0;
		obj.identified = 1;
	}
	add_to_pack(obj, &mut rogue.pack, 1);

	let obj = alloc_object();
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = WEAPON;
		obj.which_kind = ARROW;
		obj.quantity = get_rand(25, 35) as c_short;
		obj.damage = "1d2";
		obj.hit_enchant = 0;
		obj.d_enchant = 0;
		obj.identified = 1;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
}

pub unsafe fn clean_up(estr: *const libc::c_char) {
	if save_is_interactive {
		if console::is_up() {
			ncurses::wmove(ncurses::stdscr(), DROWS - 1, 0);
			ncurses::refresh();
			console::down();
		}
		printf(b"\n%s\n" as *const u8 as *const libc::c_char, estr);
	}
	ncurses::endwin();
	md_exit(0);
}


#[no_mangle]
pub unsafe extern "C" fn byebye(ask_quit: bool) -> libc::c_int {
	md_ignore_signals();
	if ask_quit {
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
	if cant_int {
		did_int = true;
	} else {
		check_message();
		message("interrupt", 1 as libc::c_int);
	}
	md_heed_signals();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn error_save() -> libc::c_int {
	save_is_interactive = false;
	save_into_file(error_file);
	clean_up(b"\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
	panic!("Reached end of non-void function without returning");
}
