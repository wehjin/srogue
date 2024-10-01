use crate::motion::MoveResult;
use crate::objects::note_tables::NoteTables;
use crate::player::RogueHealth;
use crate::resources::rogue::depth::RogueDepth;
use crate::ring::effects::RingEffects;
use fighter::Fighter;
use rand::Rng;
use spot::RogueSpot;

pub mod depth;
pub mod fighter;
pub mod spot;

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct Rogue {
	pub has_amulet: bool,
	pub depth: RogueDepth,
	pub spot: RogueSpot,
	pub ring_effects: RingEffects,
	pub fighter: Fighter,
	pub health: RogueHealth,
	pub fight_to_death: Option<u64>,
	pub notes: NoteTables,
	pub wizard: bool,
	pub move_result: Option<MoveResult>,
}

impl Rogue {
	pub fn new(depth: usize) -> Self {
		Self {
			has_amulet: false,
			depth: RogueDepth::new(depth),
			spot: RogueSpot::None,
			ring_effects: Default::default(),
			fighter: Default::default(),
			health: Default::default(),
			fight_to_death: None,
			notes: Default::default(),
			wizard: false,
			move_result: None,
		}
	}
	pub fn outfit(mut self, rng: &mut impl Rng) -> Self {
		self.fighter.provision(rng);
		self.notes.assign_dynamic_titles();
		// TODO ring_stats(false, &mut game);
		self
	}
	pub fn descend(&mut self) {
		self.depth.descend();
		self.spot = RogueSpot::None;
	}
}

