pub use constants::*;

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::player::RoomMark;
use crate::prelude::DungeonSpot;
use crate::render_system::appearance::appearance_for_spot;
use crate::render_system::backend::get_char;
use crate::render_system::stats::format_stats;
use crate::room::RoomBounds;

pub(crate) mod animation;
pub(crate) mod appearance;
pub(crate) mod backend;
pub(crate) mod constants;
pub(crate) mod hallucinate;
pub(crate) mod stats;


#[derive(Copy, Clone)]
pub enum RenderAction {
	Spot(DungeonSpot),
	MonstersFloorAndPlayer,
	RoomAndPlayer(usize),
	Room(usize),
	Init,
}

pub fn detect_all_rows() -> Vec<String> {
	let mut rows = Vec::new();
	for row in 0..DROWS as i64 {
		// Read the rows in out of the window.
		let mut chars = Vec::new();
		for col in 0..DCOLS as i64 {
			let spot = DungeonSpot { row, col };
			let char = get_char(spot);
			chars.push(char);
		}
		rows.push(chars.iter().collect());
	}
	rows
}

pub fn render_all_rows<'a>(f: impl Fn(usize) -> &'a str) {
	for row in 0..DROWS {
		backend::set_row_eol(f(row), row);
	}
	backend::push_screen();
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

pub(crate) fn show_darkened_room_after_player_exit(vacated_spot: DungeonSpot, game: &mut GameState) {
	game.render_spot(vacated_spot);
	if let RoomMark::Cavern(rn) = game.level.room_at_spot(vacated_spot) {
		darken_room(rn, game);
	}
}

pub(crate) fn darken_room(rn: usize, game: &mut GameState) {
	game.render(&[RenderAction::Room(rn)]);
}


pub fn refresh(game: &mut GameState) {
	if game.stats_changed {
		const STATS_ROW: i32 = DROWS as i32 - 1;
		backend::set_row_eol(&format_stats(&game.player), STATS_ROW as usize);
		game.stats_changed = false;
	}
	if game.render_queue.len() > 0 {
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
				backend::show_cursor(false);
				for row in bounds.rows() {
					for col in bounds.cols() {
						let spot = DungeonSpot { row, col };
						let appearance = appearance_for_spot(spot, game);
						let char = appearance.to_char();
						backend::set_char(char, spot);
					}
				}
				backend::set_char(PLAYER_CHAR, game.player.to_spot());
				backend::show_cursor(true);
			}
			Some(_) | None => {}
		}
		game.render_queue.clear();
	}
	backend::move_cursor(game.player.to_spot());
	backend::push_screen();
}
