use crate::player::LAST_DUNGEON;
use crate::resources::dungeon::party;
use crate::resources::level::roll_level;
use crate::resources::level::RoomSizing;

pub fn run() {
	let mut party_depth = party::roll_next_depth(0);
	let mut depth: usize = 0;
	let mut max_depth: usize = 0;
	for _ in 0..1 {
		if depth < LAST_DUNGEON as usize {
			depth += 1;
			max_depth = max_depth.max(depth);
		}
		let room_sizing = if depth == party_depth { RoomSizing::BigRoll } else { RoomSizing::SmallAlways };
		let level = roll_level(depth, room_sizing);
		level.map.print();

		if level.depth == party_depth {
			party_depth = party::roll_next_depth(party_depth);
		}
	}
}
