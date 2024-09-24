use crate::resources::level::size::{LevelSize, LevelSpot};
use crate::room::RoomBounds;
use rand::Rng;

impl RoomBounds {
	pub fn roll_spot(&self, rng: &mut impl Rng) -> LevelSpot {
		let row = self.to_random_row(rng);
		let col = self.to_random_col(rng);
		LevelSpot::new(row, col)
	}
}
impl RoomBounds {
	pub fn to_random_row(&self, rng: &mut impl Rng) -> LevelSize {
		let x = self.top;
		let y = self.bottom;
		LevelSize(rng.gen_range(x..=y) as isize)
	}
	pub fn to_random_col(&self, rng: &mut impl Rng) -> LevelSize {
		let x = self.left;
		let y = self.right;
		LevelSize(rng.gen_range(x..=y) as isize)
	}
}