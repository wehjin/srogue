use crate::objects::Object;
use crate::random::{get_rand, rand_percent};
use crate::resources::level::map::LevelMap;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::resources::level::size::LevelSpot;

use crate::resources::level::room::LevelRoom;
use crate::resources::party::PartyDepth;
use crate::resources::rogue::depth::RogueDepth;
use crate::room::{RoomBounds, RoomType};
use design::roll_design;
use std::collections::HashMap;

pub struct DungeonLevel {
	pub depth: usize,
	pub is_max: bool,
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub map: LevelMap,
	pub rogue_spot: LevelSpot,
}
impl DungeonLevel {
	pub fn new(depth: usize, is_max: bool) -> Self {
		Self {
			depth,
			is_max,
			rooms: HashMap::new(),
			map: LevelMap::new(),
			rogue_spot: LevelSpot::from_i64(0, 0),
		}
	}
	pub fn put_object(&mut self, spot: LevelSpot, object: Object) {
		self.map.add_object(object.what_is, spot);
	}
}
impl DungeonLevel {
	pub fn roll_drop_spot(&self) -> LevelSpot {
		loop {
			let spot = self.map.roll_floor_or_tunnel_spot();
			if self.room_or_maze_at_spot(spot).is_some() && spot != self.rogue_spot {
				return spot;
			}
		}
	}
	fn room_or_maze_at_spot(&self, spot: LevelSpot) -> Option<&LevelRoom> {
		for (id, room) in &self.rooms {
			let ty = id.room_type();
			let is_room_or_maze_space = ty == RoomType::Room || ty == RoomType::Maze;
			let is_within_room = room.contains_spot(spot);
			if is_room_or_maze_space && is_within_room {
				return Some(room);
			}
		}
		None
	}
}

pub fn roll_level(depth: usize, is_max: bool, room_sizing: RoomSizing) -> DungeonLevel {
	if roll_big_room(room_sizing) {
		let bounds = RoomBounds {
			top: get_rand(ROW0, ROW0 + 1),
			bottom: get_rand(ROW3 - 6, ROW3 - 1),
			left: get_rand(COL0, 10),
			right: get_rand(COL3 - 11, COL3 - 1),
		};
		let room = LevelRoom { ty: RoomType::Room, bounds, ..LevelRoom::default() };
		DungeonLevel {
			depth,
			is_max,
			rooms: vec![(RoomId::Big, room)].into_iter().collect(),
			map: LevelMap::new().put_walls_and_floor(bounds),
			rogue_spot: LevelSpot::from_i64(0, 0),
		}
	} else {
		let design = roll_design();
		let level = PlainLevel::new(depth)
			.add_rooms(design)
			.add_mazes()
			.connect_spaces()
			.add_deadends()
			;
		let rooms = ALL_SECTORS
			.into_iter()
			.map(|sector| {
				let space = level.space(sector);
				let room_id = RoomId::Little(sector, space.ty);
				(room_id, *space)
			})
			.collect();
		let map = level.into_map();
		DungeonLevel { depth, is_max, rooms, map, rogue_spot: LevelSpot::from_i64(0, 0) }
	}
}

fn roll_big_room(sizing: RoomSizing) -> bool {
	let big_room = match sizing {
		RoomSizing::BigAlways => true,
		RoomSizing::BigRoll if rand_percent(1) => true,
		_ => false,
	};
	big_room
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RoomSizing {
	BigAlways,
	BigRoll,
	SmallAlways,
}

impl RoomSizing {
	pub fn from_depths(rogue: &RogueDepth, party: &PartyDepth) -> Self {
		if rogue.usize() == party.usize() {
			Self::BigRoll
		} else {
			Self::SmallAlways
		}
	}
}

pub mod room_id {
	use crate::resources::level::sector::Sector;
	use crate::room::RoomType;

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
	pub enum RoomId {
		Big,
		Little(Sector, RoomType),
	}

	impl RoomId {
		pub fn room_type(&self) -> RoomType {
			match self {
				RoomId::Big => RoomType::Room,
				RoomId::Little(_, ty) => *ty,
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::resources::dungeon::stats::DungeonStats;
	use crate::resources::level::roll_level;
	use crate::resources::level::setup::roll_objects;
	use crate::resources::level::RoomSizing;

	#[test]
	fn make_level_works() {
		let mut stats = DungeonStats { food_drops: 7 };
		let mut level = roll_level(16, true, RoomSizing::BigRoll);
		roll_objects(&mut level, &mut stats);
		level.map.print();
	}
}

pub mod design;
pub mod deadend;
pub mod map;
pub mod maze;
pub mod plain;
pub mod sector;
pub mod setup;
pub mod size;
pub mod room;
