use crate::random::{get_rand, rand_percent};
use crate::resources::level::design::Design;
use crate::resources::level::map::LevelMap;
use crate::resources::level::maze::{add_random_maze_tunnels, hide_random_maze_tunnels, LevelMaze};
use crate::resources::level::plain::space::{ExitId, SectorSpace};
use crate::resources::level::sector::{shuffled_sectors, Sector, ALL_SECTORS};
use crate::resources::level::size::LevelSpot;
use crate::room::RoomType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlainLevel {
	level: usize,
	spaces: [SectorSpace; 9],
	map: LevelMap,
	mazes: HashMap<Sector, LevelMaze>,
}
impl PlainLevel {
	pub fn new(level: usize) -> Self {
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
		Self { level, spaces, map: LevelMap::new(), mazes: HashMap::new() }
	}
	pub fn space_mut(&mut self, sector: Sector) -> &mut SectorSpace {
		&mut self.spaces[sector as usize]
	}

	pub fn space(&self, sector: Sector) -> &SectorSpace {
		&self.spaces[sector as usize]
	}
	pub fn into_map(self) -> LevelMap {
		self.map
	}
}


impl PlainLevel {
	pub fn add_rooms(self, design: Design) -> Self {
		let PlainLevel { level, mut spaces, mut map, mazes } = self;
		for sector in ALL_SECTORS {
			if !design.requires_room_in_sector(sector) && rand_percent(40) {
				continue;
			} else {
				let space_index = sector as usize;
				spaces[space_index].ty = RoomType::Room;
				map.put_walls_and_floor(&spaces[space_index].bounds);
			}
		}
		Self { level, spaces, map, mazes }
	}
	pub fn add_mazes(self) -> Self {
		if self.level <= 1 {
			self
		} else {
			let Self { level, mut spaces, mut map, mut mazes } = self;
			let maze_percent = (self.level * 5) / 4 + if self.level > 15 { self.level } else { 0 };
			for sector in ALL_SECTORS {
				let space_index = sector as usize;
				if spaces[space_index].is_nothing() && rand_percent(maze_percent) {
					let mut maze = LevelMaze::new(spaces[space_index].bounds.clone());
					add_random_maze_tunnels(&mut maze);
					hide_random_maze_tunnels(get_rand(0, 2), self.level, &mut maze);
					map.put_maze(&maze);
					mazes.insert(sector, maze);
					spaces[space_index].ty = RoomType::Maze;
				}
			}
			Self { level, spaces, map, mazes }
		}
	}

	pub fn connect_spaces(self) -> Self {
		let Self { level, mut spaces, mut map, mazes } = self;
		for sector in shuffled_sectors() {
			connect_neighbors(Axis::Horizontal, sector, level, &mut spaces, &mut map);
			connect_neighbors(Axis::Vertical, sector, level, &mut spaces, &mut map);
		}
		Self { level, spaces, map, mazes }
	}
}

pub enum Axis { Horizontal, Vertical }

fn connect_neighbors(axis: Axis, sector: Sector, current_level: usize, spaces: &mut [SectorSpace; 9], map: &mut LevelMap) {
	if !spaces[sector as usize].is_room_or_maze() {
		return;
	}
	let find_neighbor = match axis {
		Axis::Horizontal => Sector::neighbor_to_right,
		Axis::Vertical => Sector::neighbor_below
	};
	if let Some(near_sector) = find_neighbor(&sector) {
		match spaces[near_sector as usize].ty {
			RoomType::Room | RoomType::Maze => {
				connect_spaces(axis, sector, near_sector, current_level, spaces, map);
			}
			RoomType::Nothing => if let Some(far_sector) = find_neighbor(&near_sector) {
				if spaces[far_sector as usize].is_room_or_maze() {
					connect_spaces(axis, sector, far_sector, current_level, spaces, map);
					spaces[near_sector as usize].ty = RoomType::Cross;
				}
			},
			_ => {}
		}
	}
}

fn connect_spaces(axis: Axis, sector1: Sector, sector2: Sector, current_level: usize, spaces: &mut [SectorSpace; 9], map: &mut LevelMap) {
	let start: LevelSpot;
	let end: LevelSpot;
	match axis {
		Axis::Horizontal => {
			start = spaces[sector1 as usize].put_exit(ExitId::Right, sector2, current_level, map);
			end = spaces[sector2 as usize].put_exit(ExitId::Left, sector1, current_level, map);
		}
		Axis::Vertical => {
			start = spaces[sector1 as usize].put_exit(ExitId::Bottom, sector2, current_level, map);
			end = spaces[sector2 as usize].put_exit(ExitId::Top, sector1, current_level, map);
		}
	}
	map.put_passage(axis, start, end);
}

mod space {
	use crate::prelude::HIDE_PERCENT;
	use crate::random::{get_rand, rand_percent};
	use crate::resources::level::map::{Feature, LevelMap};
	use crate::resources::level::sector::{Sector, SectorBounds};
	use crate::resources::level::size::LevelSpot;
	use crate::room::{RoomBounds, RoomType};

	#[derive(Debug, Copy, Clone)]
	pub struct SectorSpace {
		pub ty: RoomType,
		pub bounds: RoomBounds,
		pub exits: [SpaceExit; 4],
	}

	impl SectorSpace {
		pub fn from_sector(sector: Sector) -> Self {
			let bounds = get_random_room_bounds(&sector.bounds());
			Self { ty: RoomType::Nothing, bounds, exits: [SpaceExit::None; 4] }
		}
		pub fn put_exit(&mut self, exit: ExitId, sector: Sector, current_level: usize, map: &mut LevelMap) -> LevelSpot {
			let wall_width = if self.is_maze() { 0u64 } else { 1 };
			let row: i64;
			let col: i64;
			match exit {
				ExitId::Top | ExitId::Bottom => {
					row = if exit == ExitId::Top { self.bounds.top } else { self.bounds.bottom };
					let search_bounds = self.bounds.inset(0, wall_width);
					'init_col: loop {
						let maybe_col = search_bounds.random_col();
						let feature = map.feature_at(row as usize, maybe_col as usize);
						if feature == Feature::HorizWall || feature == Feature::Tunnel {
							col = maybe_col;
							break 'init_col;
						}
					}
				}
				ExitId::Left | ExitId::Right => {
					col = if exit == ExitId::Right { self.bounds.right } else { self.bounds.left };
					let search_bounds = self.bounds.inset(wall_width, 0);
					'init_row: loop {
						let maybe_row = search_bounds.random_row();
						let feature = map.feature_at(maybe_row as usize, col as usize);
						if feature == Feature::VertWall || feature == Feature::Tunnel {
							row = maybe_row;
							break 'init_row;
						}
					}
				}
			}
			let concealed = current_level > 2 && rand_percent(HIDE_PERCENT);
			if self.ty == RoomType::Room {
				let feature = if concealed { Feature::ConcealedDoor } else { Feature::Door };
				map.put_feature(row, col, feature);
			} else {
				if concealed {
					map.put_feature(row, col, Feature::ConcealedTunnel)
				}
			}
			self.exits[exit as usize] = SpaceExit::Passage { to: sector, row, col };
			LevelSpot::from_i64(row, col)
		}
		pub fn is_nothing(&self) -> bool { self.ty == RoomType::Nothing }
		pub fn is_room(&self) -> bool { self.ty == RoomType::Room }
		pub fn is_maze(&self) -> bool { self.ty == RoomType::Maze }
		pub fn is_room_or_maze(&self) -> bool { self.ty == RoomType::Room || self.ty == RoomType::Maze }
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

	#[derive(Debug, Copy, Clone, Eq, PartialEq)]
	pub enum ExitId {
		Top = 0,
		Right = 1,
		Left = 2,
		Bottom = 3,
	}
	impl ExitId {
		pub fn opposite(&self) -> Self {
			match self {
				ExitId::Top => ExitId::Bottom,
				ExitId::Right => ExitId::Left,
				ExitId::Left => ExitId::Right,
				ExitId::Bottom => ExitId::Top,
			}
		}
	}

	#[derive(Debug, Copy, Clone)]
	pub enum SpaceExit {
		None,
		Passage { to: Sector, row: i64, col: i64 },
	}
}