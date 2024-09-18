use crate::random::{get_rand, rand_percent};
use crate::resources::level::design::{Design, SECTOR_DESIGNS};
use crate::resources::level::map::LevelMap;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::room::RoomBounds;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DungeonLevel {
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub map: LevelMap,
}

fn make_level(current_level: usize, party_level: bool) -> DungeonLevel {
	let mut rooms = HashMap::<RoomId, LevelRoom>::new();
	let level_map: LevelMap;

	let design = get_random_level_design(party_level);
	if design == Design::BigRoom {
		let mut map = LevelMap::new();
		let bounds = RoomBounds {
			top: get_rand(ROW0, ROW0 + 1),
			bottom: get_rand(ROW3 - 6, ROW3 - 1),
			left: get_rand(COL0, 10),
			right: get_rand(COL3 - 11, COL3 - 1),
		};
		map.put_walls_and_floor(&bounds);
		rooms.insert(RoomId::Big, LevelRoom { bounds });
		level_map = map;
	} else {
		let level = PlainLevel::new().add_rooms(design).add_mazes(current_level)
			;
		for sector in ALL_SECTORS {
			let space = level.space(sector);
			if space.is_room() {
				rooms.insert(RoomId::Little(sector), LevelRoom { bounds: space.bounds.clone() });
			}
		}
		level_map = level.into_map();
	}
	DungeonLevel { rooms, map: level_map }
}

#[derive(Debug)]
pub struct LevelRoom {
	pub bounds: RoomBounds,
}

fn get_random_level_design(party_level: bool) -> Design {
	if party_level && rand_percent(1) { Design::BigRoom } else { SECTOR_DESIGNS[get_rand(0usize, 5)] }
}


#[cfg(test)]
mod tests {
	use crate::resources::level::make_level;

	#[test]
	fn make_level_works() {
		let level = make_level(16, false);
		level.map.print();
	}
}

pub mod design;
pub mod map;
pub mod maze;
pub mod plain;
pub mod room;
pub mod sector;
pub mod size;
