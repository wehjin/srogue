use crate::level::constants::{DCOLS, DROWS};
use crate::prelude::HIDE_PERCENT;
use crate::random::{get_rand, rand_percent};
use crate::resources::level::maze::hide_random_tunnels;
use crate::resources::level::plain::Axis;
use crate::resources::level::size::LevelSpot;
use crate::room::RoomBounds;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Feature {
	None,
	HorizWall,
	VertWall,
	Floor,
	Tunnel,
	ConcealedTunnel,
	Door,
	ConcealedDoor,
}
impl Feature {
	pub fn is_any_tunnel(&self) -> bool {
		match self {
			Feature::Tunnel | Feature::ConcealedTunnel => true,
			_ => false,
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct LevelMap {
	pub rows: [[Feature; DCOLS]; DROWS],
}

impl LevelMap {
	pub fn new() -> Self {
		Self { rows: [[Feature::None; DCOLS]; DROWS] }
	}
}

impl LevelMap {
	pub fn feature_at(&self, row: usize, col: usize) -> Feature {
		self.rows[row][col]
	}
	pub fn put_feature(&mut self, row: i64, col: i64, feature: Feature) {
		self.rows[row as usize][col as usize] = feature;
	}
}

impl LevelMap {
	pub fn feature_at_spot(&self, spot: LevelSpot) -> Feature {
		let row = spot.row.i64();
		let col = spot.col.i64();
		if row < 0 || row >= 24 || col < 0 || col >= 80 {
			Feature::None
		} else {
			self.rows[row as usize][col as usize]
		}
	}
	pub fn put_feature_at_spot(&mut self, spot: LevelSpot, feature: Feature) {
		self.rows[spot.row.usize()][spot.col.usize()] = feature;
	}
}

impl LevelMap {
	pub fn put_passage(&mut self, axis: Axis, spot1: LevelSpot, spot2: LevelSpot, current_level: usize) {
		let (start, end) = axis.sort_spots(spot1, spot2);
		let (start_row, start_col) = start.i64();
		let (end_row, end_col) = end.i64();
		match axis {
			Axis::Horizontal => {
				let middle_col = get_rand(start_col + 1, end_col - 1);
				for col in (start_col + 1)..middle_col {
					self.put_feature(start_row, col, Feature::Tunnel);
				}
				{
					let (row1, row2) = if start_row <= end_row { (start_row, end_row) } else { (end_row, start_row) };
					for row in row1..=row2 {
						self.put_feature(row, middle_col, Feature::Tunnel);
					}
				}
				for col in (middle_col + 1)..end_col {
					self.put_feature(end_row, col, Feature::Tunnel);
				}
			}
			Axis::Vertical => {
				let middle_row = get_rand(start_row + 1, end_row - 1);
				for row in (start_row + 1)..middle_row {
					self.put_feature(row, start_col, Feature::Tunnel);
				}
				{
					let (col1, col2) = if start_col <= end_col { (start_col, end_col) } else { (end_col, start_col) };
					for col in col1..=col2 {
						self.put_feature(middle_row, col, Feature::Tunnel);
					}
				}
				for row in (middle_row + 1)..end_row {
					self.put_feature(row, end_col, Feature::Tunnel);
				}
			}
		}
		if rand_percent(HIDE_PERCENT) {
			let top = start_row.min(end_row);
			let bottom = start_row.max(end_row);
			let left = start_col.min(end_col);
			let right = start_col.max(end_col);
			let bounds = RoomBounds { top, right, bottom, left };
			hide_random_tunnels(bounds, 1, current_level, self)
		}
	}
}

impl LevelMap {
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
					Feature::HorizWall => '-',
					Feature::VertWall => '|',
					Feature::Floor => '.',
					Feature::Tunnel => '#',
					Feature::ConcealedTunnel => '_',
					Feature::Door => '+',
					Feature::ConcealedDoor => '_',
				}
			}).collect::<String>();
			println!("{}", line);
		}
	}
}