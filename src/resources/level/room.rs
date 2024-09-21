use crate::prelude::HIDE_PERCENT;
use crate::random::{get_rand, rand_percent};
use crate::resources::level::map::feature::Feature;
use crate::resources::level::map::LevelMap;
use crate::resources::level::sector::{Sector, SectorBounds};
use crate::resources::level::size::LevelSpot;
use crate::room::{RoomBounds, RoomType};

#[derive(Debug, Copy, Clone, Default)]
pub struct LevelRoom {
	pub ty: RoomType,
	pub bounds: RoomBounds,
	pub exits: [RoomExit; 4],
}
impl LevelRoom {
	pub fn contains_spot(&self, spot: LevelSpot) -> bool {
		self.bounds.contains_spot(spot)
	}
}
impl LevelRoom {
	pub fn exit_at(&self, exit: ExitId) -> &RoomExit {
		&self.exits[exit as usize]
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
		self.exits[exit as usize] = RoomExit::Passage { to: sector, row, col };
		LevelSpot::from_i64(row, col)
	}
}

impl LevelRoom {
	pub fn from_sector(sector: Sector) -> Self {
		let bounds = get_random_room_bounds(&sector.bounds());
		Self { ty: RoomType::Nothing, bounds, exits: [RoomExit::None; 4] }
	}
	pub fn is_nothing(&self) -> bool { self.ty == RoomType::Nothing }
	pub fn is_vault(&self) -> bool { self.ty == RoomType::Room }
	pub fn is_maze(&self) -> bool { self.ty == RoomType::Maze }
	pub fn is_vault_or_maze(&self) -> bool { self.ty == RoomType::Room || self.ty == RoomType::Maze }
}

fn get_random_room_bounds(sector_bounds: &SectorBounds) -> RoomBounds {
	let height = get_rand(4, sector_bounds.height());
	let width = get_rand(7, sector_bounds.width() - 3);
	let row_offset = get_rand(0, sector_bounds.height() - height);
	let col_offset = get_rand(0, sector_bounds.width() - width);

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

#[derive(Debug, Copy, Clone, Default)]
pub enum RoomExit {
	#[default]
	None,
	Passage { to: Sector, row: i64, col: i64 },
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
}
