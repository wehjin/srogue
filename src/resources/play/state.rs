use crate::actions::instruct::instruction_lines;
use crate::inventory::inventory;
use crate::prelude::object_what::PackFilter;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::resources::rogue::Rogue;
use rand::Rng;

pub struct RunState {
	pub stats: DungeonStats,
	pub level: DungeonLevel,
	pub visor: DungeonVisor,
}
impl RunState {
	pub fn init(rng: &mut impl Rng) -> Self {
		let mut stats = DungeonStats::new(rng);
		let rogue = Rogue::new(1).outfit(rng);
		let party_type = PartyType::NoParty;
		let mut level = roll_level(party_type, rogue, &mut stats, rng);
		level.lighting_enabled = true;
		Self { stats, level, visor: DungeonVisor::Map }
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