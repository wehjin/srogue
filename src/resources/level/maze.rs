use crate::random::rand_percent;
use crate::resources::level::map::feature::Feature;
use crate::resources::level::map::LevelMap;
use crate::resources::level::size::LevelSpot;
use crate::room::RoomBounds;
use rand::prelude::SliceRandom;

pub fn make_maze(bounds: RoomBounds, map: &mut LevelMap) {
	let start_spot = bounds.inset(1, 1).to_random_level_spot();
	make_maze_from_spot(start_spot, bounds, map);
}

fn make_maze_from_spot(spot: LevelSpot, bounds: RoomBounds, map: &mut LevelMap) {
	map.put_feature_at_spot(spot, Feature::Tunnel);
	let maze_steps = if rand_percent(33) {
		let mut steps = ALL_MAZE_STEPS.to_vec();
		steps.shuffle(&mut rand::thread_rng());
		steps
	} else {
		ALL_MAZE_STEPS.to_vec()
	};
	for maze_step in maze_steps {
		if let Some(new_tunnel_spot) = maze_step.find_new_tunnel_spot(spot, bounds, map) {
			make_maze_from_spot(new_tunnel_spot, bounds, map);
		}
	}
}

pub fn hide_random_tunnels(bounds: RoomBounds, count: usize, current_level: usize, map: &mut LevelMap) {
	if current_level <= 2 {
		return;
	}
	let (height, width) = bounds.height_width();
	if height >= 5 || width >= 5 {
		let search_bounds = {
			let row_cut = if height > 2 { 1u64 } else { 0 };
			let col_cut = if width > 2 { 1u64 } else { 0 };
			bounds.inset(row_cut, col_cut)
		};
		for _ in 0..count {
			const MAX_ATTEMPTS: usize = 10;
			'attempts: for _ in 0..MAX_ATTEMPTS {
				let conceal_spot = search_bounds.to_random_level_spot();
				if map.feature_at_spot(conceal_spot) == Feature::Tunnel {
					map.put_feature_at_spot(conceal_spot, Feature::ConcealedTunnel);
					break 'attempts;
				}
			}
		}
	}
}

const ALL_MAZE_STEPS: &[MazeStep] = &[MazeStep::Up, MazeStep::Down, MazeStep::Left, MazeStep::Right];
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MazeStep { Up, Down, Left, Right }
impl MazeStep {
	pub fn find_new_tunnel_spot(&self, spot: LevelSpot, bounds: RoomBounds, map: &LevelMap) -> Option<LevelSpot> {
		if let Some(destination_spot) = self.find_destination_spot(spot, bounds) {
			if count_axial_tunnels(destination_spot, map) == 1 {
				Some(destination_spot)
			} else {
				None
			}
		} else {
			None
		}
	}
	fn find_destination_spot(&self, spot: LevelSpot, bounds: RoomBounds) -> Option<LevelSpot> {
		let row = spot.row.i64();
		let col = spot.col.i64();
		match self {
			MazeStep::Up => if row == bounds.top { None } else { Some(LevelSpot::from_i64(row - 1, col)) },
			MazeStep::Down => if row == bounds.bottom { None } else { Some(LevelSpot::from_i64(row + 1, col)) },
			MazeStep::Left => if col == bounds.left { None } else { Some(LevelSpot::from_i64(row, col - 1)) },
			MazeStep::Right => if col == bounds.right { None } else { Some(LevelSpot::from_i64(row, col + 1)) },
		}
	}
}

fn count_axial_tunnels(spot: LevelSpot, map: &LevelMap) -> usize {
	spot.with_axial_neighbors()
		.into_iter()
		.filter(|spot| map.feature_at_spot(*spot) == Feature::Tunnel)
		.count()
}
