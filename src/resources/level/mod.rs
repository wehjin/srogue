use crate::random::get_rand;
use crate::resources::level::design::Design;
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

pub fn roll_level(depth: usize, level_type: LevelType) -> DungeonLevel {
	let design = roll_design(level_type);
	if design == Design::BigRoom {
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum LevelType {
	PlainAlways,
	PartyRoll,
	PartyAlways,
}


#[derive(Debug)]
pub struct LevelRoom {
	pub bounds: RoomBounds,
}

#[cfg(test)]
mod tests {
	use crate::resources::level::roll_level;
	use crate::resources::level::LevelType;

	#[test]
	fn make_level_works() {
		let level = roll_level(16, LevelType::PlainAlways);
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
