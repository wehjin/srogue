use crate::random::{get_rand, rand_percent};
use crate::resources::level::design::Design;
use crate::resources::level::map::LevelMap;
use crate::resources::level::maze::{add_random_maze_tunnels, hide_random_maze_tunnels, LevelMaze};
use crate::resources::level::plain::space::SectorSpace;
use crate::resources::level::sector::{Sector, ALL_SECTORS};
use crate::room::RoomType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlainLevel {
	spaces: [SectorSpace; 9],
	map: LevelMap,
	mazes: HashMap<Sector, LevelMaze>,
}
impl PlainLevel {
	pub fn new() -> Self {
		let spaces = [
			SectorSpace::from_sector(Sector::TopLeft),
			SectorSpace::from_sector(Sector::TopCenter),
			SectorSpace::from_sector(Sector::TopRight),
			SectorSpace::from_sector(Sector::MiddleLeft),
			SectorSpace::from_sector(Sector::MiddleCenter),
			SectorSpace::from_sector(Sector::MiddleRight),
			SectorSpace::from_sector(Sector::BottomLeft),
			SectorSpace::from_sector(Sector::BottomCenter),
			SectorSpace::from_sector(Sector::BottomRight),
		];
		Self { spaces, map: LevelMap::new(), mazes: HashMap::new() }
	}
	pub fn into_map(self) -> LevelMap {
		self.map
	}

	pub fn space_mut(&mut self, sector: Sector) -> &mut SectorSpace {
		&mut self.spaces[sector as usize]
	}
	pub fn space(&self, sector: Sector) -> &SectorSpace {
		&self.spaces[sector as usize]
	}
}


impl PlainLevel {
	pub fn add_rooms(self, design: Design) -> Self {
		let PlainLevel { mut spaces, mut map, mazes } = self;
		for sector in ALL_SECTORS {
			if !design.requires_room_in_sector(sector) && rand_percent(40) {
				continue;
			} else {
				let space_index = sector as usize;
				spaces[space_index].ty = RoomType::Room;
				map.put_walls_and_floor(&spaces[space_index].bounds);
			}
		}
		Self { spaces, map, mazes }
	}
	pub fn add_mazes(self, current_level: usize) -> Self {
		if current_level <= 1 {
			self
		} else {
			let Self { mut spaces, mut map, mut mazes } = self;
			let maze_percent = (current_level * 5) / 4 + if current_level > 15 { current_level } else { 0 };
			for sector in ALL_SECTORS {
				let space_index = sector as usize;
				if spaces[space_index].is_nothing() && rand_percent(maze_percent) {
					let mut maze = LevelMaze::new(spaces[space_index].bounds.clone());
					add_random_maze_tunnels(&mut maze);
					hide_random_maze_tunnels(get_rand(0, 2), current_level, &mut maze);
					map.put_maze(&maze);
					mazes.insert(sector, maze);
					spaces[space_index].ty = RoomType::Maze;
				}
			}
			Self { spaces, map, mazes }
		}
	}
}

mod space {
	use crate::random::get_rand;
	use crate::resources::level::sector::{Sector, SectorBounds};
	use crate::room::{RoomBounds, RoomType};

	#[derive(Debug, Copy, Clone)]
	pub struct SectorSpace {
		pub ty: RoomType,
		pub bounds: RoomBounds,
	}

	impl SectorSpace {
		pub fn from_sector(sector: Sector) -> Self {
			let bounds = get_random_room_bounds(&sector.bounds());
			Self { ty: RoomType::Nothing, bounds }
		}
		pub fn is_nothing(&self) -> bool {
			self.ty == RoomType::Nothing
		}
		pub fn is_room(&self) -> bool {
			self.ty == RoomType::Room
		}
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
}