use crate::level::constants::{DCOLS, DROWS};
use crate::resources::level::maze::LevelMaze;
use crate::resources::level::size::LevelSize;
use crate::room::RoomBounds;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LevelSprite {
	None,
	HorizWall,
	VertWall,
	Floor,
	Tunnel,
	HiddenTunnel,
}

#[derive(Debug)]
pub struct LevelReport {
	pub rows: [[LevelSprite; DCOLS]; DROWS],
}

impl LevelReport {
	pub fn new() -> Self {
		Self { rows: [[LevelSprite::None; DCOLS]; DROWS] }
	}
	pub fn put_tunnels(&mut self, maze: &LevelMaze) {
		for row in maze.rows() {
			for col in maze.cols() {
				let (level_row, level_col) = (LevelSize::from_i64(row), LevelSize::from_i64(col));
				if maze.check_tunnel(level_row, level_col) {
					if maze.check_concealed(level_row, level_col) {
						self.put_sprite(row, col, LevelSprite::HiddenTunnel)
					} else {
						self.put_sprite(row, col, LevelSprite::Tunnel)
					}
				}
			}
		}
	}
	pub fn put_walls_and_floor(&mut self, room: &RoomBounds) {
		for row in room.top..=room.bottom {
			for col in room.left..=room.right {
				if row == room.top || row == room.bottom {
					self.put_sprite(row, col, LevelSprite::HorizWall);
				} else if col == room.left || col == room.right {
					self.put_sprite(row, col, LevelSprite::VertWall);
				} else {
					self.put_sprite(row, col, LevelSprite::Floor);
				}
			}
		}
	}
	fn put_sprite(&mut self, row: i64, col: i64, sprite: LevelSprite) {
		self.rows[row as usize][col as usize] = sprite;
	}

	pub fn print(&self) {
		for row in &self.rows {
			let line = row.iter().map(|sprite| {
				match sprite {
					LevelSprite::None => ' ',
					LevelSprite::HorizWall => '=',
					LevelSprite::VertWall => '|',
					LevelSprite::Floor => '.',
					LevelSprite::Tunnel => '#',
					LevelSprite::HiddenTunnel => 'W',
				}
			}).collect::<String>();
			println!("{}", line);
		}
	}
}