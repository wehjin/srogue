use crate::level::constants::{DCOLS, DROWS};
use crate::resources::level::maze::LevelMaze;
use crate::resources::level::size::LevelSize;
use crate::room::RoomBounds;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Feature {
	None,
	HorizWall,
	VertWall,
	Floor,
	Tunnel,
	ConcealedTunnel,
}

#[derive(Debug, Copy, Clone)]
pub struct LevelMap {
	pub rows: [[Feature; DCOLS]; DROWS],
}

impl LevelMap {
	pub fn new() -> Self {
		Self { rows: [[Feature::None; DCOLS]; DROWS] }
	}
	pub fn put_maze(&mut self, maze: &LevelMaze) {
		for row in maze.rows() {
			for col in maze.cols() {
				let (level_row, level_col) = (LevelSize::from_i64(row), LevelSize::from_i64(col));
				if maze.check_tunnel(level_row, level_col) {
					if maze.check_concealed(level_row, level_col) {
						self.put_sprite(row, col, Feature::ConcealedTunnel)
					} else {
						self.put_sprite(row, col, Feature::Tunnel)
					}
				}
			}
		}
	}
	pub fn put_walls_and_floor(&mut self, room: &RoomBounds) {
		for row in room.top..=room.bottom {
			for col in room.left..=room.right {
				if row == room.top || row == room.bottom {
					self.put_sprite(row, col, Feature::HorizWall);
				} else if col == room.left || col == room.right {
					self.put_sprite(row, col, Feature::VertWall);
				} else {
					self.put_sprite(row, col, Feature::Floor);
				}
			}
		}
	}
	fn put_sprite(&mut self, row: i64, col: i64, sprite: Feature) {
		self.rows[row as usize][col as usize] = sprite;
	}

	pub fn print(&self) {
		for row in &self.rows {
			let line = row.iter().map(|sprite| {
				match sprite {
					Feature::None => ' ',
					Feature::HorizWall => '=',
					Feature::VertWall => '|',
					Feature::Floor => '.',
					Feature::Tunnel => '#',
					Feature::ConcealedTunnel => '_',
				}
			}).collect::<String>();
			println!("{}", line);
		}
	}
}