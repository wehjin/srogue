use crate::objects::Object;
use crate::resources::level::map::LevelMap;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;

use crate::monster::Monster;
use crate::prelude::object_what::ObjectWhat;
use crate::resources::level::map::feature::{Feature, FeatureFilter};
use crate::resources::level::room::LevelRoom;
use crate::resources::party::PartyDepth;
use crate::resources::rogue::depth::RogueDepth;
use crate::room::RoomType;
use crate::trap::trap_kind::TrapKind;
use crate::trap::Trap;
use std::collections::HashMap;

pub struct DungeonLevel {
	pub depth: usize,
	pub is_max: bool,
	pub ty: LevelType,
	pub rooms: HashMap<RoomId, LevelRoom>,
	pub map: LevelMap,
	pub rogue_spot: LevelSpot,
	pub party_room: Option<RoomId>,
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
	pub fn spot_is_vacant(&self, spot: LevelSpot, allow_objects: bool, allow_monsters: bool) -> bool {
		let is_floor_or_tunnel = self.spot_is_floor_or_tunnel(spot);
		let no_rogue = self.rogue_spot != spot;
		let no_object = allow_objects || self.object_at(spot).is_none();
		let no_monsters = allow_monsters || self.monster_at(spot).is_none();
		no_monsters && no_object && no_rogue && is_floor_or_tunnel
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
	pub fn roll_vacant_spot(&self, allow_objects: bool, allow_monsters: bool, allow_stairs: bool) -> LevelSpot {
		let feature_filter = if allow_stairs { FeatureFilter::FloorTunnelOrStair } else { FeatureFilter::FloorOrTunnel };
		loop {
			let spot = self.map.roll_spot_with_feature_filter(feature_filter);
			let in_vault_or_maze = self.spot_in_vault_or_maze(spot);
			let is_vacant = self.spot_is_vacant(spot, allow_objects, allow_monsters);
			if is_vacant && in_vault_or_maze {
				return spot;
			}
		}
	}
}
impl DungeonLevel {
	pub fn monster_at(&self, spot: LevelSpot) -> Option<&Monster> {
		self.map.monster_at(spot)
	}
	pub fn put_monster(&mut self, spot: LevelSpot, mut monster: Monster) {
		let (row, col) = spot.i64();
		monster.set_spot(row, col);
		self.map.add_monster(monster, spot);
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
}
impl DungeonLevel {
	pub fn trap_at(&self, spot: LevelSpot) -> Option<TrapKind> {
		self.map.trap_at(spot)
	}
	pub fn put_trap(&mut self, spot: LevelSpot, mut trap: Trap) {
		let (row, col) = spot.usize();
		trap.trap_row = row;
		trap.trap_col = col;
		self.map.add_trap(trap.trap_type, spot);
	}
}
impl DungeonLevel {
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
			party_room: None,
		}
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
	use crate::resources::level::setup::roll_complete_level;
	use crate::resources::level::LevelType;

	#[test]
	fn plain_level_works() {
		let mut stats = DungeonStats { food_drops: 7 };
		let level = roll_complete_level(16, true, LevelType::Plain, &mut stats);
		level.map.print();
	}
	#[test]
	fn party_level_works() {
		let mut stats = DungeonStats { food_drops: 3 };
		let level = roll_complete_level(8, true, LevelType::PartyBig, &mut stats);
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
