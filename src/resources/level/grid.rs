use crate::level::constants::{DCOLS, DROWS};
use crate::prelude::MIN_ROW;
use crate::resources::level::size::LevelSpot;
use crate::room::RoomBounds;

#[derive(Debug, Clone)]
pub struct LevelGrid<T> {
	rows: [[T; DCOLS]; DROWS],
}
impl<T: Default + Copy> LevelGrid<T> {
	pub fn new() -> Self {
		Self {
			rows: [[T::default(); DCOLS]; DROWS]
		}
	}
	pub fn value_at(&self, spot: LevelSpot) -> T {
		if Self::RANGE.contains_spot(spot) {
			let (row, col) = spot.usize();
			self.rows[row][col]
		} else {
			T::default()
		}
	}
}
impl<T> LevelGrid<T> {
	pub fn bounds(&self) -> &RoomBounds { &Self::RANGE }
	const RANGE: RoomBounds = RoomBounds { top: MIN_ROW, bottom: (DROWS - 2) as i64, left: 0, right: (DCOLS - 1) as i64 };

	pub fn put_value(&mut self, spot: LevelSpot, value: T) {
		if Self::RANGE.contains_spot(spot) {
			let (row, col) = spot.usize();
			self.rows[row][col] = value;
		}
	}
}
