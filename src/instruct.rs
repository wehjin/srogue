#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn waddnstr(_: *mut WINDOW, _: *const libc::c_char, _: libc::c_int) -> libc::c_int;

	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn fgets(_: *mut libc::c_char, _: libc::c_int, _: *mut FILE) -> *mut libc::c_char;
	fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
	static mut stdscr: *mut WINDOW;
	fn strchr(_: *const libc::c_char, _: libc::c_int) -> *mut libc::c_char;
}

use ncurses::{clrtoeol, refresh};
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

static mut instructions: *mut libc::c_char = b"/usr/games/rogue.instr\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn Instructions() {
	let mut buffer: [[libc::c_char; 81]; 25] = [[0; 81]; 25];
	let mut buf: [libc::c_char; 256] = [0; 256];
	let mut f: *mut FILE = 0 as *mut FILE;
	let mut row: libc::c_short = 0;
	let mut i: libc::c_int = 0;
	let mut j: libc::c_int = 0;
	f = fopen(instructions, b"r\0" as *const u8 as *const libc::c_char);
	if f.is_null() {
		message("Help file not on line.", 0);
		return;
	}
	row = 0 as libc::c_int as libc::c_short;
	while (row as libc::c_int) < 24 as libc::c_int {
		j = 0 as libc::c_int;
		while j < 80 as libc::c_int {
			buffer[row
				as usize][j
				as usize] = (if wmove(stdscr, row as libc::c_int, j)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int) as chtype
			} else {
				winch(stdscr)
			}) as libc::c_char;
			j += 1;
		}
		buffer[row as usize][j as usize] = 0 as libc::c_int as libc::c_char;
		if wmove(stdscr, row as libc::c_int, 0 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buffer[row as usize].as_mut_ptr(), -(1 as libc::c_int));
		};
		clrtoeol();
		row += 1;
	}
	wmove(stdscr, 0 as libc::c_int, 0 as libc::c_int);
	i = 0 as libc::c_int;
	while i < 24 as libc::c_int {
		wmove(stdscr, i, 0 as libc::c_int);
		clrtoeol();
		i += 1;
	}
	refresh();
	i = 0 as libc::c_int;
	while i < 24 as libc::c_int {
		if fgets(buf.as_mut_ptr(), 250 as libc::c_int, f).is_null() {
			break;
		}
		if !strchr(buf.as_mut_ptr(), '\n' as i32).is_null() {
			*strchr(buf.as_mut_ptr(), '\n' as i32) = 0 as libc::c_int as libc::c_char;
		}
		wmove(stdscr, i, 0 as libc::c_int);
		clrtoeol();
		if wmove(stdscr, i, 0 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buf.as_mut_ptr(), -(1 as libc::c_int));
		};
		i += 1;
	}
	refresh();
	rgetchar();
	wmove(stdscr, 0 as libc::c_int, 0 as libc::c_int);
	clrtoeol();
	i = 0 as libc::c_int;
	while i < 24 as libc::c_int {
		wmove(stdscr, i, 0 as libc::c_int);
		clrtoeol();
		i += 1;
	}
	refresh();
	i = 0 as libc::c_int;
	while i < 24 as libc::c_int {
		if wmove(stdscr, i, 0 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(stdscr, buffer[i as usize].as_mut_ptr(), -(1 as libc::c_int));
		};
		i += 1;
	}
	refresh();
}
