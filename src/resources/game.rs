use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::roll_level;
use crate::resources::level::setup::roll_objects;
use crate::resources::level::LevelType;
use crate::resources::party::PartyDepth;
use crate::resources::rogue::depth::RogueDepth;

pub fn run() {
	let mut party_depth = PartyDepth::new();
	let mut rogue_depth = RogueDepth::new(0);
	let mut dungeon_stats = DungeonStats::new();
	for _ in 0..1 {
		// Drop depth to next value.
		rogue_depth = rogue_depth.descend();

		// Build a level.
		let level_type = LevelType::from_depths(&rogue_depth, &party_depth);
		let mut level = roll_level(rogue_depth.usize(), rogue_depth.is_max(), level_type);
		roll_objects(&mut level, &mut dungeon_stats);
		level.map.print();

		// Recompute the party depth depending on the current level.
		party_depth = party_depth.recompute(level.depth);
	}
}
