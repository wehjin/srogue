#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn cbreak() -> i64;
	fn noecho() -> i64;
	fn nonl() -> i64;
	fn printf(_: *const libc::c_char, _: ...) -> i64;
	static mut __stdoutp: *mut FILE;
	fn fflush(_: *mut FILE) -> i64;
	fn strncpy(
		_: *mut libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> *mut libc::c_char;
	fn md_getenv() -> *mut libc::c_char;
	fn md_malloc() -> *mut libc::c_char;
	fn alloc_object() -> *mut object;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> i64;
}

use std::{io};
use std::io::Write;
use libc::c_short;
use settings::nick_name;
use crate::{console, settings};
use crate::prelude::*;
use crate::prelude::armor_kind::RINGMAIL;
use crate::prelude::object_what::ObjectWhat::{Armor, Weapon};
use crate::prelude::weapon_kind::{ARROW, BOW, MACE};
use crate::settings::{rest_file, score_only};

pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
pub type fpos_t = __darwin_off_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
	pub _base: *mut libc::c_uchar,
	pub _size: i64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: i64,
	pub _w: i64,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
	pub _lbfsize: i64,
	pub _cookie: *mut libc::c_void,
	pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> i64>,
	pub _read: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*mut libc::c_char,
			i64,
		) -> i64,
	>,
	pub _seek: Option::<
		unsafe extern "C" fn(*mut libc::c_void, fpos_t, i64) -> fpos_t,
	>,
	pub _write: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*const libc::c_char,
			i64,
		) -> i64,
	>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: i64,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: i64,
	pub _offset: fpos_t,
}

pub type FILE = __sFILE;


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
pub type attr_t = ncurses::chtype;

pub static mut cant_int: bool = false;
pub static mut did_int: bool = false;
pub static mut save_is_interactive: bool = true;
pub static mut error_file: &'static str = "rogue.esave";
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
	party_counter = get_rand(1, 10);
	ring_stats(false);
	return false;
}

unsafe fn player_init() {
	rogue.pack.next_object = 0 as *mut obj;

	let mut obj = alloc_object();
	get_food(&mut *obj, true);
	add_to_pack(obj, &mut rogue.pack, 1);

	let obj = alloc_object();           /* initial armor */
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = (Armor);
		obj.which_kind = RINGMAIL;
		obj.class = RINGMAIL as isize + 2;
		obj.is_protected = 0;
		obj.d_enchant = 1;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
	do_wear(&mut *obj);

	let obj = alloc_object();           /* initial weapons */
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = (Weapon);
		obj.which_kind = MACE;
		obj.damage = "2d3";
		obj.hit_enchant = 1;
		obj.d_enchant = 1;
		obj.identified = true;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
	do_wield(&mut *obj);

	let obj = alloc_object();
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = (Weapon);
		obj.which_kind = BOW;
		obj.damage = "1d2";
		obj.hit_enchant = 1;
		obj.d_enchant = 0;
		obj.identified = true;
	}
	add_to_pack(obj, &mut rogue.pack, 1);

	let obj = alloc_object();
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = (Weapon);
		obj.which_kind = ARROW;
		obj.quantity = get_rand(25, 35) as c_short;
		obj.damage = "1d2";
		obj.hit_enchant = 0;
		obj.d_enchant = 0;
		obj.identified = true;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
}

pub unsafe fn clean_up(estr: *const libc::c_char) {
	if save_is_interactive {
		if console::is_up() {
			ncurses::wmove(ncurses::stdscr(), (DROWS - 1) as i32, 0);
			ncurses::refresh();
			console::down();
		}
		printf(b"\n%s\n" as *const u8 as *const libc::c_char, estr);
	}
	ncurses::endwin();
	md_exit(0);
}


#[no_mangle]
pub unsafe extern "C" fn byebye(ask_quit: bool) -> i64 {
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
pub unsafe extern "C" fn onintr() -> i64 {
	md_ignore_signals();
	if cant_int {
		did_int = true;
	} else {
		check_message();
		message("interrupt", 1);
	}
	md_heed_signals();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn error_save() -> i64 {
	save_is_interactive = false;
	save_into_file(error_file);
	clean_up(b"\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
	panic!("Reached end of non-void function without returning");
}
