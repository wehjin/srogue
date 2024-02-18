use libc::c_int;
use ncurses::mvaddch;

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::objects::ObjectId;
use crate::prelude::DungeonSpot;
use crate::throw::ThrowEnding;

pub fn detect_all_rows() -> Vec<String> {
	let mut rows = Vec::new();
	for row in 0..DROWS {
		// Read the rows in out of the window.
		let mut chars = Vec::new();
		for col in 0..DCOLS {
			let ch = ncurses::mvinch(row as c_int, col as c_int);
			chars.push(ch as u8);
		}
		rows.push(String::from_utf8(chars).expect("valid utf8"));
	}
	rows
}

pub fn render_all_rows<'a>(f: impl Fn(usize) -> &'a str) {
	for row in 0..DROWS {
		ncurses::mvaddstr(row as i32, 0, f(row));
		ncurses::clrtoeol();
	}
	ncurses::refresh();
}

pub fn get_ch(spot: &DungeonSpot) -> ncurses::chtype {
	let ch_at_spot = ncurses::mvinch(spot.row as c_int, spot.col as c_int);
	ch_at_spot
}

pub fn set_ch(ch: ncurses::chtype, spot: &DungeonSpot) {
	mvaddch(spot.row as i32, spot.col as i32, ch);
}

pub fn swap_ch(ch: ncurses::chtype, spot: &DungeonSpot) -> ncurses::chtype {
	let old_ch = get_ch(spot);
	set_ch(ch, spot);
	old_ch
}

pub fn move_curs(spot: &DungeonSpot) {
	ncurses::mv(spot.row as i32, spot.col as i32);
}

pub fn await_frame() {
	ncurses::refresh();
	ncurses::napms(16);
}

pub fn animate_throw(obj_id: ObjectId, throw_ending: &ThrowEnding, game: &GameState) {
	let what = game.player.object_what(obj_id);
	let ch = ncurses::chtype::from(what.to_char());
	for spot in throw_ending.flight_path() {
		if game.player_can_see(*spot) {
			let restore_ch = swap_ch(ch, spot);
			move_curs(spot);
			await_frame();
			set_ch(restore_ch, spot);
		}
	}
}
