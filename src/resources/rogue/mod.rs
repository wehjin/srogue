use crate::resources::dungeon::RogueSpot;
use crate::resources::rogue::depth::RogueDepth;
use crate::ring::effects::RingEffects;

pub mod depth;


#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct Rogue {
	pub has_amulet: bool,
	pub depth: RogueDepth,
	pub spot: RogueSpot,
	pub ring_effects: RingEffects,
}

impl Rogue {
	pub fn new(depth: usize) -> Self {
		Self {
			has_amulet: false,
			depth: RogueDepth::new(depth),
			spot: RogueSpot::None,
			ring_effects: Default::default(),
		}
	}
	pub fn descend(&mut self) {
		self.depth.descend();
		self.spot = RogueSpot::None;
	}
}
