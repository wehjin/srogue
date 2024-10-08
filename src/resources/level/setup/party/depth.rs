use crate::resources::rogue::depth::RogueDepth;
use rand::Rng;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct PartyDepth(usize);

impl PartyDepth {
	pub fn new(depth: usize) -> Self { Self(depth) }
	pub fn roll(rng: &mut impl Rng) -> Self {
		let depth = roll_next_depth(0, rng);
		Self::new(depth)
	}
	pub fn usize(&self) -> usize {
		self.0
	}

	pub fn roll_next(self, rogue_depth: &RogueDepth, rng: &mut impl Rng) -> Self {
		if rogue_depth.usize() == self.0 {
			Self(roll_next_depth(self.0, rng))
		} else {
			self
		}
	}
}

fn roll_next_depth(previous_depth: usize, rng: &mut impl Rng) -> usize {
	let base_level = (previous_depth as f32 / PARTY_INTERVAL as f32).ceil() as usize * PARTY_INTERVAL;
	base_level + rng.gen_range(1..=PARTY_INTERVAL)
}

const PARTY_INTERVAL: usize = 10;