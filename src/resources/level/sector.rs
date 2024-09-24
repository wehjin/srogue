use crate::level::constants::{DCOLS, DROWS};
use crate::prelude::{COL1, COL2, MIN_ROW, ROW1, ROW2};
use crate::room::RoomBounds;
use rand::seq::SliceRandom;
use rand::Rng;
use Sector::{BottomCenter, BottomLeft, BottomRight, MiddleCenter, MiddleLeft, MiddleRight, TopCenter, TopLeft, TopRight};

pub const ALL_SECTORS: [Sector; 9] = [
	TopLeft,
	TopCenter,
	TopRight,
	MiddleLeft,
	MiddleCenter,
	MiddleRight,
	BottomLeft,
	BottomCenter,
	BottomRight,
];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

pub fn shuffled_sectors(rng: &mut impl Rng) -> Vec<Sector> {
	let mut sectors = ALL_SECTORS.to_vec();
	sectors.shuffle(rng);
	sectors
}

impl Sector {
	pub fn find_neighbor(&self, neighbor: SectorNeighbor) -> Option<Self> {
		match neighbor {
			SectorNeighbor::Above => self.neighbor_above(),
			SectorNeighbor::Below => self.neighbor_below(),
			SectorNeighbor::Right => self.neighbor_to_right(),
			SectorNeighbor::Left => self.neighbor_to_left(),
		}
	}
}

impl Sector {
	pub fn neighbor_to_right(&self) -> Option<Self> {
		match self {
			TopLeft => Some(TopCenter),
			TopCenter => Some(TopRight),
			TopRight => None,
			MiddleLeft => Some(MiddleCenter),
			MiddleCenter => Some(MiddleRight),
			MiddleRight => None,
			BottomLeft => Some(BottomCenter),
			BottomCenter => Some(BottomRight),
			BottomRight => None,
		}
	}
	pub fn neighbor_to_left(&self) -> Option<Self> {
		match self {
			TopLeft => None,
			TopCenter => Some(TopLeft),
			TopRight => Some(TopCenter),
			MiddleLeft => None,
			MiddleCenter => Some(MiddleLeft),
			MiddleRight => Some(MiddleCenter),
			BottomLeft => None,
			BottomCenter => Some(BottomLeft),
			BottomRight => Some(BottomCenter),
		}
	}
	pub fn neighbor_above(&self) -> Option<Self> {
		match self {
			TopLeft => None,
			TopCenter => None,
			TopRight => None,
			MiddleLeft => Some(TopLeft),
			MiddleCenter => Some(TopCenter),
			MiddleRight => Some(TopRight),
			BottomLeft => Some(MiddleLeft),
			BottomCenter => Some(MiddleCenter),
			BottomRight => Some(MiddleRight),
		}
	}
	pub fn neighbor_below(&self) -> Option<Self> {
		match self {
			TopLeft => Some(MiddleLeft),
			TopCenter => Some(MiddleCenter),
			TopRight => Some(MiddleRight),
			MiddleLeft => Some(BottomLeft),
			MiddleCenter => Some(BottomCenter),
			MiddleRight => Some(BottomRight),
			BottomLeft => None,
			BottomCenter => None,
			BottomRight => None,
		}
	}
}

impl Sector {
	pub fn bounds(&self) -> &'static SectorBounds { &SECTOR_BOUNDS[*self as usize] }
	pub fn is_top(&self) -> bool { if let TopLeft | TopCenter | TopRight = self { true } else { false } }
	pub fn is_middle(&self) -> bool { if let MiddleLeft | MiddleCenter | MiddleRight = self { true } else { false } }
	pub fn is_bottom(&self) -> bool { if let BottomLeft | BottomCenter | BottomRight = self { true } else { false } }
	pub fn is_left(&self) -> bool { if let TopLeft | MiddleLeft | BottomLeft = self { true } else { false } }
	pub fn is_center(&self) -> bool { if let TopCenter | MiddleCenter | BottomCenter = self { true } else { false } }
	pub fn is_right(&self) -> bool { if let TopRight | MiddleRight | BottomRight = self { true } else { false } }
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


pub fn shuffled_sector_neighbors(rng: &mut impl Rng) -> Vec<SectorNeighbor> {
	let mut neighbors = SECTOR_NEIGHBORS.to_vec();
	neighbors.shuffle(rng);
	neighbors
}

pub const SECTOR_NEIGHBORS: &'static [SectorNeighbor] = &[
	SectorNeighbor::Left, SectorNeighbor::Right, SectorNeighbor::Below, SectorNeighbor::Above
];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SectorNeighbor {
	Above,
	Below,
	Right,
	Left,
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn bounds() {
		let bounds = TopLeft.bounds();
		assert_eq!(&SectorBounds { left: 0, right: 25, top: 1, bottom: 6 }, bounds);
	}
}
