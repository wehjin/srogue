use crate::prelude::HIDE_PERCENT;
use crate::resources::dice::roll_chance;
use crate::resources::level::feature_grid::feature::Feature;
use crate::resources::level::feature_grid::FeatureGrid;
use crate::resources::level::plain::Axis;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::{Sector, SectorBounds};
use crate::resources::level::size::LevelSpot;
use crate::room::{RoomBounds, RoomType};
use rand::Rng;
use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct LevelRoom {
	pub ty: RoomType,
	pub bounds: RoomBounds,
	pub exits: [RoomExit; 4],
}

impl Index<ExitSide> for LevelRoom {
	type Output = RoomExit;
	fn index(&self, index: ExitSide) -> &Self::Output {
		&self.exits[index as usize]
	}
}
impl IndexMut<ExitSide> for LevelRoom {
	fn index_mut(&mut self, index: ExitSide) -> &mut Self::Output {
		&mut self.exits[index as usize]
	}
}

impl LevelRoom {
	pub fn contains_spot(&self, spot: LevelSpot) -> bool {
		self.bounds.contains_spot(spot)
	}
}
impl LevelRoom {
	pub fn exit_at(&self, exit: ExitSide) -> &RoomExit {
		&self.exits[exit as usize]
	}
	pub fn put_exit(&mut self, near_side: ExitSide, far_sector: Sector, depth: usize, map: &mut FeatureGrid, rng: &mut impl Rng) -> LevelSpot {
		let near_spot: LevelSpot;
		let wall_width = if self.is_maze() { 0u64 } else { 1 };
		match near_side {
			ExitSide::Top | ExitSide::Bottom => {
				let near_row = if near_side == ExitSide::Top { self.bounds.top } else { self.bounds.bottom };
				let near_col;
				let col_bounds = self.bounds.inset(0, wall_width);
				'init_col: loop {
					let maybe_col = col_bounds.random_col(rng);
					let feature = map.feature_at(LevelSpot::from_i64(near_row, maybe_col));
					if feature == Feature::HorizWall || feature == Feature::Tunnel {
						near_col = maybe_col;
						break 'init_col;
					}
				}
				near_spot = LevelSpot::from_i64(near_row, near_col)
			}
			ExitSide::Left | ExitSide::Right => {
				let near_col = if near_side == ExitSide::Right { self.bounds.right } else { self.bounds.left };
				let near_row;
				let row_bounds = self.bounds.inset(wall_width, 0);
				'init_row: loop {
					let maybe_row = row_bounds.random_row(rng);
					let feature = map.feature_at(LevelSpot::from_i64(maybe_row, near_col));
					if feature == Feature::VertWall || feature == Feature::Tunnel {
						near_row = maybe_row;
						break 'init_row;
					}
				}
				near_spot = LevelSpot::from_i64(near_row, near_col)
			}
		}
		{
			let conceal = depth > 2 && roll_chance(HIDE_PERCENT, rng);
			let feature = if self.is_vault() {
				let wall_axis = near_side.to_axis().flip();
				if conceal { Feature::ConcealedDoor(wall_axis) } else { Feature::Door }
			} else {
				if conceal { Feature::ConcealedTunnel } else { Feature::Tunnel }
			};
			map.put_feature(near_spot, feature);
		}
		self.exits[near_side as usize] = RoomExit::Passage { near_spot, far_sector, far_spot: None };
		near_spot
	}
}

impl LevelRoom {
	pub fn from_sector(sector: Sector, rng: &mut impl Rng) -> Self {
		let bounds = roll_room_bounds(&sector.bounds(), rng);
		Self { ty: RoomType::Nothing, bounds, exits: [RoomExit::None; 4] }
	}
	pub fn is_nothing(&self) -> bool { self.ty == RoomType::Nothing }
	pub fn is_vault(&self) -> bool { self.ty == RoomType::Room }
	pub fn is_maze(&self) -> bool { self.ty == RoomType::Maze }
	pub fn is_vault_or_maze(&self) -> bool { self.ty == RoomType::Room || self.ty == RoomType::Maze }
}

fn roll_room_bounds(sector_bounds: &SectorBounds, rng: &mut impl Rng) -> RoomBounds {
	let height = rng.gen_range(4..=sector_bounds.height());
	let width = rng.gen_range(7..=sector_bounds.width() - 3);
	let row_offset = rng.gen_range(0..=sector_bounds.height() - height);
	let col_offset = rng.gen_range(0..=sector_bounds.width() - width);

	let top = sector_bounds.top + row_offset;
	let bottom = top + height - 1;
	let left = sector_bounds.left + col_offset;
	let right = left + width - 1;
	RoomBounds { top, right, bottom, left }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExitSide {
	Top = 0,
	Right = 1,
	Bottom = 2,
	Left = 3,
}
impl ExitSide {
	pub fn flip(&self) -> Self {
		match self {
			ExitSide::Top => ExitSide::Bottom,
			ExitSide::Right => ExitSide::Left,
			ExitSide::Left => ExitSide::Right,
			ExitSide::Bottom => ExitSide::Top,
		}
	}
	pub fn to_axis(&self) -> Axis {
		match self {
			ExitSide::Top | ExitSide::Bottom => Axis::Vertical,
			ExitSide::Right | ExitSide::Left => Axis::Horizontal,
		}
	}
}
pub const ALL_EXIT_SIDES: [ExitSide; 4] = [ExitSide::Top, ExitSide::Right, ExitSide::Bottom, ExitSide::Left];

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub enum RoomExit {
	#[default]
	None,
	Passage { near_spot: LevelSpot, far_sector: Sector, far_spot: Option<LevelSpot> },
}

impl RoomExit {
	pub fn is_empty(&self) -> bool {
		match self {
			RoomExit::None => true,
			RoomExit::Passage { .. } => false,
		}
	}
	pub fn is_passage(&self) -> bool {
		match self {
			RoomExit::Passage { .. } => true,
			RoomExit::None => false,
		}
	}
	pub fn leads_to_room(&self, room: RoomId) -> bool {
		let room_sector = room.as_sector();
		match self {
			RoomExit::None => false,
			RoomExit::Passage { far_sector: to_sector, .. } => room_sector == Some(to_sector),
		}
	}
	pub fn get_near_spot(&self) -> Option<&LevelSpot> {
		match self {
			RoomExit::None => None,
			RoomExit::Passage { near_spot, .. } => Some(near_spot),
		}
	}
	pub fn get_far_spot(&self) -> Option<LevelSpot> {
		match self {
			RoomExit::None => None,
			RoomExit::Passage { far_spot, .. } => *far_spot,
		}
	}
	pub fn set_far_spot(&mut self, spot: LevelSpot) {
		match self {
			RoomExit::None => {}
			RoomExit::Passage { far_spot, .. } => {
				far_spot.replace(spot);
			}
		}
	}
}
