#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn waddnstr(_: *mut WINDOW, _: *const libc::c_char, _: libc::c_int) -> libc::c_int;

	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut curscr: *mut WINDOW;
	static mut stdscr: *mut WINDOW;
	fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
	static mut __stdoutp: *mut FILE;
	fn fclose(_: *mut FILE) -> libc::c_int;
	fn fflush(_: *mut FILE) -> libc::c_int;

	fn putchar(_: libc::c_int) -> libc::c_int;
	fn putc(_: libc::c_int, _: *mut FILE) -> libc::c_int;
	fn getchar() -> libc::c_int;
	fn fputs(_: *const libc::c_char, _: *mut FILE) -> libc::c_int;
	static mut rogue: fighter;
	fn onintr() -> libc::c_int;
	static mut save_is_interactive: libc::c_char;
	static mut add_strength: libc::c_short;
	static mut cur_level: libc::c_short;
}

use libc::{c_int, sprintf};
use ncurses::ll::mvaddstr;
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

pub static mut msg_written: String = String::new();
pub static mut msg_cleared: bool = true;
pub static mut hunger_str: [libc::c_char; 8] = unsafe {
	*::core::mem::transmute::<&[u8; 8], &mut [libc::c_char; 8]>(b"\0\0\0\0\0\0\0\0")
};

pub unsafe extern "C" fn message(msg: &str, intrpt: libc::c_int) {
	if save_is_interactive == 0 {
		return;
	}
	if intrpt != 0 {
		interrupted = true;
		md_slurp();
	}
	cant_int = true;

	if !msg_cleared {
		ncurses::mvaddstr(MIN_ROW - 1, msg_written.len() as i32, MORE);
		ncurses::refresh();
		wait_for_ack();
		check_message();
	}
	ncurses::mvaddstr(MIN_ROW - 1, 0, msg);
	ncurses::addch(chtype::from(' '));
	ncurses::refresh();
	msg_written = msg.to_string();
	msg_cleared = false;
	cant_int = false;
	if did_int {
		did_int = false;
		onintr();
	}
}

pub unsafe extern "C" fn remessage() {
	if !msg_written.is_empty() {
		message(&msg_written, 0);
	}
}

pub unsafe fn check_message() {
	if msg_cleared {
		return;
	}
	ncurses::mv(MIN_ROW - 1, 0);
	ncurses::clrtoeol();
	ncurses::refresh();
	msg_cleared = true;
}

pub const CANCEL: char = '\u{1b}';

#[no_mangle]
pub unsafe extern "C" fn get_input_line(prompt: &str, insert: Option<&str>, if_cancelled: Option<&str>, add_blank: bool, do_echo: bool) -> String {
	message(prompt, 0);

	let mut line: Vec<char> = Vec::new();
	let n = prompt.len();
	if let Some(insert) = &insert {
		ncurses::mvaddstr(0, (n + 1) as i32, insert);
		line.extend(insert.as_bytes());
		ncurses::mv(0, (n + line.len() + 1) as i32);
		ncurses::refresh();
	}
	let mut ch: char;
	loop {
		ch = rgetchar() as u8 as char;
		if ch == '\r' || ch == '\n' || ch == CANCEL {
			break;
		}
		if ch >= ' ' && ch <= '~' && line.len() < MAX_TITLE_LENGTH {
			if ch != ' ' || line.len() > 0 {
				line.push(ch);
				if do_echo {
					ncurses::addch(ch as chtype);
				}
			}
		}
		const BACKSPACE: char = '\u{8}';
		if ch == BACKSPACE && line.len() > 0 {
			if do_echo {
				ncurses::mvaddch(0, (line.len() + n) as i32, ' ' as chtype);
				ncurses::mv(MIN_ROW - 1, (line.len() + n) as i32);
			}
			line.pop();
		}
		ncurses::refresh();
	}
	check_message();
	if add_blank {
		line.push(' ');
	} else {
		while let Some(' ') = line.last() {
			line.pop()
		}
	}
	if ch == CANCEL || line.is_empty() || (line.len() == 1 && add_blank) {
		if let Some(msg) = if_cancelled {
			message(msg, 0);
		}
		"".to_string()
	} else {
		line.iter().collect()
	}
}

const X_CHAR: libc::c_int = 'X' as libc::c_int;
const CTRL_R_CHAR: libc::c_int = 0o022 as libc::c_int;

pub unsafe fn rgetchar() -> c_int {
	let mut done = false;
	let mut ch = 0;
	while !done {
		ch = libc::getchar();
		match ch {
			CTRL_R_CHAR => { ncurses::wrefresh(ncurses::curscr()); }
			X_CHAR => { save_screen(); }
			_ => { done = true; }
		}
	}
	return ch;
}

#[no_mangle]
pub unsafe extern "C" fn print_stats(mut stat_mask: libc::c_int) -> libc::c_int {
	let mut buf: [libc::c_char; 16] = [0; 16];
	let mut label: libc::c_char = 0;
	let mut row: libc::c_int = 24 as libc::c_int - 1 as libc::c_int;
	label = (if stat_mask & 0o200 as libc::c_int != 0 {
		1 as libc::c_int
	} else {
		0 as libc::c_int
	}) as libc::c_char;
	if stat_mask & 0o1 as libc::c_int != 0 {
		if label != 0 {
			if wmove(stdscr, row, 0 as libc::c_int) == -(1 as libc::c_int) {
				-(1 as libc::c_int);
			} else {
				waddnstr(
					stdscr,
					b"Level: \0" as *const u8 as *const libc::c_char,
					-(1 as libc::c_int),
				);
			};
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d\0" as *const u8 as *const libc::c_char,
			cur_level as libc::c_int,
		);
		if wmove(stdscr, row, 7 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		pad(buf.as_mut_ptr(), 2);
	}
	if stat_mask & 0o2 as libc::c_int != 0 {
		if label != 0 {
			if rogue.gold > 900000 as libc::c_int as libc::c_long {
				rogue.gold = 900000 as libc::c_int as libc::c_long;
			}
			if wmove(stdscr, row, 10 as libc::c_int) == -(1 as libc::c_int) {
				-(1 as libc::c_int);
			} else {
				waddnstr(
					stdscr,
					b"Gold: \0" as *const u8 as *const libc::c_char,
					-(1 as libc::c_int),
				);
			};
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%ld\0" as *const u8 as *const libc::c_char,
			rogue.gold,
		);
		if wmove(stdscr, row, 16 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		pad(buf.as_mut_ptr(), 6);
	}
	if stat_mask & 0o4 as libc::c_int != 0 {
		if label != 0 {
			if wmove(stdscr, row, 23 as libc::c_int) == -(1 as libc::c_int) {
				-(1 as libc::c_int);
			} else {
				waddnstr(
					stdscr,
					b"Hp: \0" as *const u8 as *const libc::c_char,
					-(1 as libc::c_int),
				);
			};
			if rogue.hp_max as libc::c_int > 800 as libc::c_int {
				rogue
					.hp_current = (rogue.hp_current as libc::c_int
					- (rogue.hp_max as libc::c_int - 800 as libc::c_int))
					as libc::c_short;
				rogue.hp_max = 800 as libc::c_int as libc::c_short;
			}
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d(%d)\0" as *const u8 as *const libc::c_char,
			rogue.hp_current as libc::c_int,
			rogue.hp_max as libc::c_int,
		);
		if wmove(stdscr, row, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		pad(buf.as_mut_ptr(), 8);
	}
	if stat_mask & 0o10 as libc::c_int != 0 {
		if label != 0 {
			if wmove(stdscr, row, 36 as libc::c_int) == -(1 as libc::c_int) {
				-(1 as libc::c_int);
			} else {
				waddnstr(
					stdscr,
					b"Str: \0" as *const u8 as *const libc::c_char,
					-(1 as libc::c_int),
				);
			};
		}
		if rogue.str_max as libc::c_int > 99 as libc::c_int {
			rogue
				.str_current = (rogue.str_current as libc::c_int
				- (rogue.str_max as libc::c_int - 99 as libc::c_int)) as libc::c_short;
			rogue.str_max = 99 as libc::c_int as libc::c_short;
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d(%d)\0" as *const u8 as *const libc::c_char,
			rogue.str_current as libc::c_int + add_strength as libc::c_int,
			rogue.str_max as libc::c_int,
		);
		if wmove(stdscr, row, 41 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		pad(buf.as_mut_ptr(), 6);
	}
	if stat_mask & 0o20 as libc::c_int != 0 {
		if label != 0 {
			if wmove(stdscr, row, 48 as libc::c_int) == -(1 as libc::c_int) {
				-(1 as libc::c_int);
			} else {
				waddnstr(
					stdscr,
					b"Arm: \0" as *const u8 as *const libc::c_char,
					-(1 as libc::c_int),
				);
			};
		}
		if !rogue.armor.is_null()
			&& (*rogue.armor).d_enchant as libc::c_int > 99 as libc::c_int
		{
			(*rogue.armor).d_enchant = 99 as libc::c_int as libc::c_short;
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d\0" as *const u8 as *const libc::c_char,
			get_armor_class(rogue.armor),
		);
		if wmove(stdscr, row, 53 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		pad(buf.as_mut_ptr(), 2);
	}
	if stat_mask & 0o40 as libc::c_int != 0 {
		if label != 0 {
			if wmove(stdscr, row, 56 as libc::c_int) == -(1 as libc::c_int) {
				-(1 as libc::c_int);
			} else {
				waddnstr(
					stdscr,
					b"Exp: \0" as *const u8 as *const libc::c_char,
					-(1 as libc::c_int),
				);
			};
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d/%ld\0" as *const u8 as *const libc::c_char,
			rogue.exp as libc::c_int,
			rogue.exp_points,
		);
		if wmove(stdscr, row, 61 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		pad(buf.as_mut_ptr(), 11);
	}
	if stat_mask & 0o100 as libc::c_int != 0 {
		if wmove(stdscr, row, 73 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, hunger_str.as_mut_ptr(), -(1 as libc::c_int));
		};
		ncurses::clrtoeol();
	}
	ncurses::refresh();
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn pad(s: *const libc::c_char, n: libc::size_t) {
	for _ in libc::strlen(s)..n {
		ncurses::addch(' ' as chtype);
	}
}

pub fn save_screen() {
	// TODO
	// FILE *fp;
	// short i, j, row, col;
	// char buf[DCOLS+2];
	// boolean found_non_blank;
	//
	//
	// if ((fp = fopen("rogue.screen", "w")) != NULL) {
	// 	for (i = 0; i < DROWS; i++) {
	// 		found_non_blank = 0;
	// 		for (j = (DCOLS - 1); j >= 0; j--) {
	// 			buf[j] = mvinch(i, j);
	// 			if (!found_non_blank) {
	// 				if ((buf[j] != ' ') || (j == 0)) {
	// 					buf[j + ((j == 0) ? 0 : 1)] = 0;
	// 					found_non_blank = 1;
	// 				}
	// 			}
	// 		}
	// 		fputs(buf, fp);
	// 		putc('\n', fp);
	// 	}
	// 	fclose(fp);
	// } else {
	// 	sound_bell();
	// }
}

pub fn sound_bell() {
	// TODO
	// putchar(7);
	// fflush(stdout);
}

#[no_mangle]
pub unsafe extern "C" fn is_digit(mut ch: libc::c_short) -> libc::c_char {
	return (ch as libc::c_int >= '0' as i32 && ch as libc::c_int <= '9' as i32)
		as libc::c_int as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn r_index(
	mut str: *mut libc::c_char,
	mut ch: libc::c_int,
	mut last: libc::c_char,
) -> libc::c_int {
	let mut i: libc::c_int = 0 as libc::c_int;
	if last != 0 {
		i = libc::strlen(str).wrapping_sub(1) as libc::c_int;
		while i >= 0 as libc::c_int {
			if *str.offset(i as isize) as libc::c_int == ch {
				return i;
			}
			i -= 1;
		}
	} else {
		i = 0 as libc::c_int;
		while *str.offset(i as isize) != 0 {
			if *str.offset(i as isize) as libc::c_int == ch {
				return i;
			}
			i += 1;
		}
	}
	return -(1 as libc::c_int);
}
