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


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct LevelSize(pub isize);

impl LevelSize {
	pub fn from_usize(value: usize) -> Self { Self(value as isize) }
	pub fn to_usize(&self) -> usize { self.0 as usize }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct LevelSpot {
	pub row: LevelSize,
	pub col: LevelSize,
}

impl LevelSpot {
	pub fn from_i64(row: i64, col: i64) -> Self {
		Self {
			row: LevelSize(row as isize),
			col: LevelSize(col as isize),
		}
	}
}
