#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;

	fn fgets(_: *mut libc::c_char, _: i64, _: *mut FILE) -> *mut libc::c_char;
	fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
	fn strchr(_: *const libc::c_char, _: i64) -> *mut libc::c_char;
}

use ncurses::{clrtoeol, refresh, waddnstr};
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

static mut instructions: *mut libc::c_char = b"/usr/games/rogue.instr\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn Instructions() {
	let mut buffer: [[libc::c_char; 81]; 25] = [[0; 81]; 25];
	let mut buf: [libc::c_char; 256] = [0; 256];
	let mut f: *mut FILE = 0 as *mut FILE;
	let mut row: libc::c_short = 0;
	let mut i: i64 = 0;
	let mut j: i64 = 0;
	f = fopen(instructions, b"r\0" as *const u8 as *const libc::c_char);
	if f.is_null() {
		message("Help file not on line.", 0);
		return;
	}
	row = 0;
	while (row as i64) < 24 as i64 {
		j = 0 as i64;
		while j < 80 as i64 {
			buffer[row
				as usize][j
				as usize] = (if ncurses::wmove(ncurses::stdscr(), row as i64, j)
				== -(1)
			{
				-(1) as ncurses::chtype
			} else {
				ncurses::winch(ncurses::stdscr())
			}) as libc::c_char;
			j += 1;
		}
		buffer[row as usize][j as usize] = 0 as i64 as libc::c_char;
		if ncurses::wmove(ncurses::stdscr(), row as i64, 0 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buffer[row as usize].as_mut_ptr(), -(1));
		};
		clrtoeol();
		row += 1;
	}
	ncurses::wmove(ncurses::stdscr(), 0 as i64, 0 as i64);
	i = 0 as i64;
	while i < 24 as i64 {
		ncurses::wmove(ncurses::stdscr(), i, 0 as i64);
		clrtoeol();
		i += 1;
	}
	refresh();
	i = 0 as i64;
	while i < 24 as i64 {
		if fgets(buf.as_mut_ptr(), 250 as i64, f).is_null() {
			break;
		}
		if !strchr(buf.as_mut_ptr(), '\n' as i32).is_null() {
			*strchr(buf.as_mut_ptr(), '\n' as i32) = 0 as i64 as libc::c_char;
		}
		ncurses::wmove(ncurses::stdscr(), i, 0 as i64);
		clrtoeol();
		if ncurses::wmove(ncurses::stdscr(), i, 0 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buf.as_mut_ptr(), -(1));
		};
		i += 1;
	}
	refresh();
	rgetchar();
	ncurses::wmove(ncurses::stdscr(), 0 as i64, 0 as i64);
	clrtoeol();
	i = 0 as i64;
	while i < 24 as i64 {
		ncurses::wmove(ncurses::stdscr(), i, 0 as i64);
		clrtoeol();
		i += 1;
	}
	refresh();
	i = 0 as i64;
	while i < 24 as i64 {
		if ncurses::wmove(ncurses::stdscr(), i, 0 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(ncurses::stdscr(), buffer[i as usize].as_mut_ptr(), -(1));
		};
		i += 1;
	}
	refresh();
}
