use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::level::design::Design;
use crate::resources::level::feature_grid::FeatureGrid;
use crate::resources::level::maze::hide_random_tunnels;
use crate::resources::level::room::{ExitId, LevelRoom};
use crate::resources::level::sector::{shuffled_sectors, Sector, ALL_SECTORS};
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{deadend, maze};
use crate::room::RoomType;
use deadend::make_deadend;
use maze::make_maze;

#[derive(Debug, Clone)]
pub struct PlainLevel {
	level: usize,
	spaces: [LevelRoom; 9],
	map: FeatureGrid,
}
impl PlainLevel {
	pub fn new(level: usize) -> Self {
		let spaces = [
			LevelRoom::from_sector(Sector::TopLeft),
			LevelRoom::from_sector(Sector::TopCenter),
			LevelRoom::from_sector(Sector::TopRight),
			LevelRoom::from_sector(Sector::MiddleLeft),
			LevelRoom::from_sector(Sector::MiddleCenter),
			LevelRoom::from_sector(Sector::MiddleRight),
			LevelRoom::from_sector(Sector::BottomLeft),
			LevelRoom::from_sector(Sector::BottomCenter),
			LevelRoom::from_sector(Sector::BottomRight),
		];
		Self { level, spaces, map: FeatureGrid::new() }
	}
	pub fn space_mut(&mut self, sector: Sector) -> &mut LevelRoom {
		&mut self.spaces[sector as usize]
	}
	pub fn space(&self, sector: Sector) -> &LevelRoom {
		&self.spaces[sector as usize]
	}
	pub fn into_map(self) -> FeatureGrid {
		self.map
	}
}


impl PlainLevel {
	pub fn add_rooms(self, design: Design) -> Self {
		let PlainLevel { level, mut spaces, mut map, } = self;
		for sector in ALL_SECTORS {
			if !design.requires_room_in_sector(sector) && rand_percent(40) {
				continue;
			} else {
				let space = &mut spaces[sector as usize];
				space.ty = RoomType::Room;
				map = map.put_walls_and_floor(space.bounds);
			}
		}
		Self { level, spaces, map }
	}
	pub fn add_mazes(self) -> Self {
		if self.level > 1 {
			let Self { level, mut spaces, mut map, } = self;
			let maze_percent = (self.level * 5) / 4 + if self.level > 15 { self.level } else { 0 };
			let candidate_sectors = roll_empty_sectors(&spaces, maze_percent);
			for sector in candidate_sectors {
				let maze_bounds = spaces[sector as usize].bounds;
				make_maze(maze_bounds, &mut map);
				hide_random_tunnels(maze_bounds, get_rand(0, 2), self.level, &mut map);
				spaces[sector as usize].ty = RoomType::Maze;
			}
			Self { level, spaces, map }
		} else {
			self
		}
	}

	pub fn add_deadends(self) -> Self {
		let PlainLevel { level, mut spaces, mut map, } = self;
		let mut recursed_sectors = Vec::new();
		let candidate_sectors = shuffled_sectors()
			.into_iter()
			.filter(|sector| {
				match spaces[*sector as usize].ty {
					RoomType::Nothing => true,
					RoomType::Cross if coin_toss() => true,
					_ => false,
				}
			})
			.collect::<Vec<_>>();
		for sector in candidate_sectors {
			let new_recursed = make_deadend(sector, true, level, &mut spaces, &mut map);
			recursed_sectors.extend(new_recursed);
		}
		// Make sure the last recursed deadend connects to a room or maze.
		if let Some(&recursed_sector) = recursed_sectors.last() {
			make_deadend(recursed_sector, false, level, &mut spaces, &mut map);
		}
		Self { level, spaces, map }
	}

	pub fn connect_spaces(self) -> Self {
		let Self { level, mut spaces, mut map, } = self;
		for sector in shuffled_sectors() {
			connect_neighbors(Axis::Horizontal, sector, level, &mut spaces, &mut map);
			connect_neighbors(Axis::Vertical, sector, level, &mut spaces, &mut map);
		}
		Self { level, spaces, map }
	}
}

fn roll_empty_sectors(spaces: &[LevelRoom; 9], percent: usize) -> Vec<Sector> {
	let mut empty_sectors = Vec::new();
	for sector in ALL_SECTORS {
		if spaces[sector as usize].is_nothing() && rand_percent(percent) {
			empty_sectors.push(sector);
		}
	}
	empty_sectors
}


#[derive(Debug)]
pub enum Axis { Horizontal, Vertical }
impl Axis {
	pub fn sort_spots(&self, spot1: LevelSpot, spot2: LevelSpot) -> (LevelSpot, LevelSpot) {
		match self {
			Axis::Horizontal => if spot1.col < spot2.col { (spot1, spot2) } else { (spot2, spot1) },
			Axis::Vertical => if spot1.row < spot2.row { (spot1, spot2) } else { (spot2, spot1) }
		}
	}
}

fn connect_neighbors(axis: Axis, sector: Sector, current_level: usize, spaces: &mut [LevelRoom; 9], map: &mut FeatureGrid) {
	if !spaces[sector as usize].is_vault_or_maze() {
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
				if spaces[far_sector as usize].is_vault_or_maze() {
					connect_spaces(axis, sector, far_sector, current_level, spaces, map);
					spaces[near_sector as usize].ty = RoomType::Cross;
				}
			},
			_ => {}
		}
	}
}

fn connect_spaces(axis: Axis, sector1: Sector, sector2: Sector, current_level: usize, spaces: &mut [LevelRoom; 9], map: &mut FeatureGrid) {
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
	map.put_passage(axis, start, end, current_level);
}

