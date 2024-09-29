use crate::level::materials::Visibility;
use crate::prelude::HIDE_PERCENT;
use crate::resources::dice::roll_chance;
use crate::resources::level::feature_grid::feature::{Feature, FeatureFilter};
use crate::resources::level::grid::LevelGrid;
use crate::resources::level::maze::hide_random_tunnels;
use crate::resources::level::plain::Axis;
use crate::resources::level::size::LevelSpot;
use crate::room::RoomBounds;
use crate::trap::trap_kind::TrapKind;
use rand::Rng;
use std::fmt::Debug;
use std::ops::RangeInclusive;

pub mod feature {
	use crate::level::materials::Visibility;
	use crate::resources::level::plain::Axis;
	use crate::trap::trap_kind::TrapKind;

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
	pub enum Feature {
		#[default]
		None,
		HorizWall,
		VertWall,
		Floor,
		Tunnel,
		ConcealedTunnel,
		Door,
		ConcealedDoor(Axis),
		Stairs,
		Trap(TrapKind, Visibility),
	}
	impl Feature {
		pub fn is_any_trap(&self) -> bool {
			match self {
				Feature::Trap(..) => true,
				_ => false,
			}
		}
		pub fn is_any_tunnel(&self) -> bool {
			match self {
				Feature::Tunnel | Feature::ConcealedTunnel => true,
				_ => false,
			}
		}
		pub fn is_any_door(&self) -> bool {
			match self {
				Feature::Door | Feature::ConcealedDoor(_) => true,
				_ => false
			}
		}
		pub fn is_stairs(&self) -> bool {
			match self {
				Feature::Stairs => true,
				_ => false
			}
		}
		pub fn is_nothing(&self) -> bool {
			match self {
				Feature::None => true,
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FeatureGrid(LevelGrid<Feature>);

impl FeatureGrid {
	pub fn new() -> Self {
		Self(LevelGrid::<Feature>::new())
	}
	pub fn bounds(&self) -> &RoomBounds {
		self.0.bounds()
	}
	pub fn feature_at(&self, spot: LevelSpot) -> Feature {
		self.0.value_at(spot)
	}
	pub fn put_feature(&mut self, spot: LevelSpot, feature: Feature) {
		self.0.put_value(spot, feature);
	}
}

impl FeatureGrid {
	pub fn is_passable(&self, spot: LevelSpot) -> bool {
		let feature = self.feature_at(spot);
		match feature {
			Feature::None => false,
			Feature::HorizWall => false,
			Feature::VertWall => false,
			Feature::Floor => true,
			Feature::Tunnel => true,
			Feature::ConcealedTunnel => false,
			Feature::Door => true,
			Feature::ConcealedDoor(_) => false,
			Feature::Stairs => true,
			Feature::Trap(_, _) => true,
		}
	}
	pub fn can_move(&self, from: LevelSpot, to: LevelSpot) -> bool {
		if !self.is_passable(to) {
			return false;
		}
		if to != from {
			if self.feature_at(from).is_any_door() {
				if !to.has_same_row_or_col(from) {
					return false;
				}
			}
			if self.feature_at(to).is_any_door() {
				if !from.has_same_row_or_col(to) {
					return false;
				}
			}
		}
		true
	}

	pub fn roll_spot(&self, filter: FeatureFilter, rng: &mut impl Rng) -> LevelSpot {
		loop {
			let spot = self.0.bounds().roll_spot(rng);
			let feature = self.feature_at(spot);
			if filter.pass_feature(feature) {
				return spot;
			}
		}
	}
	pub fn put_passage(&mut self, axis: Axis, spot1: LevelSpot, spot2: LevelSpot, current_level: usize, rng: &mut impl Rng) {
		let (start, end) = axis.sort_spots(spot1, spot2);
		struct Leg {
			rows: RangeInclusive<i64>,
			cols: RangeInclusive<i64>,
		}
		let legs = match axis {
			Axis::Horizontal => {
				let start_col = start.col.i64() + 1;
				let end_col = end.col.i64() - 1;
				let middle_col = rng.gen_range(start_col..=end_col);
				let start_row = start.row.i64();
				let end_row = end.row.i64();
				let (row1, row2) = if start_row <= end_row { (start_row, end_row) } else { (end_row, start_row) };
				[
					Leg { rows: start_row..=start_row, cols: start_col..=middle_col },
					Leg { rows: end_row..=end_row, cols: middle_col..=end_col },
					Leg { rows: row1..=row2, cols: middle_col..=middle_col }
				]
			}
			Axis::Vertical => {
				let start_row = start.row.i64() + 1;
				let end_row = end.row.i64() - 1;
				let middle_row = rng.gen_range(start_row..=end_row);
				let start_col = start.col.i64();
				let end_col = end.col.i64();
				let (col1, col2) = if start_col <= end_col { (start_col, end_col) } else { (end_col, start_col) };
				[
					Leg { rows: start_row..=middle_row, cols: start_col..=start_col },
					Leg { rows: middle_row..=end_row, cols: end_col..=end_col },
					Leg { rows: middle_row..=middle_row, cols: col1..=col2 },
				]
			}
		};
		for leg in legs {
			for row in leg.rows {
				for col in leg.cols.clone() {
					let spot = LevelSpot::from_i64(row, col);
					self.put_feature(spot, Feature::Tunnel)
				}
			}
		}

		let hide = roll_chance(HIDE_PERCENT, rng);
		if hide {
			let (start_row, start_col) = start.i64();
			let (end_row, end_col) = end.i64();
			let top = start_row.min(end_row);
			let bottom = start_row.max(end_row);
			let left = start_col.min(end_col);
			let right = start_col.max(end_col);
			let bounds = RoomBounds { top, right, bottom, left };
			hide_random_tunnels(bounds, 1, current_level, self, rng)
		}
	}
	pub fn put_walls_and_floor(mut self, room: RoomBounds) -> Self {
		for row in room.top..=room.bottom {
			for col in room.left..=room.right {
				let feature = match (row, col) {
					(row, _) if row == room.top || row == room.bottom => Feature::HorizWall,
					(_, col) if col == room.left || col == room.right => Feature::VertWall,
					_ => Feature::Floor
				};
				self.put_feature(LevelSpot::from_i64(row, col), feature)
			}
		}
		self
	}
}

impl FeatureGrid {
	pub fn trap_at(&self, spot: LevelSpot) -> Option<TrapKind> {
		match self.feature_at(spot) {
			Feature::Trap(kind, _) => Some(kind),
			_ => None
		}
	}
	pub fn add_trap(&mut self, trap: TrapKind, spot: LevelSpot) {
		self.put_feature(spot, Feature::Trap(trap, Visibility::Hidden))
	}
}

