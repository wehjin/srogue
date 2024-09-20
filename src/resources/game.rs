use crate::resources::dungeon::party;
use crate::resources::level::roll_level;
use crate::resources::level::RoomSizing;

pub fn run() {
	let mut party_depth = party::roll_next_depth(0);
	let rogue_depth: usize = 1;
	for _ in 0..1 {
		let room_sizing = if rogue_depth == party_depth { RoomSizing::BigRoll } else { RoomSizing::SmallAlways };
		let level = roll_level(rogue_depth, room_sizing);

		level.map.print();

		if level.depth == party_depth {
			party_depth = party::roll_next_depth(party_depth);
		}
	}
}
