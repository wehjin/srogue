use crate::actions::instruct::instruction_lines;
use crate::init::Dungeon;
use crate::inventory::{get_obj_desc, inventory};
use crate::monster::Monster;
use crate::objects::{Object, ObjectId};
use crate::prelude::object_what::PackFilter;
use crate::resources::diary::Diary;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::resources::physics::rogue_sees_spot;
use crate::resources::rogue::Rogue;

use crate::prelude::DungeonSpot;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::course::dr_course;
use crate::settings::Settings;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct RunState {
	pub settings: Settings,
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
		Self { stats, level, visor: DungeonVisor::Map, diary: Diary::default(), settings: Settings::default() }
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
				let pack = self.as_rogue_pack();
				let rogue = &self.level.rogue;
				inventory(pack, PackFilter::AllObjects, self.settings.fruit.as_str(), &rogue.notes, rogue.wizard)
			}
		}
	}
	pub fn get_rogue_obj_desc(&self, obj_id: ObjectId) -> String {
		let obj = self.as_fighter().pack.object(obj_id).unwrap();
		let obj_ichar = obj.ichar;
		let obj_desc = get_obj_desc(obj, self);
		format!("{}({})", obj_desc, obj_ichar)
	}
}

impl Dungeon for RunState {
	fn move_mon_to(&mut self, mon_id: u64, row: i64, col: i64) {
		let to_spot = DungeonSpot { row, col };
		let from_spot = self.as_monster(mon_id).spot;
		{
			let monster = self.level.take_monster(LevelSpot::from(from_spot)).unwrap();
			self.level.put_monster(LevelSpot::from(to_spot), monster);
		}
		if self.is_any_door_at(to_spot.row, to_spot.col) {
			let entering = self.is_any_tunnel_at(from_spot.row, from_spot.col);
			dr_course(mon_id, entering, row, col, self);
		} else {
			let monster = self.as_monster_mut(mon_id);
			monster.spot = to_spot;
		}
	}

	fn set_interrupted(&mut self, value: bool) {
		self.diary.interrupted = value;
	}

	fn rogue_can_move(&self, row: i64, col: i64) -> bool {
		let from = self.level.rogue.spot.as_spot();
		self.level.features.can_move(*from, LevelSpot::from_i64(row, col))
	}

	fn has_monster_at(&self, row: i64, col: i64) -> bool {
		self.level.try_monster(LevelSpot::from_i64(row, col)).is_some()
	}

	fn get_monster_at(&self, row: i64, col: i64) -> Option<u64> {
		match self.level.try_monster(LevelSpot::from_i64(row, col)) {
			None => None,
			Some(monster) => Some(monster.id())
		}
	}

	fn get_monster(&self, mon_id: u64) -> Option<&Monster> {
		match self.level.find_monster(mon_id) {
			None => None,
			Some(spot) => self.level.try_monster(spot)
		}
	}

	fn interrupt_and_slurp(&mut self) {
		self.diary.interrupted = true;
		// TODO slurp or get rid of this function.
	}

	fn as_diary(&self) -> &Diary {
		&self.diary
	}

	fn as_diary_mut(&mut self) -> &mut Diary {
		&mut self.diary
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

	fn is_any_door_at(&self, row: i64, col: i64) -> bool {
		self.level.features.feature_at(LevelSpot::from_i64(row, col)).is_any_door()
	}

	fn is_any_tunnel_at(&self, row: i64, col: i64) -> bool {
		self.level.features.feature_at(LevelSpot::from_i64(row, col)).is_any_tunnel()
	}

	fn is_any_trap_at(&self, row: i64, col: i64) -> bool {
		self.level.features.feature_at(LevelSpot::from_i64(row, col)).is_any_trap()
	}

	fn is_no_feature_at(&self, row: i64, col: i64) -> bool {
		self.level.features.feature_at(LevelSpot::from_i64(row, col)).is_nothing()
	}

	fn is_passable_at(&self, row: i64, col: i64) -> bool {
		self.level.features.is_passable(LevelSpot::from_i64(row, col))
	}

	fn has_object_at(&self, row: i64, col: i64) -> bool {
		self.level.try_object(LevelSpot::from_i64(row, col)).is_some()
	}

	fn try_object_at(&self, row: i64, col: i64) -> Option<&Object> {
		self.level.try_object(LevelSpot::from_i64(row, col))
	}

	fn shows_skull(&self) -> bool {
		true
	}

	fn player_name(&self) -> String {
		whoami::username()
	}

	fn monster_ids(&self) -> Vec<u64> {
		self.level.monster_ids()
	}

	fn cleaned_up(&self) -> Option<String> {
		self.diary.cleaned_up.clone()
	}

	fn roll_wanderer_spot(&self, rng: &mut impl Rng) -> Option<LevelSpot> {
		for _ in 0..25 {
			let spot = self.level.roll_vacant_spot(true, false, true, rng);
			let rogue_can_see = rogue_sees_spot(spot, self, self, self);
			let out_of_sight = !rogue_can_see;
			if out_of_sight {
				return Some(spot.into());
			}
		}
		None
	}

	fn airdrop_monster_at(&mut self, row: i64, col: i64, monster: Monster) {
		let spot = LevelSpot::from_i64(row, col);
		self.level.put_monster(spot, monster);
		// TODO Call aim_monster(monster, &self.level);
	}
}