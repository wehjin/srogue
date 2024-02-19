use libc::c_int;
use ncurses::{chtype, mvaddch};
use ncurses::CURSOR_VISIBILITY::{CURSOR_INVISIBLE, CURSOR_VISIBLE};

pub use constants::*;
pub use hallucinate::*;

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::objects::ObjectId;
use crate::player::RoomMark;
use crate::prelude::DungeonSpot;
use crate::random::get_rand;
use crate::render_system::appearance::appearance_for_spot;
use crate::room::RoomBounds;
use crate::throw::ThrowEnding;

mod hallucinate;
pub(crate) mod constants;
pub(crate) mod appearance;

#[derive(Copy, Clone)]
pub enum RenderAction {
	Spot(DungeonSpot),
	MonstersFloorAndPlayer,
	RoomAndPlayer(usize),
	Room(usize),
	Init,
}

pub fn gr_obj_char() -> char {
	let index = get_rand(0, DISGUISE_CHARS.len() - 1);
	let char = DISGUISE_CHARS[index];
	char
}

pub fn gr_obj_ch() -> chtype { chtype::from(gr_obj_char()) }

pub fn erase_screen() {
	ncurses::clear();
}

pub fn animate_throw(obj_id: ObjectId, throw_ending: &ThrowEnding, game: &GameState) {
	let what = game.player.object_what(obj_id);
	let ch = ncurses::chtype::from(what.to_char());
	for spot in throw_ending.flight_path() {
		if game.player_can_see(*spot) {
			let restore_ch = swap_ch(ch, *spot);
			move_curs(spot);
			await_frame();
			set_ch(restore_ch, *spot);
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

fn set_char(value: char, spot: DungeonSpot) {
	set_ch(ncurses::chtype::from(value), spot);
}

fn get_ch(spot: DungeonSpot) -> chtype {
	let ch_at_spot = ncurses::mvinch(spot.row as c_int, spot.col as c_int);
	ch_at_spot
}

fn set_ch(ch: chtype, spot: DungeonSpot) {
	mvaddch(spot.row as i32, spot.col as i32, ch);
}

pub fn swap_ch(ch: chtype, spot: DungeonSpot) -> chtype {
	let old_ch = get_ch(spot);
	set_ch(ch, spot);
	old_ch
}

pub fn move_curs(spot: &DungeonSpot) {
	ncurses::mv(spot.row as i32, spot.col as i32);
}

struct ExpanderBounds {
	top_row: i64,
	right_col: i64,
	bottom_row: i64,
	left_col: i64,
}

impl Default for ExpanderBounds {
	fn default() -> Self {
		Self {
			top_row: DROWS as i64 - 2,
			right_col: 0,
			bottom_row: 1,
			left_col: DCOLS as i64 - 1,
		}
	}
}

impl ExpanderBounds {
	fn expand_spot(&mut self, spot: DungeonSpot) {
		self.top_row = self.top_row.min(spot.row);
		self.bottom_row = self.bottom_row.max(spot.row);
		self.left_col = self.left_col.min(spot.col);
		self.right_col = self.right_col.max(spot.col);
	}
	fn into_dungeon_bounds(mut self) -> Option<RoomBounds> {
		self.top_row = self.top_row.max(0);
		self.left_col = self.left_col.max(0);
		self.bottom_row = self.bottom_row.min(DROWS as i64 - 2);
		self.right_col = self.right_col.min(DCOLS as i64 - 1);
		if self.right_col >= self.left_col && self.bottom_row >= self.top_row {
			let bounds = RoomBounds {
				top: self.top_row,
				right: self.right_col,
				bottom: self.bottom_row,
				left: self.left_col,
			};
			Some(bounds)
		} else {
			None
		}
	}
}


pub fn refresh(game: &mut GameState) {
	let mut expander_bounds = ExpanderBounds::default();
	let mut use_dungeon_bounds = false;
	for action in &game.render_queue {
		match action {
			RenderAction::Spot(spot) => {
				expander_bounds.expand_spot(*spot);
			}
			RenderAction::MonstersFloorAndPlayer
			| RenderAction::RoomAndPlayer(_)
			| RenderAction::Room(_)
			| RenderAction::Init => {
				use_dungeon_bounds = true;
				break;
			}
		}
	}
	let bounds = if use_dungeon_bounds {
		Some(game.dungeon_bounds())
	} else {
		expander_bounds.into_dungeon_bounds()
	};
	match bounds {
		Some(bounds) if bounds.area() >= 1 => {
			ncurses::curs_set(CURSOR_INVISIBLE);
			for row in bounds.rows() {
				for col in bounds.cols() {
					let spot = DungeonSpot { row, col };
					let appearance = appearance_for_spot(spot, game);
					let char = appearance.to_char();
					set_char(char, spot);
				}
			}
			set_char(PLAYER_CHAR, game.player.to_spot());
			ncurses::curs_set(CURSOR_VISIBLE);
		}
		Some(_) | None => {}
	}
	game.render_queue.clear();
	ncurses::refresh();
}

pub fn await_frame() {
	ncurses::refresh();
	ncurses::napms(17);
}

pub(crate) fn show_darkened_room_after_player_exit(vacated_spot: DungeonSpot, game: &mut GameState) {
	game.render_spot(vacated_spot);
	if let RoomMark::Cavern(rn) = game.level.room_at_spot(vacated_spot) {
		darken_room(rn, game);
	}
}

pub(crate) fn darken_room(rn: usize, game: &mut GameState) {
	game.render(&[RenderAction::Room(rn)]);
}