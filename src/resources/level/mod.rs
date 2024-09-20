use crate::random::{get_rand, rand_percent};
use crate::resources::level::map::LevelMap;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::room::RoomBounds;
use design::roll_design;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DungeonLevel {
	pub depth: usize,
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub map: LevelMap,
}
impl DungeonLevel {
	pub fn new(depth: usize) -> Self {
		Self { depth, rooms: HashMap::new(), map: LevelMap::new() }
	}
}

pub fn roll_level(depth: usize, room_sizing: RoomSizing) -> DungeonLevel {
	if roll_big_room(room_sizing) {
		let bounds = RoomBounds {
			top: get_rand(ROW0, ROW0 + 1),
			bottom: get_rand(ROW3 - 6, ROW3 - 1),
			left: get_rand(COL0, 10),
			right: get_rand(COL3 - 11, COL3 - 1),
		};
		let map = LevelMap::new().put_walls_and_floor(bounds);
		let rooms = {
			let mut rooms = HashMap::<RoomId, LevelRoom>::new();
			rooms.insert(RoomId::Big, LevelRoom { bounds });
			rooms
		};
		DungeonLevel { depth, rooms, map }
	} else {
		let design = roll_design();
		let level = PlainLevel::new(depth)
			.add_rooms(design)
			.add_mazes()
			.connect_spaces()
			.add_deadends()
			;
		let rooms = {
			let mut rooms = HashMap::<RoomId, LevelRoom>::new();
			for sector in ALL_SECTORS {
				let space = level.space(sector);
				if space.is_room() {
					rooms.insert(RoomId::Little(sector), LevelRoom { bounds: space.bounds });
				}
			}
			rooms
		};
		let map = level.into_map();
		DungeonLevel { depth, rooms, map }
	}
}

fn roll_big_room(sizing: RoomSizing) -> bool {
	let big_room = match sizing {
		RoomSizing::BigAlways => true,
		RoomSizing::BigRoll if rand_percent(1) => true,
		_ => false,
	};
	big_room
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RoomSizing {
	BigAlways,
	BigRoll,
	SmallAlways,
}


#[derive(Debug)]
pub struct LevelRoom {
	pub bounds: RoomBounds,
}

#[cfg(test)]
mod tests {
	use crate::resources::level::roll_level;
	use crate::resources::level::RoomSizing;

	#[test]
	fn make_level_works() {
		let level = roll_level(16, RoomSizing::BigRoll);
		level.map.print();
	}
}

pub mod design;
pub mod deadend;
pub mod map;
pub mod maze;
pub mod plain;
pub mod room;
pub mod sector;

pub mod size;
