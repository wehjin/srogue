use crate::objects::ObjectPack;
use crate::resources::rogue::depth::RogueDepth;
use crate::ring::effects::RingEffects;
use fighter::Fighter;
use rand::Rng;
use spot::RogueSpot;

pub mod depth;
pub mod fighter;
pub mod spot;

#[derive(Debug, Clone, Default, Eq, Hash, PartialEq)]
pub struct Rogue {
	pub has_amulet: bool,
	pub depth: RogueDepth,
	pub spot: RogueSpot,
	pub ring_effects: RingEffects,
	pub fighter: Fighter,
}

impl Rogue {
	pub fn as_pack(&self) -> &ObjectPack {
		&self.fighter.pack
	}
}

impl Rogue {
	pub fn new(depth: usize) -> Self {
		Self {
			has_amulet: false,
			depth: RogueDepth::new(depth),
			spot: RogueSpot::None,
			ring_effects: Default::default(),
			fighter: Default::default(),
		}
	}
	pub fn outfit(mut self, rng: &mut impl Rng) -> Self {
		self.fighter.provision(rng);
		self
	}
	pub fn descend(&mut self) {
		self.depth.descend();
		self.spot = RogueSpot::None;
	}
}

