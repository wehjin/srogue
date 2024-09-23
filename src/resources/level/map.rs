use crate::level::constants::{DCOLS, DROWS};
use crate::level::materials::Visibility;
use crate::monster::Monster;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::{HIDE_PERCENT, MIN_ROW};
use crate::random::{get_rand, rand_percent};
use crate::resources::level::map::feature::{Feature, FeatureFilter};
use crate::resources::level::maze::hide_random_tunnels;
use crate::resources::level::plain::Axis;
use crate::resources::level::size::LevelSpot;
use crate::room::RoomBounds;
use crate::trap::trap_kind::TrapKind;
use std::collections::HashMap;
use std::fmt::Debug;

pub mod feature {
	use crate::level::materials::Visibility;
	use crate::trap::trap_kind::TrapKind;

	#[derive(Debug, Copy, Clone, Eq, PartialEq)]
	pub enum Feature {
		None,
		HorizWall,
		VertWall,
		Floor,
		Tunnel,
		ConcealedTunnel,
		Door,
		ConcealedDoor,
		Stairs,
		Trap(TrapKind, Visibility),
	}
	impl Feature {
		pub fn is_any_tunnel(&self) -> bool {
			match self {
				Feature::Tunnel | Feature::ConcealedTunnel => true,
				_ => false,
			}
		}
	}

	#[derive(Copy, Clone)]
	pub enum FeatureFilter {
		FloorOrTunnel,
		FloorTunnelOrStair,
	}
	impl FeatureFilter {
		pub fn pass_feature(&self, feature: Feature) -> bool {
			match self {
				FeatureFilter::FloorOrTunnel => feature == Feature::Floor || feature.is_any_tunnel(),
				FeatureFilter::FloorTunnelOrStair => feature == Feature::Stairs || feature == Feature::Floor || feature.is_any_tunnel(),
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct LevelMap {
	pub min_spot: LevelSpot,
	pub max_spot: LevelSpot,
	pub rows: [[Feature; DCOLS]; DROWS],
	pub objects: HashMap<LevelSpot, ObjectWhat>,
	pub monsters: HashMap<LevelSpot, Monster>,
}

impl LevelMap {
	pub fn bounds(&self) -> RoomBounds {
		RoomBounds { top: 0, right: (DCOLS - 1) as i64, bottom: (DROWS - 1) as i64, left: 0 }
	}
}

impl LevelMap {
	pub fn monster_at(&self, spot: LevelSpot) -> Option<&Monster> {
		self.monsters.get(&spot)
	}
	pub fn add_monster(&mut self, monster: Monster, spot: LevelSpot) {
		self.monsters.insert(spot, monster);
	}
}

impl LevelMap {
	pub fn trap_at(&self, spot: LevelSpot) -> Option<TrapKind> {
		match self.feature_at_spot(spot) {
			Feature::Trap(kind, _) => Some(kind),
			_ => None
		}
	}
	pub fn add_trap(&mut self, trap: TrapKind, spot: LevelSpot) {
		self.put_feature_at_spot(spot, Feature::Trap(trap, Visibility::Hidden))
	}
}

impl LevelMap {
	pub fn object_at(&self, spot: LevelSpot) -> Option<&ObjectWhat> {
		self.objects.get(&spot)
	}
	pub fn add_object(&mut self, object: ObjectWhat, spot: LevelSpot) {
		self.objects.insert(spot, object);
	}
}

impl LevelMap {
	pub fn roll_spot(&self) -> LevelSpot {
		let row = get_rand(self.min_spot.row.i64(), self.max_spot.row.i64());
		let col = get_rand(self.min_spot.col.i64(), self.max_spot.col.i64());
		LevelSpot::from_i64(row, col)
	}
	pub fn roll_spot_with_feature_filter(&self, filter: FeatureFilter) -> LevelSpot {
		loop {
			let spot = self.roll_spot();
			let feature = self.feature_at_spot(spot);
			if filter.pass_feature(feature) {
				return spot;
			}
		}
	}
}

impl LevelMap {
	pub fn feature_at(&self, row: usize, col: usize) -> Feature {
		self.rows[row][col]
	}
	pub fn put_feature(&mut self, row: i64, col: i64, feature: Feature) {
		self.rows[row as usize][col as usize] = feature;
	}
}

impl LevelMap {
	pub fn feature_at_spot(&self, spot: LevelSpot) -> Feature {
		let row = spot.row.i64();
		let col = spot.col.i64();
		if row < 0 || row >= 24 || col < 0 || col >= 80 {
			Feature::None
		} else {
			self.rows[row as usize][col as usize]
		}
	}
	pub fn put_feature_at_spot(&mut self, spot: LevelSpot, feature: Feature) {
		self.rows[spot.row.usize()][spot.col.usize()] = feature;
	}
}

impl LevelMap {
	pub fn new() -> Self {
		Self {
			min_spot: LevelSpot::from_i64(MIN_ROW, 0),
			max_spot: LevelSpot::from_i64((DROWS - 2) as i64, (DCOLS - 1) as i64),
			rows: [[Feature::None; DCOLS]; DROWS],
			objects: Default::default(),
			monsters: Default::default(),
		}
	}
}

impl LevelMap {
	pub fn put_passage(&mut self, axis: Axis, spot1: LevelSpot, spot2: LevelSpot, current_level: usize) {
		let (start, end) = axis.sort_spots(spot1, spot2);
		let (start_row, start_col) = start.i64();
		let (end_row, end_col) = end.i64();
		match axis {
			Axis::Horizontal => {
				let middle_col = get_rand(start_col + 1, end_col - 1);
				for col in (start_col + 1)..middle_col {
					self.put_feature(start_row, col, Feature::Tunnel);
				}
				{
					let (row1, row2) = if start_row <= end_row { (start_row, end_row) } else { (end_row, start_row) };
					for row in row1..=row2 {
						self.put_feature(row, middle_col, Feature::Tunnel);
					}
				}
				for col in (middle_col + 1)..end_col {
					self.put_feature(end_row, col, Feature::Tunnel);
				}
			}
			Axis::Vertical => {
				let middle_row = get_rand(start_row + 1, end_row - 1);
				for row in (start_row + 1)..middle_row {
					self.put_feature(row, start_col, Feature::Tunnel);
				}
				{
					let (col1, col2) = if start_col <= end_col { (start_col, end_col) } else { (end_col, start_col) };
					for col in col1..=col2 {
						self.put_feature(middle_row, col, Feature::Tunnel);
					}
				}
				for row in (middle_row + 1)..end_row {
					self.put_feature(row, end_col, Feature::Tunnel);
				}
			}
		}
		if rand_percent(HIDE_PERCENT) {
			let top = start_row.min(end_row);
			let bottom = start_row.max(end_row);
			let left = start_col.min(end_col);
			let right = start_col.max(end_col);
			let bounds = RoomBounds { top, right, bottom, left };
			hide_random_tunnels(bounds, 1, current_level, self)
		}
	}
}

impl LevelMap {
	pub fn put_walls_and_floor(mut self, room: RoomBounds) -> Self {
		for row in room.top..=room.bottom {
			for col in room.left..=room.right {
				if row == room.top || row == room.bottom {
					self.put_sprite(row, col, Feature::HorizWall);
				} else if col == room.left || col == room.right {
					self.put_sprite(row, col, Feature::VertWall);
				} else {
					self.put_sprite(row, col, Feature::Floor);
				}
			}
		}
		self
	}
	fn put_sprite(&mut self, row: i64, col: i64, sprite: Feature) {
		self.rows[row as usize][col as usize] = sprite;
	}
}

