use crate::objects::Object;
use crate::resources::level::feature_grid::FeatureGrid;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;

use crate::monster::Monster;
use crate::resources::level::feature_grid::feature::{Feature, FeatureFilter};
use crate::resources::level::room::LevelRoom;
use crate::resources::level::torch_grid::TorchGrid;
use crate::resources::rogue::spot::RogueSpot;
use crate::resources::rogue::Rogue;
use crate::room::{RoomBounds, RoomType};
use crate::trap::trap_kind::TrapKind;
use crate::trap::Trap;
use rand::Rng;
use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DungeonLevel {
	pub depth: usize,
	pub is_max: bool,
	pub ty: PartyType,
	pub rooms: BTreeMap<RoomId, LevelRoom>, // BTreeMap ensures consistent iteration order.
	pub features: FeatureGrid,
	pub torches: TorchGrid,
	pub party_room: Option<RoomId>,
	pub lighting_enabled: bool,
	pub objects: BTreeMap<LevelSpot, Object>,
	pub monsters: BTreeMap<LevelSpot, Monster>,
	pub rogue: Rogue,
}

impl DungeonLevel {
	pub fn rogue_at_spot(&self, spot: LevelSpot) -> bool {
		self.rogue.spot.is_spot(spot)
	}
	pub fn put_rogue(&mut self, spot: RogueSpot) {
		self.rogue.spot = spot;
	}
}

impl DungeonLevel {
	pub fn as_room(&self, room_id: RoomId) -> &LevelRoom {
		self.rooms.get(&room_id).unwrap()
	}
	pub fn as_room_mut(&mut self, room_id: RoomId) -> &mut LevelRoom {
		self.rooms.get_mut(&room_id).unwrap()
	}
}

impl DungeonLevel {
	pub fn room_at_spot(&self, spot: LevelSpot) -> Option<RoomId> {
		for (id, room) in &self.rooms {
			let within_room = room.contains_spot(spot);
			if within_room {
				return Some(*id);
			}
		}
		None
	}
	pub fn vault_and_maze_rooms(&self) -> Vec<RoomId> {
		let result = self.rooms.iter()
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
	pub fn is_lit_at(&self, spot: LevelSpot) -> bool {
		self.torches.lit_at(spot)
	}
	pub fn light_room(&mut self, room_id: RoomId) {
		let room = self.as_room_mut(room_id);
		for spot in room.bounds.to_spots() {
			self.torches.light(spot);
		}
	}
	pub fn light_tunnel_spot(&mut self, spot: LevelSpot) {
		self.torches.light(spot);
		for to in RoomBounds::from(spot).expand(1, 1).to_spots() {
			if self.features.can_move(spot, to) {
				self.torches.light(to);
			}
		}
	}
}

impl DungeonLevel {
	pub fn spot_is_vacant(&self, spot: LevelSpot, allow_objects: bool, allow_monsters: bool) -> bool {
		let is_floor_or_tunnel = self.spot_is_floor_or_tunnel(spot);
		let no_rogue = !self.rogue.spot.is_spot(spot);
		let no_object = allow_objects || self.try_object(spot).is_none();
		let no_monsters = allow_monsters || self.try_monster(spot).is_none();
		no_monsters && no_object && no_rogue && is_floor_or_tunnel
	}
	pub fn spot_is_tunnel(&self, spot: LevelSpot) -> bool {
		let feature = self.features.feature_at(spot);
		feature == Feature::Tunnel
	}
	pub fn spot_is_floor_or_tunnel(&self, spot: LevelSpot) -> bool {
		let feature = self.features.feature_at(spot);
		feature == Feature::Floor || feature == Feature::Tunnel
	}
	pub fn spot_in_vault_or_maze(&self, spot: LevelSpot) -> bool {
		match self.room_at_spot(spot) {
			Some(id) => {
				let ty = id.room_type();
				ty == RoomType::Room || ty == RoomType::Maze
			}
			None => false,
		}
	}
	pub fn roll_vacant_spot(&self, allow_objects: bool, allow_monsters: bool, allow_stairs: bool, rng: &mut impl Rng) -> LevelSpot {
		let feature_filter = if allow_stairs { FeatureFilter::FloorTunnelOrStair } else { FeatureFilter::FloorOrTunnel };
		loop {
			let spot = self.features.roll_spot(feature_filter, rng);
			let in_vault_or_maze = self.spot_in_vault_or_maze(spot);
			let is_vacant = self.spot_is_vacant(spot, allow_objects, allow_monsters);
			if is_vacant && in_vault_or_maze {
				return spot;
			}
		}
	}
}

impl DungeonLevel {
	pub fn find_monster(&self, mon_id: u64) -> Option<LevelSpot> {
		for (spot, monster) in &self.monsters {
			if monster.id == mon_id {
				return Some(*spot);
			}
		}
		None
	}
	pub fn try_monster(&self, spot: LevelSpot) -> Option<&Monster> {
		self.monsters.get(&spot)
	}
	pub fn as_monster_mut(&mut self, spot: LevelSpot) -> &mut Monster {
		self.monsters.get_mut(&spot).expect("invalid monster spot")
	}
	pub fn put_monster(&mut self, spot: LevelSpot, mut monster: Monster) {
		let (row, col) = spot.i64();
		monster.set_spot(row, col);
		self.monsters.insert(spot, monster);
	}
	pub fn monster_spots_in(&self, room: RoomId) -> Vec<LevelSpot> {
		let room = self.rooms.get(&room).expect("invalid {room}");
		self.monsters.iter().filter_map(|(spot, _)| {
			match room.contains_spot(*spot) {
				true => Some(*spot),
				false => None
			}
		}).collect()
	}
}

impl DungeonLevel {
	pub fn try_object(&self, spot: LevelSpot) -> Option<&Object> {
		self.objects.get(&spot)
	}
	pub fn put_object(&mut self, spot: LevelSpot, mut object: Object) {
		object.set_spot(spot);
		self.objects.insert(spot, object);
	}
}
impl DungeonLevel {
	pub fn trap_at(&self, spot: LevelSpot) -> Option<TrapKind> {
		self.features.trap_at(spot)
	}
	pub fn put_trap(&mut self, spot: LevelSpot, mut trap: Trap) {
		let (row, col) = spot.usize();
		trap.trap_row = row;
		trap.trap_col = col;
		self.features.add_trap(trap.trap_type, spot);
	}
}
impl DungeonLevel {
	pub fn put_stairs(&mut self, spot: LevelSpot) {
		self.features.put_feature(spot, Feature::Stairs);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PartyType {
	PartyBig,
	PartyRollBig,
	NoParty,
}

impl PartyType {
	pub fn is_party(&self) -> bool {
		match self {
			PartyType::PartyBig => true,
			PartyType::PartyRollBig => true,
			PartyType::NoParty => false,
		}
	}
}

pub mod room_id {
	use crate::resources::level::sector::Sector;
	use crate::room::RoomType;

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
		pub fn is_vault(&self) -> bool {
			self.room_type().is_vault()
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::objects::note_tables::NoteTables;
	use crate::prelude::AMULET_LEVEL;
	use crate::resources::dungeon::stats::{DungeonStats, DEFAULT_FRUIT};
	use crate::resources::level::setup::party::depth::PartyDepth;
	use crate::resources::level::setup::roll_level;
	use crate::resources::level::{DungeonLevel, PartyType};
	use crate::resources::rogue::Rogue;
	use rand::SeedableRng;
	use rand_chacha::ChaChaRng;
	use std::collections::HashSet;

	#[test]
	fn same_rng_builds_same_level() {
		fn build_level() -> DungeonLevel {
			let rng = &mut ChaChaRng::seed_from_u64(17);
			let stats = &mut DungeonStats {
				party_depth: PartyDepth::new(99),
				food_drops: 7,
				fruit: DEFAULT_FRUIT.to_string(),
				notes: NoteTables::new(),
				wizard: false,
				m_moves: 0,
			};
			roll_level(PartyType::NoParty, Rogue::new(16), stats, rng)
		}
		let mut set = HashSet::new();
		for _ in 0..10 {
			set.insert(build_level());
		}
		assert_eq!(1, set.len());
	}

	#[test]
	fn no_party_works() {
		let rng = &mut ChaChaRng::seed_from_u64(17);
		let stats = &mut DungeonStats {
			party_depth: PartyDepth::new(99),
			food_drops: 7,
			fruit: DEFAULT_FRUIT.to_string(),
			notes: NoteTables::new(),
			wizard: false,
			m_moves: 0,
		};
		let mut level = roll_level(PartyType::NoParty, Rogue::new(16), stats, rng);
		level.print(true);
		level.lighting_enabled = true;
		level.print(false);
	}
	#[test]
	fn party_big_works() {
		let rng = &mut ChaChaRng::seed_from_u64(17);
		let stats = &mut DungeonStats {
			party_depth: PartyDepth::roll(rng),
			food_drops: (AMULET_LEVEL / 2 - 1) as usize,
			fruit: DEFAULT_FRUIT.to_string(),
			notes: NoteTables::new(),
			wizard: false,
			m_moves: 0,
		};
		let mut level = roll_level(PartyType::PartyBig, Rogue::new(AMULET_LEVEL as usize), stats, rng);
		level.lighting_enabled = true;
		level.print(true);
	}
}

pub mod design;
pub mod deadend;
pub mod feature_grid;
pub mod grid;
pub mod maze;
pub mod plain;
pub mod print;
pub mod sector;
pub mod setup;
pub mod size;
pub mod torch_grid;
pub mod room;
pub mod wake;
pub mod rogue;

