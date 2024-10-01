use crate::prelude::DungeonSpot;
use crate::room::RoomBounds;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct RoomSize(pub isize);

impl RoomSize {
	pub fn to_usize(&self) -> usize { self.0 as usize }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct RoomSpot(RoomSize, RoomSize);

impl RoomSpot {
	pub fn new(row: RoomSize, col: RoomSize) -> Self {
		Self(row, col)
	}
	pub fn from_level_sizes(level_row: LevelSize, level_col: LevelSize, room_bounds: &RoomBounds) -> Self {
		let room_row = level_row.to_room_row(room_bounds);
		let room_col = level_col.to_room_col(room_bounds);
		Self::new(room_row, room_col)
	}
}

impl RoomSpot {
	pub fn as_row(&self) -> &RoomSize { &self.0 }
	pub fn as_col(&self) -> &RoomSize { &self.1 }
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct LevelSize(pub isize);

impl LevelSize {
	pub fn from_usize(value: usize) -> Self { Self(value as isize) }
	pub fn usize(&self) -> usize { self.0 as usize }
}

impl LevelSize {
	pub fn from_i64(value: i64) -> Self { Self(value as isize) }
	pub fn i64(&self) -> i64 { self.0 as i64 }
}

impl LevelSize {
	pub fn to_room_row(&self, room_bounds: &RoomBounds) -> RoomSize {
		RoomSize(self.0 - room_bounds.top as isize)
	}
	pub fn to_room_col(&self, room_bounds: &RoomBounds) -> RoomSize {
		RoomSize(self.0 - room_bounds.left as isize)
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct LevelSpot {
	pub row: LevelSize,
	pub col: LevelSize,
}

impl LevelSpot {
	pub fn new(row: LevelSize, col: LevelSize) -> Self {
		Self { row, col }
	}
	pub fn has_same_row(&self, other: LevelSpot) -> bool {
		self.row.i64() == other.row.i64()
	}
	pub fn has_same_col(&self, other: LevelSpot) -> bool {
		self.col.i64() == other.col.i64()
	}
	pub fn has_same_row_or_col(&self, other: LevelSpot) -> bool {
		self.has_same_row(other) || self.has_same_col(other)
	}
}
impl From<DungeonSpot> for LevelSpot {
	fn from(value: DungeonSpot) -> Self {
		let DungeonSpot { row, col } = value;
		Self::from_i64(row, col)
	}
}

impl From<(i64, i64)> for LevelSpot {
	fn from(value: (i64, i64)) -> Self {
		Self::from_i64(value.0, value.1)
	}
}

impl Into<(i64, i64)> for LevelSpot {
	fn into(self) -> (i64, i64) { self.i64() }
}

impl LevelSpot {
	pub fn i64(&self) -> (i64, i64) {
		(self.row.0 as i64, self.col.0 as i64)
	}
	pub const fn from_i64(row: i64, col: i64) -> Self {
		Self {
			row: LevelSize(row as isize),
			col: LevelSize(col as isize),
		}
	}
}

impl LevelSpot {
	pub fn usize(&self) -> (usize, usize) {
		(self.row.0 as usize, self.col.0 as usize)
	}
	pub fn from_usize(row: usize, col: usize) -> Self {
		Self {
			row: LevelSize(row as isize),
			col: LevelSize(col as isize),
		}
	}
}

impl LevelSpot {
	pub fn with_axial_neighbors(&self) -> [LevelSpot; 5] {
		let center_row = self.row.i64();
		let center_col = self.col.i64();
		[
			LevelSpot::from_i64(center_row - 1, center_col),
			LevelSpot::from_i64(center_row, center_col - 1),
			LevelSpot::from_i64(center_row, center_col),
			LevelSpot::from_i64(center_row, center_col + 1),
			LevelSpot::from_i64(center_row + 1, center_col)
		]
	}
}
