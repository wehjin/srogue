use crate::level::constants::{DCOLS, DROWS};
use crate::random::{get_rand, rand_percent};
use crate::resources::level::design::{Design, SECTOR_DESIGNS};
use crate::resources::level::room::RoomId;
use crate::resources::level::sector::{SectorBounds, ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::room::RoomBounds;
use rand::seq::SliceRandom;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DungeonLevel {
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub mazes: HashMap<RoomId, LevelMaze>,
}

impl DungeonLevel {
	pub fn to_map(&self) -> LevelMap {
		let mut map = LevelMap::new();
		for (_id, room) in &self.rooms {
			map.put_walls_and_floor(&room.bounds);
		}
		for (_id, maze) in &self.mazes {
			map.put_tunnels(maze);
		}
		map
	}
}

pub mod design;
pub mod room;
pub mod sector;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LevelSprite {
	None,
	HorizWall,
	VertWall,
	Floor,
	Tunnel,
}
#[derive(Debug)]
pub struct LevelMap {
	pub rows: [[LevelSprite; DCOLS]; DROWS],
}
impl LevelMap {
	pub fn new() -> Self {
		Self { rows: [[LevelSprite::None; DCOLS]; DROWS] }
	}
	pub fn put_tunnels(&mut self, maze: &LevelMaze) {
		for d_row in 0..maze.height {
			let row = maze.bounds.top + d_row as i64;
			for d_col in 0..maze.width {
				let col = maze.bounds.left + d_col as i64;
				if maze.tunnels[d_row][d_col] {
					self.put_sprite(row, col, LevelSprite::Tunnel)
				}
			}
		}
	}
	pub fn put_walls_and_floor(&mut self, room: &RoomBounds) {
		for row in room.top..=room.bottom {
			for col in room.left..=room.right {
				if row == room.top || row == room.bottom {
					self.put_sprite(row, col, LevelSprite::HorizWall);
				} else if col == room.left || col == room.right {
					self.put_sprite(row, col, LevelSprite::VertWall);
				} else {
					self.put_sprite(row, col, LevelSprite::Floor);
				}
			}
		}
	}
	fn put_sprite(&mut self, row: i64, col: i64, sprite: LevelSprite) {
		self.rows[row as usize][col as usize] = sprite;
	}

	pub fn print(&self) {
		for row in &self.rows {
			let line = row.iter().map(|sprite| {
				match sprite {
					LevelSprite::None => ' ',
					LevelSprite::HorizWall => '=',
					LevelSprite::VertWall => '|',
					LevelSprite::Floor => '.',
					LevelSprite::Tunnel => '#',
				}
			}).collect::<String>();
			println!("{}", line);
		}
	}
}

fn make_level(current_level: usize, party_level: bool) -> DungeonLevel {
	let mut rooms = HashMap::<RoomId, LevelRoom>::new();
	let mut mazes = HashMap::<RoomId, LevelMaze>::new();

	let design = get_random_level_design(party_level);
	if design == Design::BigRoom {
		let bounds = RoomBounds {
			top: get_rand(ROW0, ROW0 + 1),
			bottom: get_rand(ROW3 - 6, ROW3 - 1),
			left: get_rand(COL0, 10),
			right: get_rand(COL3 - 11, COL3 - 1),
		};
		rooms.insert(RoomId::Big, LevelRoom { bounds });
	} else {
		let mut maze_candidates = HashMap::<RoomId, RoomBounds>::new();
		for sector in ALL_SECTORS {
			let room_id = RoomId::Little(sector);
			let bounds = get_random_room_bounds(&sector.bounds());
			if !design.requires_sector(sector) && rand_percent(40) {
				maze_candidates.insert(room_id, bounds);
			} else {
				rooms.insert(room_id, LevelRoom { bounds });
			}
		}

		if current_level > 1 {
			let maze_percent = (current_level * 5) / 4 + if current_level > 15 { current_level } else { 0 };
			for (id, bounds) in maze_candidates {
				if rand_percent(maze_percent) {
					let maze = LevelMaze::new(bounds);
					mazes.insert(id, maze);
				}
			}
		}
	}
	DungeonLevel { rooms, mazes }
}

const ALL_TUNNEL_DIRS: &[TunnelDir] = &[TunnelDir::Up, TunnelDir::Down, TunnelDir::Left, TunnelDir::Right];
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TunnelDir { Up, Down, Left, Right }

impl TunnelDir {
	pub fn derive_tunnel_spot(&self, row: usize, col: usize, maze: &LevelMaze) -> Option<(usize, usize)> {
		if let Some((check_row, check_col)) = self.to_candidate_spot(row, col, maze) {
			if maze.count_tunnels(check_row, check_col) == 1 {
				Some((check_row, check_col))
			} else {
				None
			}
		} else {
			None
		}
	}
	fn to_candidate_spot(&self, row: usize, col: usize, maze: &LevelMaze) -> Option<(usize, usize)> {
		match self {
			TunnelDir::Up => if row == 0 { None } else { Some((row - 1, col)) },
			TunnelDir::Down => if row == maze.height - 1 { None } else { Some((row + 1, col)) },
			TunnelDir::Left => if col == 0 { None } else { Some((row, col - 1)) },
			TunnelDir::Right => if col == maze.width - 1 { None } else { Some((row, col + 1)) },
		}
	}
}

#[derive(Debug)]
pub struct LevelMaze {
	pub bounds: RoomBounds,
	pub width: usize,
	pub height: usize,
	pub tunnels: Vec<Vec<bool>>,
}

impl LevelMaze {
	pub fn new(bounds: RoomBounds) -> Self {
		let height = dbg!(bounds.rows().count());
		let width = dbg!(bounds.cols().count());
		let mut maze = Self { bounds, width, height, tunnels: vec![vec![false; width]; height] };
		let start_row = get_rand(1, height - 2);
		let start_col = get_rand(1, width - 2);
		maze.add_tunnels(start_row, start_col);
		maze
	}
	fn add_tunnels(&mut self, row: usize, col: usize) {
		self.tunnels[row][col] = true;
		let mut shuffled = ALL_TUNNEL_DIRS.to_vec();
		shuffled.shuffle(&mut rand::thread_rng());
		for tunnel_dir in shuffled {
			let tunnel_spot = tunnel_dir.derive_tunnel_spot(row, col, self);
			if let Some((next_row, next_col)) = tunnel_spot {
				self.add_tunnels(next_row, next_col);
			}
		}
	}
	fn count_tunnels(&self, row: usize, col: usize) -> usize {
		let (row, col) = (row as isize, col as isize);
		let positions = [(row - 1, col), (row, col - 1), (row, col), (row, col + 1), (row + 1, col)];
		let count = positions.iter()
			.map(|&(row, col)| self.tunnel_exists(row, col) as usize)
			.sum();
		count
	}
	fn tunnel_exists(&self, row: isize, col: isize) -> bool {
		if row < 0 || row >= self.height as isize || col < 0 || col >= self.width as isize {
			false
		} else {
			self.tunnels[row as usize][col as usize]
		}
	}
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
		let map = level.to_map();
		map.print();
	}
}