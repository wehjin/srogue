use crate::monster::{MonsterKind, MONSTERS};
use crate::prelude::DungeonSpot;
use crate::resources::level::size::{LevelSize, LevelSpot};
use crate::room::RoomBounds;
use rand::Rng;

impl RoomBounds {
	pub fn roll_spot(&self, rng: &mut impl Rng) -> LevelSpot {
		let row = self.to_random_row(rng);
		let col = self.to_random_col(rng);
		LevelSpot::new(row, col)
	}
	pub fn to_random_row(&self, rng: &mut impl Rng) -> LevelSize {
		LevelSize(self.random_row(rng) as isize)
	}
	pub fn to_random_col(&self, rng: &mut impl Rng) -> LevelSize {
		LevelSize(self.random_col(rng) as isize)
	}
	pub fn to_random_spot(&self, rng: &mut impl Rng) -> DungeonSpot {
		let row = self.random_row(rng);
		let col = self.random_col(rng);
		(row, col).into()
	}
	pub fn random_row(&self, rng: &mut impl Rng) -> i64 {
		rng.gen_range(self.top..=self.bottom)
	}
	pub fn random_col(&self, rng: &mut impl Rng) -> i64 {
		rng.gen_range(self.left..=self.right)
	}
}
impl MonsterKind {
	pub fn random_any(rng: &mut impl Rng) -> Self {
		let y = MONSTERS - 1;
		Self::LIST[rng.gen_range(0..=y)]
	}
}
