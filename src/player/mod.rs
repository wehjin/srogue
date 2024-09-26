use serde::{Deserialize, Serialize};

use crate::armors::ArmorKind;
use crate::components::hunger::HungerLevel;
use crate::level::{DungeonCell, Level};
use crate::machdep::md_slurp;
use crate::objects::note_tables::NoteTables;
use crate::objects::{Object, ObjectId, ObjectPack};
use crate::player::effects::TimeEffect;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::{DungeonSpot, MAX_GOLD, MAX_HP, MAX_STRENGTH};
use crate::resources::rogue::fighter::Fighter;
use crate::ring::effects::RingEffects;
use crate::room::RoomType;
use crate::room::RoomType::Maze;
use crate::settings::Settings;
use crate::weapons::kind::WeaponKind;

pub(crate) mod rings;
pub(crate) mod objects;
pub mod effects;
pub mod constants;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RoomMark {
	None,
	Passage,
	Cavern(usize),
}

impl From<Option<usize>> for RoomMark {
	fn from(value: Option<usize>) -> Self {
		if let Some(rn) = value {
			RoomMark::Cavern(rn)
		} else {
			RoomMark::None
		}
	}
}

impl RoomMark {
	pub fn is_none(&self) -> bool { self == &Self::None }
	pub fn is_cavern(&self) -> bool {
		if let RoomMark::Cavern(_) = self { true } else { false }
	}
	pub fn rn(&self) -> Option<usize> {
		if let RoomMark::Cavern(rn) = self { Some(*rn) } else { None }
	}
	pub fn is_maze(&self, level: &Level) -> bool {
		self.is_type(&[Maze], level)
	}
	pub fn is_type<T: AsRef<[RoomType]>>(&self, room_type: T, level: &Level) -> bool {
		match self {
			RoomMark::None => false,
			RoomMark::Passage => false,
			RoomMark::Cavern(rn) => level.rooms[*rn].room_type.is_type(room_type),
		}
	}
	pub fn is_room(&self, room: RoomMark) -> bool {
		self == &room
	}
}

impl Player {
	pub fn in_room(&self, row: i64, col: i64, level: &Level) -> bool {
		self.cur_room == level.room(row, col)
	}
	pub(crate) fn is_near_spot(&self, spot: DungeonSpot) -> bool {
		self.to_spot().is_near(spot)
	}
	pub(crate) fn is_near(&self, row: i64, col: i64) -> bool {
		self.is_near_spot(DungeonSpot { row, col })
	}
	pub fn can_see_spot(&self, spot: &DungeonSpot, level: &Level) -> bool {
		self.can_see(spot.row, spot.col, level)
	}
	pub fn can_see(&self, row: i64, col: i64, level: &Level) -> bool {
		if self.health.blind.is_active() {
			false
		} else {
			if self.cur_room.is_room(level.room(row, col)) && !self.cur_room.is_maze(level) {
				true
			} else {
				self.is_near(row, col)
			}
		}
	}
	pub fn cur_cell<'a>(&self, level: &'a Level) -> &'a DungeonCell {
		&level.dungeon[self.rogue.row as usize][self.rogue.col as usize]
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
	pub reg_search_count: usize,
	pub hit_message: String,
	pub interrupted: bool,
	pub fight_monster: Option<u64>,
	pub hunger: HungerLevel,
	pub foods: usize,
	pub wizard: bool,
	pub cur_room: RoomMark,
	pub notes: NoteTables,
	pub settings: Settings,
	pub cleaned_up: Option<String>,
	pub cur_depth: isize,
	pub max_depth: isize,
	pub rogue: Fighter,
	pub party_counter: isize,
	pub ring_effects: RingEffects,
	pub health: RogueHealth,
	pub extra_hp: isize,
	pub less_hp: isize,
}

impl Avatar for Player {
	fn as_health(&self) -> &RogueHealth { &self.health }
	fn as_health_mut(&mut self) -> &mut RogueHealth { &mut self.health }
	fn rogue_row(&self) -> i64 { self.rogue.row }
	fn rogue_col(&self) -> i64 { self.rogue.col }
	fn set_rogue_row_col(&mut self, row: i64, col: i64) {
		self.rogue.row = row;
		self.rogue.col = col;
	}
}

pub trait Avatar {
	fn as_health(&self) -> &RogueHealth;
	fn as_health_mut(&mut self) -> &mut RogueHealth;
	fn rogue_row(&self) -> i64;
	fn rogue_col(&self) -> i64;
	fn set_rogue_row_col(&mut self, row: i64, col: i64);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Hash)]
pub struct RogueHealth {
	pub halluc: TimeEffect,
	pub blind: TimeEffect,
	pub levitate: TimeEffect,
	pub haste_self: TimeEffect,
	pub confused: TimeEffect,
	pub bear_trap: usize,
	pub being_held: bool,
}

impl RogueHealth {
	pub fn reset_for_new_level(&mut self) {
		self.bear_trap = 0;
		self.being_held = false;
	}
}

impl Player {
	pub fn is_blind(&self) -> bool { self.health.blind.is_active() }
	pub fn buffed_strength(&self) -> isize {
		self.ring_effects.apply_add_strength(self.cur_strength())
	}
	pub fn raise_hp_max(&mut self, raise: isize) {
		self.rogue.hp_max = (self.rogue.hp_max + raise).min(MAX_HP);
	}
	pub fn cur_strength(&self) -> isize { self.rogue.str_current }
	pub fn max_strength(&self) -> isize { self.rogue.str_max }
	pub fn raise_strength(&mut self) {
		self.rogue.str_current = (self.rogue.str_current + 1).min(MAX_STRENGTH);
		if self.rogue.str_current > self.rogue.str_max {
			self.rogue.str_max = self.rogue.str_current;
		}
	}

	pub fn exp(&self) -> isize { self.rogue.exp }
	pub fn buffed_exp(&self) -> isize {
		self.ring_effects.apply_dexterity(self.exp())
	}
	pub fn debuf_exp(&self) -> isize {
		self.hand_usage().count_hands()
	}
}

impl Player {
	pub fn interrupt_and_slurp(&mut self) {
		self.interrupted = true;
		md_slurp();
	}
	pub fn is_at_spot(&self, spot: DungeonSpot) -> bool {
		self.rogue.row == spot.row && self.rogue.col == spot.col
	}
	pub fn is_at(&self, row: i64, col: i64) -> bool {
		self.rogue.row == row && self.rogue.col == col
	}
	pub fn to_spot(&self) -> DungeonSpot {
		DungeonSpot { col: self.rogue.col, row: self.rogue.row }
	}
	pub fn wields_cursed_weapon(&self) -> bool {
		match self.weapon() {
			None => false,
			Some(obj) => obj.is_cursed()
		}
	}
	pub fn find_pack_obj(&self, f: impl Fn(&Object) -> bool) -> Option<&Object> {
		self.pack().find_object(f)
	}
	pub fn find_pack_obj_mut(&mut self, f: impl Fn(&Object) -> bool) -> Option<&mut Object> {
		self.pack_mut().find_object_mut(f)
	}
	pub fn pack_objects(&self) -> &Vec<Object> { self.rogue.pack.objects() }

	pub fn pack_mut(&mut self) -> &mut ObjectPack { &mut self.rogue.pack }

	pub fn combine_or_add_item_to_pack(&mut self, obj: Object) -> ObjectId {
		self.rogue.pack.combine_or_add_item(obj)
	}
}

pub const LAST_DUNGEON: isize = 99;

impl Player {
	pub fn armor_id(&self) -> Option<ObjectId> { self.rogue.armor }
	pub fn armor_kind(&self) -> Option<ArmorKind> {
		self.armor().map(|it| ArmorKind::from_index(it.which_kind as usize))
	}
	pub fn armor(&self) -> Option<&Object> {
		if let Some(id) = self.rogue.armor {
			self.pack().object_if_what(id, ObjectWhat::Armor)
		} else {
			None
		}
	}
	pub fn armor_mut(&mut self) -> Option<&mut Object> {
		if let Some(id) = self.rogue.armor {
			self.pack_mut().object_if_what_mut(id, ObjectWhat::Armor)
		} else {
			None
		}
	}

	pub fn unwear_armor(&mut self) -> Option<&Object> {
		let mut unworn_id = None;
		if let Some(armor) = self.armor_mut() {
			armor.in_use_flags &= !BEING_WORN;
			unworn_id = Some(armor.id());
		}
		self.rogue.armor = None;
		if let Some(obj_id) = unworn_id {
			self.object(obj_id)
		} else {
			None
		}
	}

	pub fn unwield_weapon(&mut self) {
		if let Some(obj) = self.weapon_mut() {
			obj.in_use_flags &= !BEING_WIELDED;
		}
		self.rogue.weapon = None;
	}
	pub fn weapon_id(&self) -> Option<ObjectId> { self.rogue.weapon }
	pub fn weapon_kind(&self) -> Option<WeaponKind> {
		self.weapon().map(|it| it.weapon_kind()).flatten()
	}
	pub fn weapon(&self) -> Option<&Object> {
		if let Some(id) = self.weapon_id() {
			self.pack().object_if_what(id, ObjectWhat::Weapon)
		} else {
			None
		}
	}
	pub fn weapon_mut(&mut self) -> Option<&mut Object> {
		if let Some(id) = self.weapon_id() {
			self.pack_mut().object_if_what_mut(id, ObjectWhat::Weapon)
		} else {
			None
		}
	}
	pub fn maintain_max_gold(&mut self) {
		if self.rogue.gold > MAX_GOLD {
			self.rogue.gold = MAX_GOLD;
		}
	}

	pub fn descend(&mut self) {
		self.cur_depth = (self.cur_depth + 1).min(LAST_DUNGEON);
		self.max_depth = self.max_depth.max(self.cur_depth);
	}
	pub fn ascend(&mut self) {
		self.cur_depth -= 2;
	}

	pub fn reset_for_new_level(&mut self) {
		self.health.reset_for_new_level();
		self.rogue.col = -1;
		self.rogue.row = -1;
	}
	pub fn gold(&self) -> usize { self.rogue.gold }
	pub fn new(settings: Settings) -> Self {
		Player {
			reg_search_count: 0,
			hit_message: "".to_string(),
			interrupted: false,
			fight_monster: None,
			hunger: HungerLevel::default(),
			foods: 0,
			wizard: false,
			cur_room: RoomMark::None,
			notes: NoteTables::new(),
			settings,
			cleaned_up: None,
			cur_depth: 0,
			max_depth: 1,
			rogue: Fighter::default(),
			party_counter: 0,
			ring_effects: Default::default(),
			health: RogueHealth::default(),
			extra_hp: 0,
			less_hp: 0,
		}
	}
}

