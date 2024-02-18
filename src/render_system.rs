use libc::c_int;
use ncurses::mvaddch;

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::monster::{gmc, MonsterIndex};
use crate::objects::ObjectId;
use crate::prelude::DungeonSpot;
use crate::room::get_dungeon_char_spot;
use crate::throw::ThrowEnding;

pub fn show_monster_gone(spot: DungeonSpot, game: &GameState) {
	let ch = get_dungeon_char_spot(spot, game);
	set_ch(ch, &spot);
}

pub fn show_monster_movement(mon_id: MonsterIndex, start_spot: DungeonSpot, end_spot: DungeonSpot, game: &mut GameState) {
	// Restore char on screen at the abandoned spot.
	{
		let from_ch = game.mash.monster(mon_id).trail_char;
		set_ch(from_ch, &start_spot);
	}
	// Save the char from the entered spot, then write the monster's char to the screen.
	{
		let end_trail_ch = get_ch(&end_spot);
		game.mash.monster_mut(mon_id).trail_char = end_trail_ch;
		if game.level.detect_monster || game.player.can_see_spot(&end_spot, &game.level) {
			let mon_ch = gmc(game.mash.monster(mon_id), &game.player, &game.level);
			set_ch(mon_ch, &end_spot);
		}
	}
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

pub fn get_char(spot: &DungeonSpot) -> char {
	get_ch(spot) as u8 as char
}

pub fn set_char(value: char, spot: &DungeonSpot) {
	set_ch(ncurses::chtype::from(value), spot);
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
