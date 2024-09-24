use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::setup::party::depth::PartyDepth;
use crate::resources::level::setup::{roll_level, LevelKind};
use crate::resources::level::size::LevelSpot;
use crate::resources::level::PartyType;
use crate::resources::rogue::Rogue;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

pub fn run() {
	let rng = &mut ChaChaRng::from_entropy();
	let mut party_depth = PartyDepth::new(rng);
	let mut dungeon_stats = DungeonStats::new();
	let mut rogue = Rogue::default();
	for _ in 0..1 {
		// Drop depth to next value.
		rogue.descend();
		// Build a level.
		let level_kind = LevelKind {
			depth: rogue.depth.usize(),
			is_max: rogue.depth.is_max(),
			post_amulet: rogue.has_amulet,
			party_type: if rogue.depth.usize() == party_depth.usize() { PartyType::PartyRollBig } else { PartyType::NoParty },
		};
		let mut level = roll_level(&level_kind, &mut dungeon_stats, rng);
		level.lighting_enabled = true;
		level.print(false);

		// Recompute the party depth depending on the current level.
		party_depth = party_depth.recompute(level.depth, rng);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum RogueSpot {
	#[default]
	None,
	Vault(LevelSpot, RoomId),
	Passage(LevelSpot),
}
impl RogueSpot {
	pub fn is_spot(&self, spot: LevelSpot) -> bool {
		self.as_level_spot() == Some(&spot)
	}
	pub fn as_level_spot(&self) -> Option<&LevelSpot> {
		match self {
			RogueSpot::None => None,
			RogueSpot::Vault(spot, _) => Some(spot),
			RogueSpot::Passage(spot) => Some(spot),
		}
	}
	pub fn is_in_room(&self, value: RoomId) -> bool {
		match self {
			RogueSpot::None => false,
			RogueSpot::Vault(_, room) => *room == value,
			RogueSpot::Passage(_) => false,
		}
	}
}
