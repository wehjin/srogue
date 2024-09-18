use crate::resources::level::sector::Sector;
use crate::resources::level::LevelSize;
use crate::room::RoomBounds;

pub const LITTLE_ROOMS: [RoomId; 9] = [
	RoomId::Little(Sector::TopLeft), RoomId::Little(Sector::TopCenter), RoomId::Little(Sector::TopRight),
	RoomId::Little(Sector::MiddleLeft), RoomId::Little(Sector::MiddleCenter), RoomId::Little(Sector::MiddleRight),
	RoomId::Little(Sector::BottomLeft), RoomId::Little(Sector::BottomCenter), RoomId::Little(Sector::BottomRight),
];
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RoomId {
	Big,
	Little(Sector),
}


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