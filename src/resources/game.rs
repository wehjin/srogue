use crate::resources::dungeon::party;
use crate::resources::level::roll_level;
use crate::resources::level::LevelType;

pub fn run() {
	let mut party_depth = party::roll_depth(0);
	let rogue_depth: usize = 1;


	let party_type = if rogue_depth == party_depth { LevelType::PlainAlways } else { LevelType::PartyRoll };
	if party_type == LevelType::PartyRoll {
		party_depth = party::roll_depth(rogue_depth);
	}
	let level = roll_level(rogue_depth, party_type);
	level.map.print();
}
