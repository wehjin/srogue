use crate::resources::level::setup::party::depth::PartyDepth;
use crate::resources::rogue::depth::RogueDepth;
use rand::Rng;

#[derive(Debug, Copy, Clone)]
pub struct DungeonStats {
	pub party_depth: PartyDepth,
	pub food_drops: usize,
}

impl DungeonStats {
	pub fn new(rng: &mut impl Rng) -> Self {
		Self {
			party_depth: PartyDepth::roll(rng),
			food_drops: 0,
		}
	}
	pub fn is_party_depth(&self, rogue_depth: &RogueDepth) -> bool {
		rogue_depth.usize() == self.party_depth.usize()
	}
}

