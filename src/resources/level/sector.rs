use crate::level::constants::{DCOLS, DROWS};
use crate::prelude::{COL1, COL2, MIN_ROW, ROW1, ROW2};
use crate::room::RoomBounds;

pub const ALL_SECTORS: [Sector; 9] = [
	Sector::TopLeft,
	Sector::TopCenter,
	Sector::TopRight,
	Sector::MiddleLeft,
	Sector::MiddleCenter,
	Sector::MiddleRight,
	Sector::BottomLeft,
	Sector::BottomCenter,
	Sector::BottomRight,
];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Sector {
	TopLeft = 0,
	TopCenter = 1,
	TopRight = 2,
	MiddleLeft = 3,
	MiddleCenter = 4,
	MiddleRight = 5,
	BottomLeft = 6,
	BottomCenter = 7,
	BottomRight = 8,
}

impl Sector {
	pub fn bounds(&self) -> &'static SectorBounds { &SECTOR_BOUNDS[*self as usize] }
	pub fn is_top(&self) -> bool { if let Sector::TopLeft | Sector::TopCenter | Sector::TopRight = self { true } else { false } }
	pub fn is_middle(&self) -> bool { if let Sector::MiddleLeft | Sector::MiddleCenter | Sector::MiddleRight = self { true } else { false } }
	pub fn is_bottom(&self) -> bool { if let Sector::BottomLeft | Sector::BottomCenter | Sector::BottomRight = self { true } else { false } }
	pub fn is_left(&self) -> bool { if let Sector::TopLeft | Sector::MiddleLeft | Sector::BottomLeft = self { true } else { false } }
	pub fn is_center(&self) -> bool { if let Sector::TopCenter | Sector::MiddleCenter | Sector::BottomCenter = self { true } else { false } }
	pub fn is_right(&self) -> bool { if let Sector::TopRight | Sector::MiddleRight | Sector::BottomRight = self { true } else { false } }
}

pub type SectorBounds = RoomBounds;

pub const COL0: i64 = 0;
pub const COL3: i64 = DCOLS as i64;
pub const ROW0: i64 = MIN_ROW;
pub const ROW3: i64 = DROWS as i64 - 1;
const SECTOR_BOUNDS: [SectorBounds; 9] = [
	SectorBounds { top: ROW0, right: COL1 - 1, bottom: ROW1 - 1, left: COL0 },
	SectorBounds { top: ROW0, right: COL2 - 1, bottom: ROW1 - 1, left: COL1 + 1 },
	SectorBounds { top: ROW0, right: COL3 - 1, bottom: ROW1 - 1, left: COL2 + 1 },
	SectorBounds { top: ROW1 + 1, right: COL1 - 1, bottom: ROW2 - 1, left: COL0 },
	SectorBounds { top: ROW1 + 1, right: COL2 - 1, bottom: ROW2 - 1, left: COL1 + 1 },
	SectorBounds { top: ROW1 + 1, right: COL3 - 1, bottom: ROW2 - 1, left: COL2 + 1 },
	SectorBounds { top: ROW2 + 1, right: COL1 - 1, bottom: ROW3 - 1, left: COL0 },
	SectorBounds { top: ROW2 + 1, right: COL2 - 1, bottom: ROW3 - 1, left: COL1 + 1 },
	SectorBounds { top: ROW2 + 1, right: COL3 - 1, bottom: ROW3 - 1, left: COL2 + 1 },
];

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn bounds() {
		let bounds = Sector::TopLeft.bounds();
		assert_eq!(&SectorBounds { left: 0, right: 25, top: 1, bottom: 6 }, bounds);
	}
}
