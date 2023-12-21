#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	static mut curscr: *mut WINDOW;
	fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
	static mut __stdoutp: *mut FILE;
	fn fclose(_: *mut FILE) -> i64;
	fn fflush(_: *mut FILE) -> i64;

	fn putchar(_: i64) -> i64;
	fn putc(_: i64, _: *mut FILE) -> i64;
	fn getchar() -> i64;
	fn fputs(_: *const libc::c_char, _: *mut FILE) -> i64;
	fn onintr() -> i64;
	static mut save_is_interactive: libc::c_char;
	static mut add_strength: libc::c_short;
}

use libc::{c_int, sprintf};
use ncurses::{chtype, waddnstr};
use crate::prelude::*;

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


pub static mut msg_written: String = String::new();
pub static mut msg_cleared: bool = true;
pub static mut hunger_str: String = "".to_string();

pub unsafe extern "C" fn message(msg: &str, intrpt: i64) {
	if save_is_interactive == 0 {
		return;
	}
	if intrpt != 0 {
		interrupted = true;
		md_slurp();
	}
	cant_int = true;

	if !msg_cleared {
		ncurses::mvaddstr((MIN_ROW - 1) as i32, msg_written.len() as i32, MORE);
		ncurses::refresh();
		wait_for_ack();
		check_message();
	}
	ncurses::mvaddstr((MIN_ROW - 1) as i32, 0, msg);
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
	ncurses::mv((MIN_ROW - 1) as i32, 0);
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
	if let Some(insert) = insert {
		ncurses::mvaddstr(0, (n + 1) as i32, insert);
		line.extend(insert.chars());
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
					ncurses::addch(ch as ncurses::chtype);
				}
			}
		}
		const BACKSPACE: char = '\u{8}';
		if ch == BACKSPACE && line.len() > 0 {
			if do_echo {
				ncurses::mvaddch(0, (line.len() + n) as i32, ' ' as ncurses::chtype);
				ncurses::mv((MIN_ROW - 1) as i32, (line.len() + n) as i32);
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
			line.pop();
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

const X_CHAR: c_int = 'X' as c_int;
const CTRL_R_CHAR: c_int = 0o022 as c_int;

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
pub unsafe extern "C" fn print_stats(mut stat_mask: usize) -> i64 {
	let mut buf: [libc::c_char; 16] = [0; 16];
	let mut label: libc::c_char = 0;
	let mut row: i64 = 24 as i64 - 1;
	label = (if stat_mask & 0o200 != 0 { 1 } else { 0 }) as libc::c_char;
	if stat_mask & 0o1 != 0 {
		if label != 0 {
			if ncurses::wmove(ncurses::stdscr(), row as i32, 0) == -1 {
				-1;
			} else {
				waddnstr(
					ncurses::stdscr(),
					"Level: ",
					-1,
				);
			};
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d\0" as *const u8 as *const libc::c_char,
			cur_level as i64,
		);
		if ncurses::wmove(ncurses::stdscr(), row as i32, 7) == -1 {
			-1;
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -1);
		};
		pad(buf.as_mut_ptr(), 2);
	}
	if stat_mask & 0o2 as i64 != 0 {
		if label != 0 {
			if rogue.gold > 900000 as i64 as libc::c_long {
				rogue.gold = 900000 as i64 as libc::c_long;
			}
			if ncurses::wmove(ncurses::stdscr(), row, 10) == -(1) {
				-(1);
			} else {
				waddnstr(
					ncurses::stdscr(),
					"Gold: ",
					-(1),
				);
			};
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%ld\0" as *const u8 as *const libc::c_char,
			rogue.gold,
		);
		if ncurses::wmove(ncurses::stdscr(), row, 16) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -(1));
		};
		pad(buf.as_mut_ptr(), 6);
	}
	if stat_mask & 0o4 as i64 != 0 {
		if label != 0 {
			if ncurses::wmove(ncurses::stdscr(), row, 23 as i64) == -(1) {
				-(1);
			} else {
				waddnstr(
					ncurses::stdscr(),
					b"Hp: \0" as *const u8 as *const libc::c_char,
					-(1),
				);
			};
			if rogue.hp_max as i64 > 800 as i64 {
				rogue
					.hp_current = (rogue.hp_current as i64
					- (rogue.hp_max as i64 - 800 as i64))
					as libc::c_short;
				rogue.hp_max = 800 as i64 as libc::c_short;
			}
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d(%d)\0" as *const u8 as *const libc::c_char,
			rogue.hp_current as i64,
			rogue.hp_max as i64,
		);
		if ncurses::wmove(ncurses::stdscr(), row, 27 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -(1));
		};
		pad(buf.as_mut_ptr(), 8);
	}
	if stat_mask & 0o10 as i64 != 0 {
		if label != 0 {
			if ncurses::wmove(ncurses::stdscr(), row, 36 as i64) == -(1) {
				-(1);
			} else {
				waddnstr(
					ncurses::stdscr(),
					b"Str: \0" as *const u8 as *const libc::c_char,
					-(1),
				);
			};
		}
		if rogue.str_max as i64 > 99 as i64 {
			rogue
				.str_current = (rogue.str_current as i64
				- (rogue.str_max as i64 - 99 as i64)) as libc::c_short;
			rogue.str_max = 99 as i64 as libc::c_short;
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d(%d)\0" as *const u8 as *const libc::c_char,
			rogue.str_current as i64 + add_strength as i64,
			rogue.str_max as i64,
		);
		if ncurses::wmove(ncurses::stdscr(), row, 41) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -(1));
		};
		pad(buf.as_mut_ptr(), 6);
	}
	if stat_mask & 0o20 as i64 != 0 {
		if label != 0 {
			if ncurses::wmove(ncurses::stdscr(), row, 48 as i64) == -(1) {
				-(1);
			} else {
				waddnstr(
					ncurses::stdscr(),
					b"Arm: \0" as *const u8 as *const libc::c_char,
					-(1),
				);
			};
		}
		if !rogue.armor.is_null()
			&& (*rogue.armor).d_enchant as i64 > 99 as i64
		{
			(*rogue.armor).d_enchant = 99 as i64 as libc::c_short;
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d\0" as *const u8 as *const libc::c_char,
			get_armor_class(rogue.armor),
		);
		if ncurses::wmove(ncurses::stdscr(), row, 53 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -(1));
		};
		pad(buf.as_mut_ptr(), 2);
	}
	if stat_mask & 0o40 as i64 != 0 {
		if label != 0 {
			if ncurses::wmove(ncurses::stdscr(), row, 56 as i64) == -(1) {
				-(1);
			} else {
				waddnstr(
					ncurses::stdscr(),
					b"Exp: \0" as *const u8 as *const libc::c_char,
					-(1),
				);
			};
		}
		sprintf(
			buf.as_mut_ptr(),
			b"%d/%ld\0" as *const u8 as *const libc::c_char,
			rogue.exp as i64,
			rogue.exp_points,
		);
		if ncurses::wmove(ncurses::stdscr(), row, 61) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -(1));
		};
		pad(buf.as_mut_ptr(), 11);
	}
	if stat_mask & 0o100 as i64 != 0 {
		if ncurses::wmove(ncurses::stdscr(), row, 73 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), hunger_str.as_mut_ptr(), -(1));
		};
		ncurses::clrtoeol();
	}
	ncurses::refresh();
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn pad(s: *const libc::c_char, n: libc::size_t) {
	for _ in libc::strlen(s)..n {
		ncurses::addch(' ' as ncurses::chtype);
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
	return (ch as i64 >= '0' as i32 && ch as i64 <= '9' as i32)
		as i64 as libc::c_char;
}

#[no_mangle]
pub unsafe extern "C" fn r_index(
	mut str: *mut libc::c_char,
	mut ch: i64,
	mut last: libc::c_char,
) -> i64 {
	let mut i: i64 = 0 as i64;
	if last != 0 {
		i = libc::strlen(str).wrapping_sub(1) as i64;
		while i >= 0 as i64 {
			if *str.offset(i as isize) as i64 == ch {
				return i;
			}
			i -= 1;
		}
	} else {
		i = 0 as i64;
		while *str.offset(i as isize) != 0 {
			if *str.offset(i as isize) as i64 == ch {
				return i;
			}
			i += 1;
		}
	}
	return -(1);
}
