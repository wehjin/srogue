use ncurses::{addch, chtype, clear, clrtoeol, curs_set, curscr, getch, mv, mvaddch, mvaddstr, mvinch, napms, refresh, standend, standout, wrefresh};
use ncurses::CURSOR_VISIBILITY::{CURSOR_INVISIBLE, CURSOR_VISIBLE};

use crate::prelude::DungeonSpot;

pub(crate) fn get_char(spot: DungeonSpot) -> char {
	let ch = mvinch(spot.row as i32, spot.col as i32);
	ch as u8 as char
}

pub(crate) fn set_char(value: char, spot: DungeonSpot) {
	let ch = chtype::from(value);
	mvaddch(spot.row as i32, spot.col as i32, ch);
}

pub(crate) fn set_char_at_cursor(value: char) {
	let ch = chtype::from(value);
	addch(ch);
}

pub(crate) fn set_full_row<T: AsRef<str>>(as_str: T, row: usize) {
	set_str(as_str, (row, 0).into());
	clrtoeol();
}

pub(crate) fn set_str<T: AsRef<str>>(as_str: T, spot: DungeonSpot) {
	mvaddstr(spot.row as i32, spot.col as i32, as_str.as_ref());
}

pub(crate) fn swap_char(value: char, spot: DungeonSpot) -> char {
	let old_char = get_char(spot);
	set_char(value, spot);
	old_char
}

pub(crate) fn clear_to_eol() {
	clrtoeol();
}

pub(crate) fn move_cursor(spot: DungeonSpot) {
	mv(spot.row as i32, spot.col as i32);
}

pub(crate) fn show_cursor(enable: bool) {
	if enable {
		curs_set(CURSOR_VISIBLE);
	} else {
		curs_set(CURSOR_INVISIBLE);
	}
}

pub(crate) fn stand_out(enable: bool) {
	if enable {
		standout();
	} else {
		standend();
	}
}

pub(crate) fn push_screen() {
	refresh();
}

pub(crate) fn erase_screen() {
	clear();
}

pub(crate) fn reload_screen() {
	wrefresh(curscr());
}

pub(crate) fn await_frame() {
	refresh();
	napms(17);
}

pub(crate) fn read_input_char() -> char {
	getch() as u8 as char
}
