use crate::random::{get_rand, rand_percent};
use crate::resources::level::design::{Design, SECTOR_DESIGNS};
use crate::resources::level::map::LevelMap;
use crate::resources::level::maze::{add_random_maze_tunnels, hide_random_maze_tunnels};
use crate::resources::level::room::RoomId;
use crate::resources::level::sector::{SectorBounds, ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::room::RoomBounds;
use maze::LevelMaze;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DungeonLevel {
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub mazes: HashMap<RoomId, LevelMaze>,
	pub map: LevelMap,
}

fn make_level(current_level: usize, party_level: bool) -> DungeonLevel {
	let mut rooms = HashMap::<RoomId, LevelRoom>::new();
	let mut mazes = HashMap::<RoomId, LevelMaze>::new();

	let mut map = LevelMap::new();
	let design = get_random_level_design(party_level);
	if design == Design::BigRoom {
		let bounds = RoomBounds {
			top: get_rand(ROW0, ROW0 + 1),
			bottom: get_rand(ROW3 - 6, ROW3 - 1),
			left: get_rand(COL0, 10),
			right: get_rand(COL3 - 11, COL3 - 1),
		};
		map.put_walls_and_floor(&bounds);
		rooms.insert(RoomId::Big, LevelRoom { bounds });
	} else {
		let mut maze_candidates = HashMap::<RoomId, RoomBounds>::new();
		for sector in ALL_SECTORS {
			let room_id = RoomId::Little(sector);
			let bounds = get_random_room_bounds(&sector.bounds());
			if !design.requires_sector(sector) && rand_percent(40) {
				maze_candidates.insert(room_id, bounds);
			} else {
				map.put_walls_and_floor(&bounds);
				rooms.insert(room_id, LevelRoom { bounds });
			}
		}

		if current_level > 1 {
			let maze_percent = (current_level * 5) / 4 + if current_level > 15 { current_level } else { 0 };
			for (id, bounds) in maze_candidates {
				if rand_percent(maze_percent) {
					let mut maze = LevelMaze::new(bounds.clone());
					add_random_maze_tunnels(&mut maze);
					hide_random_maze_tunnels(get_rand(0, 2), current_level, &mut maze);
					map.put_maze(&maze);
					mazes.insert(id, maze);
				}
			}
		}
	}
	DungeonLevel { rooms, mazes, map }
}

#[derive(Debug)]
pub struct LevelRoom {
	pub bounds: RoomBounds,
}

fn get_random_level_design(party_level: bool) -> Design {
	if party_level && rand_percent(1) { Design::BigRoom } else { SECTOR_DESIGNS[get_rand(0usize, 5)] }
}

fn get_random_room_bounds(sector_bounds: &SectorBounds) -> RoomBounds {
	let height = get_rand(4, sector_bounds.bottom - sector_bounds.top + 1);
	let width = get_rand(8, sector_bounds.right - sector_bounds.left - 2);
	let row_offset = get_rand(0, (sector_bounds.bottom - sector_bounds.top) - height + 1);
	let col_offset = get_rand(0, (sector_bounds.right - sector_bounds.left) - width + 1);

	let top = sector_bounds.top + row_offset;
	let bottom = top + height - 1;
	let left = sector_bounds.left + col_offset;
	let right = left + width - 1;
	RoomBounds { top, right, bottom, left }
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
pub mod room;
pub mod sector;
pub mod size;
