use crate::objects::Object;
use crate::random::{get_rand, rand_percent};
use crate::resources::level::map::LevelMap;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::resources::level::size::LevelSpot;

use crate::prelude::object_what::ObjectWhat;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::map::feature::Feature;
use crate::resources::level::room::LevelRoom;
use crate::resources::level::setup::{roll_objects, roll_stairs};
use crate::resources::party::PartyDepth;
use crate::resources::rogue::depth::RogueDepth;
use crate::room::{RoomBounds, RoomType};
use design::roll_design;
use std::collections::HashMap;

pub struct DungeonLevel {
	pub depth: usize,
	pub is_max: bool,
	pub ty: LevelType,
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub map: LevelMap,
	pub rogue_spot: LevelSpot,
}

impl DungeonLevel {
	pub fn as_room(&self, room_id: RoomId) -> Option<&LevelRoom> {
		self.rooms.get(&room_id)
	}
	pub fn vaults_and_mazes(&self) -> Vec<RoomId> {
		let result = self.rooms
			.iter()
			.filter_map(|(id, room)| {
				if room.is_vault_or_maze() {
					Some(*id)
				} else {
					None
				}
			})
			.collect();
		result
	}
}

impl DungeonLevel {
	pub fn spot_is_vacant(&self, spot: LevelSpot) -> bool {
		let is_floor_or_tunnel = self.spot_is_floor_or_tunnel(spot);
		let no_object = self.object_at(spot).is_none();
		let no_rogue = self.rogue_spot != spot;
		no_rogue && no_object && is_floor_or_tunnel
	}
	pub fn spot_is_floor_or_tunnel(&self, spot: LevelSpot) -> bool {
		let feature = self.map.feature_at_spot(spot);
		feature == Feature::Floor || feature == Feature::Tunnel
	}
	pub fn spot_in_vault_or_maze(&self, spot: LevelSpot) -> bool {
		for (id, room) in &self.rooms {
			let ty = id.room_type();
			let is_room_or_maze_space = ty == RoomType::Room || ty == RoomType::Maze;
			let is_within_room = room.contains_spot(spot);
			if is_room_or_maze_space && is_within_room {
				return true;
			}
		}
		false
	}
	pub fn roll_required_vacant_spot(&self) -> LevelSpot {
		loop {
			let spot = self.map.roll_floor_or_tunnel_spot();
			let in_vault_or_maze = self.spot_in_vault_or_maze(spot);
			let is_vacant = self.spot_is_vacant(spot);
			if is_vacant && in_vault_or_maze {
				return spot;
			}
		}
	}
}
impl DungeonLevel {
	pub fn object_at(&self, spot: LevelSpot) -> Option<&ObjectWhat> {
		self.map.object_at(spot)
	}
	pub fn put_object(&mut self, spot: LevelSpot, mut object: Object) {
		object.set_spot(spot);
		self.map.add_object(object.what_is, spot);
	}
	pub fn put_stairs(&mut self, spot: LevelSpot) {
		self.map.put_feature_at_spot(spot, Feature::Stairs);
	}
}
impl DungeonLevel {
	pub fn new(depth: usize, is_max: bool, party_type: LevelType) -> Self {
		Self {
			depth,
			is_max,
			ty: party_type,
			rooms: HashMap::new(),
			map: LevelMap::new(),
			rogue_spot: LevelSpot::from_i64(0, 0),
		}
	}
}

pub fn roll_filled_level(depth: usize, is_max: bool, party_type: LevelType, stats: &mut DungeonStats) -> DungeonLevel {
	let mut level = roll_level_with_rooms(depth, is_max, party_type);
	roll_objects(&mut level, stats);
	roll_stairs(&mut level);
	level
}
fn roll_level_with_rooms(depth: usize, is_max: bool, party_type: LevelType) -> DungeonLevel {
	if roll_big_room(party_type) {
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
			ty: party_type,
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
		DungeonLevel { depth, is_max, ty: party_type, rooms, map, rogue_spot: LevelSpot::from_i64(0, 0) }
	}
}

fn roll_big_room(level_type: LevelType) -> bool {
	match level_type {
		LevelType::PartyBig => true,
		LevelType::PartyRollBig => rand_percent(1),
		LevelType::Plain => false
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum LevelType {
	PartyBig,
	PartyRollBig,
	Plain,
}

impl LevelType {
	pub fn from_depths(rogue: &RogueDepth, party: &PartyDepth) -> Self {
		if rogue.usize() == party.usize() {
			Self::PartyRollBig
		} else {
			Self::Plain
		}
	}
	pub fn is_party(&self) -> bool {
		match self {
			LevelType::PartyBig => true,
			LevelType::PartyRollBig => true,
			LevelType::Plain => false,
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
	use crate::resources::level::roll_filled_level;
	use crate::resources::level::LevelType;

	#[test]
	fn plain_level_works() {
		let mut stats = DungeonStats { food_drops: 7 };
		let level = roll_filled_level(16, true, LevelType::Plain, &mut stats);
		level.map.print();
	}
	#[test]
	fn party_level_works() {
		let mut stats = DungeonStats { food_drops: 7 };
		let level = roll_filled_level(16, true, LevelType::PartyBig, &mut stats);
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
