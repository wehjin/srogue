use crate::random::{get_rand, rand_percent};
use crate::resources::level::size::RoomSpot;
use crate::resources::level::size::LevelSize;
use crate::room::RoomBounds;
use rand::prelude::SliceRandom;
use std::collections::HashSet;
use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct LevelMaze {
	pub bounds: RoomBounds,
	pub width: usize,
	pub height: usize,
	pub tunnels: Vec<Vec<bool>>,
	pub concealed_tunnel_spots: HashSet<RoomSpot>,
}

impl LevelMaze {
	pub fn rows(&self) -> RangeInclusive<i64> { self.bounds.rows() }
	pub fn cols(&self) -> RangeInclusive<i64> { self.bounds.cols() }
}

impl LevelMaze {
	pub fn set_concealed(&mut self, level_row: LevelSize, level_col: LevelSize) {
		let room_spot = self.get_room_spot(level_row, level_col);
		self.concealed_tunnel_spots.insert(room_spot);
	}
	pub fn check_concealed(&self, level_row: LevelSize, level_col: LevelSize) -> bool {
		let room_spot = self.get_room_spot(level_row, level_col);
		self.concealed_tunnel_spots.contains(&room_spot)
	}
}

impl LevelMaze {
	pub fn check_tunnel(&self, level_row: LevelSize, level_col: LevelSize) -> bool {
		let room_spot = self.get_room_spot(level_row, level_col);
		let tunnels_row = room_spot.as_row().to_usize();
		let tunnels_col = room_spot.as_col().to_usize();
		self.tunnels[tunnels_row][tunnels_col]
	}


	fn get_room_spot(&self, level_row: LevelSize, level_col: LevelSize) -> RoomSpot {
		let room_spot = RoomSpot::from_level_sizes(level_row, level_col, &self.bounds);
		room_spot
	}
}

impl LevelMaze {
	pub fn new(bounds: RoomBounds) -> Self {
		let height = dbg!(bounds.rows().count());
		let width = dbg!(bounds.cols().count());
		Self {
			bounds,
			width,
			height,
			tunnels: vec![vec![false; width]; height],
			concealed_tunnel_spots: HashSet::new(),
		}
	}

	fn add_tunnels_from_spot(&mut self, row: usize, col: usize) {
		self.tunnels[row][col] = true;
		let mut shuffled = ALL_MAZE_STEPS.to_vec();
		shuffled.shuffle(&mut rand::thread_rng());
		for tunnel_dir in shuffled {
			let tunnel_spot = tunnel_dir.derive_tunnel_spot(row, col, self);
			if let Some((next_row, next_col)) = tunnel_spot {
				self.add_tunnels_from_spot(next_row, next_col);
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

pub fn add_random_maze_tunnels(maze: &mut LevelMaze) {
	let start_row = get_rand(1, maze.height - 2);
	let start_col = get_rand(1, maze.width - 2);
	add_random_maze_tunnels_from_spot(start_row, start_col, maze);
}

fn add_random_maze_tunnels_from_spot(row: usize, col: usize, maze: &mut LevelMaze) {
	maze.tunnels[row][col] = true;
	for step in maze_steps_with_random_shuffle(33) {
		let tunnel_spot = step.derive_tunnel_spot(row, col, maze);
		if let Some((next_row, next_col)) = tunnel_spot {
			maze.add_tunnels_from_spot(next_row, next_col);
		}
	}
}

fn maze_steps_with_random_shuffle(percentage: usize) -> Vec<MazeStep> {
	let mut shuffled = ALL_MAZE_STEPS.to_vec();
	if rand_percent(percentage) {
		shuffled.shuffle(&mut rand::thread_rng());
	}
	shuffled
}

const ALL_MAZE_STEPS: &[MazeStep] = &[MazeStep::Up, MazeStep::Down, MazeStep::Left, MazeStep::Right];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MazeStep { Up, Down, Left, Right }

impl MazeStep {
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
			MazeStep::Up => if row == 0 { None } else { Some((row - 1, col)) },
			MazeStep::Down => if row == maze.height - 1 { None } else { Some((row + 1, col)) },
			MazeStep::Left => if col == 0 { None } else { Some((row, col - 1)) },
			MazeStep::Right => if col == maze.width - 1 { None } else { Some((row, col + 1)) },
		}
	}
}
