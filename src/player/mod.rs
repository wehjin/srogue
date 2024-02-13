use ncurses::chtype;
use serde::{Deserialize, Serialize};
use crate::monster::Fighter;
use crate::objects::{obj, object, ObjectId, ObjectPack};
use crate::pack::{check_duplicate, next_avail_ichar};
use crate::armors::ArmorKind;
use crate::player::effects::TimeEffect;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::{DungeonSpot, LAST_DUNGEON, MAX_ARMOR, MAX_GOLD};
use crate::prelude::object_what::ObjectWhat;
use crate::ring::effects::RingEffects;
use crate::settings::Settings;
use crate::weapons::WeaponKind;

pub(crate) mod rings;
pub(crate) mod objects;
pub mod effects;
pub mod constants;

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
	pub settings: Settings,
	pub cleaned_up: Option<String>,
	pub cur_depth: usize,
	pub max_depth: usize,
	pub rogue: Fighter,
	pub party_counter: usize,
	pub ring_effects: RingEffects,
	pub halluc: TimeEffect,
	pub blind: TimeEffect,
	pub levitate: TimeEffect,
	pub haste_self: TimeEffect,
	pub confused: TimeEffect,
	pub extra_hp: isize,
	pub less_hp: isize,
}

impl Player {
	pub fn cur_strength(&self) -> isize { self.rogue.str_current }
	pub fn buffed_strength(&self) -> isize {
		self.ring_effects.apply_add_strength(self.cur_strength())
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
	pub fn is_at(&self, row: i64, col: i64) -> bool {
		self.rogue.row == row && self.rogue.col == col
	}
	pub fn to_spot(&self) -> DungeonSpot {
		DungeonSpot { col: self.rogue.col, row: self.rogue.row }
	}
	pub fn to_curses_char(&self) -> chtype {
		chtype::from(self.rogue.fchar)
	}
	pub fn wields_cursed_weapon(&self) -> bool {
		match self.weapon() {
			None => false,
			Some(obj) => obj.is_cursed()
		}
	}
	pub fn find_pack_obj(&self, f: impl Fn(&obj) -> bool) -> Option<&obj> {
		self.pack().find_object(f)
	}
	pub fn find_pack_obj_mut(&mut self, f: impl Fn(&obj) -> bool) -> Option<&mut obj> {
		self.pack_mut().find_object_mut(f)
	}
	pub fn pack_objects(&self) -> &Vec<obj> { self.rogue.pack.objects() }

	pub fn pack_mut(&mut self) -> &mut ObjectPack { &mut self.rogue.pack }

	pub fn combine_or_add_item_to_pack(&mut self, mut obj: object) -> ObjectId {
		if let Some(id) = check_duplicate(&obj, &mut self.rogue.pack) {
			return id;
		}
		obj.ichar = next_avail_ichar(self);
		let obj_id = obj.id();
		self.rogue.pack.add(obj);
		return obj_id;
	}
}

impl Player {
	pub fn armor_id(&self) -> Option<ObjectId> { self.rogue.armor }
	pub fn armor_kind(&self) -> Option<ArmorKind> {
		self.armor().map(|it| ArmorKind::from_index(it.which_kind as usize))
	}
	pub fn armor(&self) -> Option<&obj> {
		if let Some(id) = self.rogue.armor {
			self.pack().object_if_what(id, ObjectWhat::Armor)
		} else {
			None
		}
	}
	pub fn armor_mut(&mut self) -> Option<&mut obj> {
		if let Some(id) = self.rogue.armor {
			self.pack_mut().object_if_what_mut(id, ObjectWhat::Armor)
		} else {
			None
		}
	}

	pub fn unwear_armor(&mut self) -> Option<&obj> {
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

	pub fn maintain_armor_max_enchant(&mut self) {
		if let Some(armor) = self.armor_mut() {
			if armor.d_enchant > MAX_ARMOR {
				armor.d_enchant = MAX_ARMOR
			}
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

	pub fn weapon(&self) -> Option<&obj> {
		if let Some(id) = self.weapon_id() {
			self.pack().object_if_what(id, ObjectWhat::Weapon)
		} else {
			None
		}
	}

	pub fn weapon_mut(&mut self) -> Option<&mut obj> {
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
		let cur = (self.cur_depth + 1).min(LAST_DUNGEON);
		let max = self.max_depth.max(cur);
		self.cur_depth = cur;
		self.max_depth = max;
	}
	pub fn ascend(&mut self) {
		let cur = if self.cur_depth < 3 { 1 } else { self.cur_depth - 2 };
		self.cur_depth = cur;
	}
	pub fn reset_spot(&mut self) {
		self.rogue.col = -1;
		self.rogue.row = -1;
	}
	pub fn gold(&self) -> usize { self.rogue.gold }
	pub fn new(settings: Settings) -> Self {
		const INIT_HP: isize = 12;
		Player {
			settings,
			cleaned_up: None,
			cur_depth: 0,
			max_depth: 1,
			rogue: Fighter {
				armor: None,
				weapon: None,
				left_ring: None,
				right_ring: None,
				hp_current: INIT_HP,
				hp_max: INIT_HP,
				str_current: 16,
				str_max: 16,
				pack: ObjectPack::new(),
				gold: 0,
				exp: 1,
				exp_points: 0,
				row: 0,
				col: 0,
				fchar: '@',
				moves_left: 1250,
			},
			party_counter: 0,
			ring_effects: Default::default(),
			halluc: Default::default(),
			blind: Default::default(),
			levitate: Default::default(),
			haste_self: Default::default(),
			confused: Default::default(),
			extra_hp: 0,
			less_hp: 0,
		}
	}
}

