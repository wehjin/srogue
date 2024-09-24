use crate::resources::level::feature_grid::feature::Feature;
use crate::resources::level::feature_grid::FeatureGrid;
use crate::resources::level::plain::Axis;
use crate::resources::level::room::{ExitId, LevelRoom};
use crate::resources::level::sector::{shuffled_sector_neighbors, Sector, SectorNeighbor};
use crate::resources::level::size::LevelSpot;
use crate::room::RoomType;
use rand::Rng;

pub fn make_deadend(sector: Sector, do_recurse: bool, current_level: usize, spaces: &mut [LevelRoom; 9], features: &mut FeatureGrid, rng: &mut impl Rng) -> Vec<Sector> {
	let bounds = spaces[sector as usize].bounds;
	let random_spot = bounds.roll_spot(rng);
	let mut found = 0usize;
	for (i, target) in get_targets(sector, spaces, rng).iter().enumerate() {
		let spot = if !do_recurse || found > 0 || !features.feature_at(random_spot).is_any_tunnel() { bounds.to_center_level_spot() } else { random_spot };
		let target_spot = spaces[target.sector as usize].put_exit(target.exit, sector, current_level, features, rng);
		let axis = get_axis(target.exit);
		features.put_passage(axis, spot, target_spot, current_level, rng);
		spaces[sector as usize].ty = RoomType::DeadEnd;
		features.put_feature(spot, Feature::Tunnel);
		found += 1;
		if found == 1 {
			let more_targets_exist = (i + 1) < get_targets(sector, spaces, rng).len();
			if more_targets_exist && rng.gen_bool(0.5) {
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
	for neighbor in shuffled_sector_neighbors(rng) {
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

fn get_targets(sector: Sector, spaces: &[LevelRoom; 9], rng: &mut impl Rng) -> Vec<Target> {
	let targets = shuffled_sector_neighbors(rng)
		.into_iter()
		.filter_map(|neighbor| {
			sector.find_neighbor(neighbor)
				.map(|neighbor_sector| Target::try_new(neighbor_sector, neighbor, spaces))
				.flatten()
		})
		.collect();
	targets
}

#[derive(Debug)]
struct Target {
	sector: Sector,
	exit: ExitId,
}

impl Target {
	pub fn try_new(sector: Sector, neighbor: SectorNeighbor, spaces: &[LevelRoom; 9]) -> Option<Self> {
		let exit = get_neighbor_exit(neighbor);
		let space = &spaces[sector as usize];
		if space.is_vault_or_maze() && space.exit_at(exit).is_empty() {
			Some(Self { sector, exit })
		} else {
			None
		}
	}
}

fn get_neighbor_exit(neighbor: SectorNeighbor) -> ExitId {
	let exit = match neighbor {
		SectorNeighbor::Above => ExitId::Bottom,
		SectorNeighbor::Below => ExitId::Top,
		SectorNeighbor::Right => ExitId::Left,
		SectorNeighbor::Left => ExitId::Right,
	};
	exit
}
fn get_axis(exit: ExitId) -> Axis {
	let axis = match exit {
		ExitId::Top | ExitId::Bottom => Axis::Vertical,
		ExitId::Right | ExitId::Left => Axis::Horizontal,
	};
	axis
}
