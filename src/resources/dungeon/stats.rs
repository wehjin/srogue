pub struct DungeonStats {
	pub food_drops: usize,
}

impl DungeonStats {
	pub fn new() -> Self {
		Self {
			food_drops: 0,
		}
	}
}

