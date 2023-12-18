#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn waddnstr(_: *mut WINDOW, _: *const libc::c_char, _: libc::c_int) -> libc::c_int;
	fn wclrtoeol(_: *mut WINDOW) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn wrefresh(_: *mut WINDOW) -> libc::c_int;
	static mut curscr: *mut WINDOW;
	static mut stdscr: *mut WINDOW;
	fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
	static mut __stdoutp: *mut FILE;
	fn fclose(_: *mut FILE) -> libc::c_int;
	fn fflush(_: *mut FILE) -> libc::c_int;
	fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
	fn putchar(_: libc::c_int) -> libc::c_int;
	fn putc(_: libc::c_int, _: *mut FILE) -> libc::c_int;
	fn getchar() -> libc::c_int;
	fn fputs(_: *const libc::c_char, _: *mut FILE) -> libc::c_int;
	static mut rogue: fighter;
	fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn onintr() -> libc::c_int;
	static mut cant_int: libc::c_char;
	static mut did_int: libc::c_char;
	static mut interrupted: libc::c_char;
	static mut save_is_interactive: libc::c_char;
	static mut add_strength: libc::c_short;
	static mut cur_level: libc::c_short;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
}

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

#[no_mangle]
pub static mut msg_line: [libc::c_char; 80] = unsafe {
	*::core::mem::transmute::<
		&[u8; 80],
		&mut [libc::c_char; 80],
	>(
		b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
	)
};
#[no_mangle]
pub static mut msg_col: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut msg_cleared: libc::c_char = 1 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut hunger_str: [libc::c_char; 8] = unsafe {
	*::core::mem::transmute::<&[u8; 8], &mut [libc::c_char; 8]>(b"\0\0\0\0\0\0\0\0")
};

#[no_mangle]
pub unsafe extern "C" fn message(
	mut msg: *const libc::c_char,
	intrpt: libc::c_int,
) {
	if save_is_interactive == 0 {
		return;
	}
	if intrpt != 0 {
		interrupted = 1 as libc::c_int as libc::c_char;
		md_slurp();
	}
	cant_int = 1 as libc::c_int as libc::c_char;
	if msg_cleared == 0 {
		if wmove(stdscr, 1 as libc::c_int - 1 as libc::c_int, msg_col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"-more-\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		wrefresh(stdscr);
		wait_for_ack();
		check_message();
	}
	strcpy(msg_line.as_mut_ptr(), msg);
	if wmove(stdscr, 1 as libc::c_int - 1 as libc::c_int, 0 as libc::c_int)
		== -(1 as libc::c_int)
	{
		-(1 as libc::c_int);
	} else {
		waddnstr(stdscr, msg, -(1 as libc::c_int));
	};
	waddch(stdscr, ' ' as i32 as chtype);
	wrefresh(stdscr);
	msg_cleared = 0 as libc::c_int as libc::c_char;
	msg_col = strlen(msg) as libc::c_short;
	cant_int = 0 as libc::c_int as libc::c_char;
	if did_int != 0 {
		did_int = 0 as libc::c_int as libc::c_char;
		onintr();
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn remessage() -> libc::c_int {
	if msg_line[0 as libc::c_int as usize] != 0 {
		message(msg_line.as_mut_ptr(), 0 as libc::c_int);
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn check_message() {
	if msg_cleared == 1 {
		return;
	}
	wmove(
		stdscr,
		1 as libc::c_int - 1 as libc::c_int,
		0 as libc::c_int,
	);
	wclrtoeol(stdscr);
	wrefresh(stdscr);
	msg_cleared = 1;
}

#[no_mangle]
pub unsafe extern "C" fn get_input_line(
	mut prompt: *mut libc::c_char,
	mut insert: *mut libc::c_char,
	mut buf: *mut libc::c_char,
	mut if_cancelled: *mut libc::c_char,
	mut add_blank: libc::c_char,
	mut do_echo: libc::c_char,
) -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut i: libc::c_short = 0 as libc::c_int as libc::c_short;
	let mut n: libc::c_short = 0;
	message(prompt, 0 as libc::c_int);
	n = strlen(prompt) as libc::c_short;
	if *insert.offset(0 as libc::c_int as isize) != 0 {
		if wmove(stdscr, 0 as libc::c_int, n as libc::c_int + 1 as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, insert, -(1 as libc::c_int));
		};
		strcpy(buf, insert);
		i = strlen(insert) as libc::c_short;
		wmove(
			stdscr,
			0 as libc::c_int,
			n as libc::c_int + i as libc::c_int + 1 as libc::c_int,
		);
		wrefresh(stdscr);
	}
	loop {
		ch = rgetchar() as libc::c_short;
		if !(ch as libc::c_int != '\r' as i32 && ch as libc::c_int != '\n' as i32
			&& ch as libc::c_int != '\u{1b}' as i32)
		{
			break;
		}
		if ch as libc::c_int >= ' ' as i32 && ch as libc::c_int <= '~' as i32
			&& (i as libc::c_int) < 30 as libc::c_int - 2 as libc::c_int
		{
			if ch as libc::c_int != ' ' as i32 || i as libc::c_int > 0 as libc::c_int {
				let fresh0 = i;
				i = i + 1;
				*buf.offset(fresh0 as isize) = ch as libc::c_char;
				if do_echo != 0 {
					waddch(stdscr, ch as chtype);
				}
			}
		}
		if ch as libc::c_int == '\u{8}' as i32 && i as libc::c_int > 0 as libc::c_int {
			if do_echo != 0 {
				if wmove(stdscr, 0 as libc::c_int, i as libc::c_int + n as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					waddch(stdscr, ' ' as i32 as chtype);
				};
				wmove(
					stdscr,
					1 as libc::c_int - 1 as libc::c_int,
					i as libc::c_int + n as libc::c_int,
				);
			}
			i -= 1;
			i;
		}
		wrefresh(stdscr);
	}
	check_message();
	if add_blank != 0 {
		let fresh1 = i;
		i = i + 1;
		*buf.offset(fresh1 as isize) = ' ' as i32 as libc::c_char;
	} else {
		while i as libc::c_int > 0 as libc::c_int
			&& *buf.offset((i as libc::c_int - 1 as libc::c_int) as isize) as libc::c_int
			== ' ' as i32
		{
			i -= 1;
			i;
		}
	}
	*buf.offset(i as isize) = 0 as libc::c_int as libc::c_char;
	if ch as libc::c_int == '\u{1b}' as i32 || i as libc::c_int == 0 as libc::c_int
		|| i as libc::c_int == 1 as libc::c_int && add_blank as libc::c_int != 0
	{
		if !if_cancelled.is_null() {
			message(if_cancelled, 0 as libc::c_int);
		}
		return 0 as libc::c_int;
	}
	return i as libc::c_int;
}

const SAVE_SCREEN_CHAR: libc::c_int = 'X' as libc::c_int;
const REFRESH_SCREEN_CHAR: libc::c_int = 0o022 as libc::c_int;

pub unsafe fn rgetchar() -> libc::c_int {
	let mut done = false;
	let mut ch: libc::c_int = 0;
	while !done {
		ch = libc::getchar();
		match ch {
			REFRESH_SCREEN_CHAR => { wrefresh(curscr); }
			SAVE_SCREEN_CHAR => { save_screen(); }
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
		wclrtoeol(stdscr);
	}
	wrefresh(stdscr);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn pad(s: *const libc::c_char, n: libc::size_t) {
	for _ in libc::strlen(s)..n {
		waddch(stdscr, ' ' as chtype);
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
		i = strlen(str).wrapping_sub(1 as libc::c_int as libc::c_ulong) as libc::c_int;
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
