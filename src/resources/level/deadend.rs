use crate::resources::level::feature_grid::feature::Feature;
use crate::resources::level::feature_grid::FeatureGrid;
use crate::resources::level::plain::Axis;
use crate::resources::level::room::{ExitSide, LevelRoom};
use crate::resources::level::sector::{shuffled_neighbor_sides, NeighborSide, Sector};
use crate::resources::level::size::LevelSpot;
use crate::room::RoomType;
use rand::Rng;

pub fn make_deadend(sector: Sector, do_recurse: bool, current_level: usize, spaces: &mut [LevelRoom; 9], features: &mut FeatureGrid, rng: &mut impl Rng) -> Vec<Sector> {
	let bounds = spaces[sector as usize].bounds;
	let random_spot = bounds.roll_spot(rng);
	let mut found = 0usize;
	for (i, neighbor) in get_neighbors(sector, spaces, rng).iter().enumerate() {
		let spot = if !do_recurse || found > 0 || !features.feature_at(random_spot).is_any_tunnel() {
			bounds.to_center_level_spot()
		} else {
			random_spot
		};
		let neighbor_door_spot = spaces[neighbor.sector as usize].put_exit(neighbor.exit, sector, current_level, features, rng);
		let axis = get_axis(neighbor.exit);
		features.put_passage(axis, spot, neighbor_door_spot, current_level, rng);
		spaces[sector as usize].ty = RoomType::DeadEnd;
		features.put_feature(spot, Feature::Tunnel);
		found += 1;
		if found == 1 {
			let more_neighbors_exist = (i + 1) < get_neighbors(sector, spaces, rng).len();
			if more_neighbors_exist && rng.gen_bool(0.5) {
				// Try to connect to another room/maze.
				continue;
			} else {
				// Try to make and to connect to another deadend.
				return make_recursive(sector, spot, current_level, spaces, features, rng);
			}
		}
		break;
	}
	vec![]
}

fn make_recursive(sector: Sector, spot: LevelSpot, current_level: usize, spaces: &mut [LevelRoom; 9], features: &mut FeatureGrid, rng: &mut impl Rng) -> Vec<Sector> {
	spaces[sector as usize].ty = RoomType::DeadEnd;
	features.put_feature(spot, Feature::Tunnel);
	let mut recursive_sectors = Vec::new();
	for neighbor in shuffled_neighbor_sides(rng) {
		if let Some(neighbor_sector) = sector.find_neighbor(neighbor) {
			let neighbor_space = &spaces[neighbor_sector as usize];
			if !neighbor_space.is_nothing() {
				continue;
			}
			let neighbor_spot = neighbor_space.bounds.to_center_level_spot();
			let neighbor_exit = get_neighbor_exit(neighbor);
			let axis = get_axis(neighbor_exit);
			features.put_passage(axis, spot, neighbor_spot, current_level, rng);
			recursive_sectors.push(neighbor_sector);
			make_recursive(neighbor_sector, neighbor_spot, current_level, spaces, features, rng);
		}
	}
	recursive_sectors
}

fn get_neighbors(sector: Sector, spaces: &[LevelRoom; 9], rng: &mut impl Rng) -> Vec<Neighbor> {
	let targets = shuffled_neighbor_sides(rng)
		.into_iter()
		.filter_map(|neighbor_side| {
			sector
				.find_neighbor(neighbor_side)
				.map(|neighbor_sector| {
					Neighbor::try_new(neighbor_sector, neighbor_side, spaces)
				})
				.flatten()
		})
		.collect();
	targets
}

#[derive(Debug)]
struct Neighbor {
	sector: Sector,
	exit: ExitSide,
}

impl Neighbor {
	pub fn try_new(sector: Sector, neighbor: NeighborSide, spaces: &[LevelRoom; 9]) -> Option<Self> {
		let exit = get_neighbor_exit(neighbor);
		let space = &spaces[sector as usize];
		if space.is_vault_or_maze() && space.exit_at(exit).is_empty() {
			Some(Self { sector, exit })
		} else {
			None
		}
	}
}

fn get_neighbor_exit(neighbor: NeighborSide) -> ExitSide {
	let exit = match neighbor {
		NeighborSide::Above => ExitSide::Bottom,
		NeighborSide::Below => ExitSide::Top,
		NeighborSide::Right => ExitSide::Left,
		NeighborSide::Left => ExitSide::Right,
	};
	exit
}
fn get_axis(exit: ExitSide) -> Axis {
	let axis = match exit {
		ExitSide::Top | ExitSide::Bottom => Axis::Vertical,
		ExitSide::Right | ExitSide::Left => Axis::Horizontal,
	};
	axis
}
