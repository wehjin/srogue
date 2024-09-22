use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::setup::{roll_level, LevelKind};
use crate::resources::level::PartyType;
use crate::resources::level::setup::party::depth::PartyDepth;
use crate::resources::rogue::Rogue;

pub fn run() {
	let mut party_depth = PartyDepth::new();
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
		let level = roll_level(&level_kind, &mut dungeon_stats);
		level.map.print();

		// Recompute the party depth depending on the current level.
		party_depth = party_depth.recompute(level.depth);
	}
}
