#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn cbreak() -> libc::c_int;
	fn endwin() -> libc::c_int;
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
	fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn strncpy(
		_: *mut libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> *mut libc::c_char;
	fn md_getenv() -> *mut libc::c_char;
	fn md_malloc() -> *mut libc::c_char;
	fn add_to_pack() -> *mut object;
	fn alloc_object() -> *mut object;
	static mut party_counter: libc::c_short;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> libc::c_int;
}

use std::{env, io};
use std::io::Write;
use crate::prelude::*;

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


pub struct Settings {
	pub score_only: bool,
	pub rest_file: Option<String>,
	pub fruit: String,
	pub save_file: Option<String>,
	pub jump: bool,
	pub nick_name: Option<String>,
	pub ask_quit: bool,
	pub show_skull: bool,
	pub login_name: String,
}

impl Settings {
	pub fn load() -> Self {
		let mut settings = Settings {
			score_only: false,
			rest_file: None,
			fruit: "slime-mold ".to_string(),
			save_file: None,
			jump: true,
			nick_name: None,
			ask_quit: true,
			show_skull: true,
			login_name: "PLACEHOLDER".to_string(),
		};
		settings.do_args();
		settings.do_opts();
		settings
	}

	fn do_args(&mut self) {
		let args = env::args();
		for s in &args[1..] {
			if s.starts_with('-') {
				if s[1..].find('s').is_some() {
					self.score_only = true;
				}
			} else {
				self.rest_file = Some(s.clone());
			}
		}
	}

	fn do_opts(&mut self) {
		const DIVIDER: char = ',';
		if let Ok(opts) = std::env::var("ROGUEOPTS") {
			const FRUIT_EQ: &'static str = "fruit=";
			const FILE_EQ: &'static str = "file=";
			const NAME: &'static str = "name=";

			for opt in opts.split(DIVIDER) {
				let opt = opt.trim();
				if opt.starts_with(FRUIT_EQ) {
					self.fruit = format!("{} ", opt[FRUIT_EQ.len()..].to_string());
				} else if opt.starts_with(FILE_EQ) {
					self.save_file = Some(opt[FILE_EQ.len()..].to_string());
				} else if opt == "nojump" {
					self.jump = false;
				} else if opt.starts_with(NAME) {
					self.nick_name = Some(opt[NAME.len()..].to_string())
				} else if opt == "noaskquit" {
					self.ask_quit = false;
				} else if opt == "noskull" || opt == "notomb" {
					self.show_skull = false;
				}
			}
		}
	}
}

#[no_mangle]
pub static mut cant_int: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut did_int: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut save_is_interactive: bool = true;
#[no_mangle]
pub static mut error_file: *mut libc::c_char = b"rogue.esave\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut byebye_string: *mut libc::c_char = b"Okay, bye bye!\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

pub struct GameState {
	pub settings: Settings,
	pub init_curses: bool,
	seed: [u8; 32],
}

impl GameState {
	pub fn new(settings: Settings) -> Self {
		GameState {
			settings,
			init_curses: false,
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
	let mut settings = Settings::load();
	match md_get_login_name() {
		None => {
			clean_up(b"Hey!  Who are you?\0" as *const u8 as *const libc::c_char);
		}
		Some(name) => {
			settings.login_name = name;
		}
	}
	if !settings.score_only && settings.rest_file.is_none() {
		print!("Hello {}, just a moment while I dig the dungeon...", match &settings.nick_name {
			None => &settings.login_name,
			Some(name) => name,
		});
		io::stdout().flush().expect("flush stdout");
	}

	ncurses::initscr();
	if ncurses::LINES() < 24 || ncurses::COLS() < 80 {
		clean_up(b"must be played on 24 x 80 or better screen\0" as *const u8 as *const libc::c_char);
	}

	let mut game = GameState::new(settings);
	start_window();
	game.init_curses = true;

	md_heed_signals();

	if settings.score_only {
		put_scores(0 as *mut object, 0 as libc::c_int);
	}
	game.set_seed(md_get_seed());
	if let Some(rest_file) = &game.settings.rest_file {
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
	ring_stats(0 as libc::c_int);
	return false;
}

pub unsafe fn clean_up(estr: *const libc::c_char) {
	if save_is_interactive {
		if init_curses {
			ncurses::wmove(ncurses::stdscr(), DROWS - 1, 0);
			ncurses::refresh();
			stop_window();
		}
		printf(b"\n%s\n" as *const u8 as *const libc::c_char, estr);
	}
	ncurses::endwin();
	md_exit(0);
}


pub fn start_window() {
	ncurses::cbreak();
	ncurses::noecho();
	ncurses::nonl();
	md_control_keybord(0);
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
