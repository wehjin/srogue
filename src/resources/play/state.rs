use crate::actions::instruct::instruction_lines;
use crate::init::Dungeon;
use crate::inventory::inventory;
use crate::monster::Monster;
use crate::player::{Avatar, RogueHealth};
use crate::prelude::object_what::PackFilter;
use crate::resources::diary::Diary;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::resources::rogue::fighter::Fighter;
use crate::resources::rogue::spot::RogueSpot;
use crate::resources::rogue::Rogue;
use crate::ring::effects::RingEffects;
use rand::Rng;

pub struct RunState {
	pub stats: DungeonStats,
	pub level: DungeonLevel,
	pub visor: DungeonVisor,
	pub diary: Diary,
}
impl RunState {
	pub fn init(rng: &mut impl Rng) -> Self {
		let mut stats = DungeonStats::new(rng);
		let rogue = Rogue::new(1).outfit(rng);
		let party_type = PartyType::NoParty;
		let mut level = roll_level(party_type, rogue, &mut stats, rng);
		level.lighting_enabled = true;
		Self { stats, level, visor: DungeonVisor::Map, diary: Diary::default() }
	}
	pub fn to_lines(&self) -> Vec<String> {
		match self.visor {
			DungeonVisor::Map => {
				let mut lines = self.level.format(true);
				lines.insert(0, "".to_string());
				lines.push("".to_string());
				lines
			}
			DungeonVisor::Help => instruction_lines(),
			DungeonVisor::Inventory => {
				let pack = self.level.rogue.as_pack();
				let stats = &self.stats;
				inventory(pack, PackFilter::AllObjects, stats.fruit.as_str(), &stats.notes, stats.wizard)
			}
		}
	}
}

impl Avatar for RunState {
	fn as_health(&self) -> &RogueHealth {
		&self.level.rogue.health
	}

	fn as_health_mut(&mut self) -> &mut RogueHealth {
		&mut self.level.rogue.health
	}

	fn rogue_row(&self) -> i64 {
		self.level.rogue.spot.as_spot().row.i64()
	}

	fn rogue_col(&self) -> i64 {
		self.level.rogue.spot.as_spot().col.i64()
	}

	fn set_rogue_row_col(&mut self, row: i64, col: i64) {
		let spot = LevelSpot::from_i64(row, col);
		self.level.rogue.spot = RogueSpot::from_spot(spot, &self.level);
	}
}

impl Dungeon for RunState {
	fn rogue_can_move(&self, row: i64, col: i64) -> bool {
		let from = self.level.rogue.spot.as_spot();
		self.level.features.can_move(*from, LevelSpot::from_i64(row, col))
	}

	fn has_monster(&self, row: i64, col: i64) -> bool {
		self.level.try_monster(LevelSpot::from_i64(row, col)).is_some()
	}

	fn as_monster_mut(&mut self, mon_id: u64) -> &mut Monster {
		let spot = self.level.find_monster(mon_id).unwrap();
		self.level.as_monster_mut(spot)
	}

	fn interrupt_and_slurp(&mut self) {
		todo!()
	}

	fn as_diary_mut(&mut self) -> &mut Diary {
		&mut self.diary
	}

	fn as_fighter(&self) -> &Fighter {
		&self.level.rogue.fighter
	}

	fn is_max_depth(&self) -> bool {
		self.level.rogue.depth.is_max()
	}

	fn m_moves(&self) -> usize {
		self.stats.m_moves
	}

	fn m_moves_mut(&mut self) -> &mut usize {
		&mut self.stats.m_moves
	}

	fn as_ring_effects(&self) -> &RingEffects {
		&self.level.rogue.ring_effects
	}

	fn is_any_door_at(&self, row: i64, col: i64) -> bool {
		self.level.features.feature_at(LevelSpot::from_i64(row, col)).is_any_door()
	}

	fn is_any_tunnel_at(&self, row: i64, col: i64) -> bool {
		self.level.features.feature_at(LevelSpot::from_i64(row, col)).is_any_tunnel()
	}
}