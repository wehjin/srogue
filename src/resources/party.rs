use crate::random::get_rand;

pub struct PartyDepth(usize);

impl PartyDepth {
	pub fn new() -> Self {
		let depth = roll_next_depth(0);
		Self(depth)
	}
	pub fn usize(&self) -> usize {
		self.0
	}
	pub fn recompute(self, level_depth: usize) -> Self {
		if level_depth == self.0 {
			Self(roll_next_depth(self.0))
		} else {
			self
		}
	}
}

fn roll_next_depth(previous_depth: usize) -> usize {
	let base_level = (previous_depth as f32 / PARTY_INTERVAL as f32).ceil() as usize * PARTY_INTERVAL;
	base_level + get_rand(1, PARTY_INTERVAL)
}
const PARTY_INTERVAL: usize = 10;
