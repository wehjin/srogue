use libc::c_ushort;
use serde::{Deserialize, Serialize};
use crate::level::constants::{DCOLS, DROWS};

pub const NO_ROOM: i64 = -1;
pub const PASSAGE: i64 = -3;
pub const BIG_ROOM: usize = 10;
pub const R_ROOM: c_ushort = 2;
pub const MIN_ROW: i64 = 1;
pub const MAX_TITLE_LENGTH: usize = 30;
pub const MORE: &'static str = "-more-";
pub const MAX_SYLLABLE: usize = 40;
pub const MAX_METAL: usize = 14;
pub const MAX_WAND_MATERIAL: usize = 30;
pub const MAX_GEM: usize = 14;
pub const COL1: i64 = 26;
pub const COL2: i64 = 52;
pub const ROW1: i64 = 7;
pub const ROW2: i64 = 15;
pub const HIDE_PERCENT: usize = 12;
pub const AMULET_LEVEL: usize = 26;
pub const MAX_EXP_LEVEL: usize = 21;
pub const MAX_EXP: isize = 10000000;
pub const MAX_GOLD: usize = 900000;
pub const MAX_ARMOR: isize = 99;
pub const MAX_HP: isize = 800;
pub const MAX_STRENGTH: isize = 99;
pub const LAST_DUNGEON: usize = 99;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DungeonSpot {
	pub col: i64,
	pub row: i64,
}

impl DungeonSpot {
	fn new_closest_value(value: i64, target: i64) -> i64 {
		if value > target {
			value - 1
		} else if value < target {
			value + 1
		} else {
			value
		}
	}
	pub fn next_closest_row(&self, target_row: i64) -> i64 {
		Self::new_closest_value(self.row, target_row)
	}
	pub fn next_closest_col(&self, target_col: i64) -> i64 {
		Self::new_closest_value(self.col, target_col)
	}
	pub fn is_at(&self, row: i64, col: i64) -> bool {
		self.row == row && self.col == col
	}
	pub fn is_out_of_bounds(&self) -> bool {
		self.row < MIN_ROW || self.row > (DROWS - 2) as i64 || self.col < 0 || self.col > (DCOLS - 1) as i64
	}
	pub fn set(&mut self, row: i64, col: i64) {
		self.row = row;
		self.col = col;
	}
}

pub mod item_usage {
	use crate::objects::obj;

	pub const NOT_USED: u16 = 0o0;
	pub const BEING_WIELDED: u16 = 0o1;
	pub const BEING_WORN: u16 = 0o2;
	pub const ON_LEFT_HAND: u16 = 0o4;
	pub const ON_RIGHT_HAND: u16 = 0o10;
	pub const ON_EITHER_HAND: u16 = ON_LEFT_HAND | ON_RIGHT_HAND;
	pub const BEING_USED: u16 = BEING_WIELDED | BEING_WORN | ON_EITHER_HAND;

	pub fn being_worn(obj: &obj) -> bool {
		(obj.in_use_flags & BEING_WORN) != 0
	}

	pub fn being_wielded(obj: &obj) -> bool {
		(obj.in_use_flags & BEING_WIELDED) != 0
	}

	pub fn on_either_hand(obj: &obj) -> bool {
		(obj.in_use_flags & ON_EITHER_HAND) != 0
	}

	pub fn on_left_hand(obj: &obj) -> bool {
		(obj.in_use_flags & ON_LEFT_HAND) != 0
	}
}

pub mod ending;

pub mod object_what {
	use serde::{Deserialize, Serialize};

	#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub enum ObjectWhat {
		Armor,
		Weapon,
		Scroll,
		Potion,
		Gold,
		Food,
		Wand,
		Ring,
		Amulet,
		None,
	}

	#[derive(Clone, Eq, PartialEq)]
	pub enum PackFilter {
		AllObjects,
		Armors,
		Weapons,
		Scrolls,
		Potions,
		Wands,
		Rings,
		Foods,
		Amulets,
		AnyFrom(Vec<ObjectWhat>),
	}

	impl PackFilter {
		pub fn includes(&self, what: ObjectWhat) -> bool {
			match self {
				PackFilter::AllObjects => true,
				PackFilter::Armors => what == ObjectWhat::Armor,
				PackFilter::Weapons => what == ObjectWhat::Weapon,
				PackFilter::Scrolls => what == ObjectWhat::Scroll,
				PackFilter::Potions => what == ObjectWhat::Potion,
				PackFilter::Wands => what == ObjectWhat::Wand,
				PackFilter::Rings => what == ObjectWhat::Ring,
				PackFilter::Foods => what == ObjectWhat::Food,
				PackFilter::Amulets => what == ObjectWhat::Amulet,
				PackFilter::AnyFrom(choices) => choices.iter().position(|choice| *choice == what).is_some(),
			}
		}
	}

	const ARMOR: u16 = 0o1;
	const WEAPON: u16 = 0o2;
	const SCROLL: u16 = 0o4;
	const POTION: u16 = 0o10;
	const GOLD: u16 = 0o20;
	const FOOD: u16 = 0o40;
	const WAND: u16 = 0o100;
	const RING: u16 = 0o200;
	const AMULET: u16 = 0o400;
	const ALL_OBJECTS: u16 = 0o777;
}

pub mod food_kind {
	pub const RATION: u16 = 0;
	pub const FRUIT: u16 = 1;
}

pub mod stat_const {
	pub const STAT_LEVEL: usize = 0o1;
	pub const STAT_GOLD: usize = 0o2;
	pub const STAT_HP: usize = 0o4;
	pub const STAT_STRENGTH: usize = 0o10;
	pub const STAT_ARMOR: usize = 0o20;
	pub const STAT_EXP: usize = 0o40;
	pub const STAT_HUNGER: usize = 0o100;
	pub const STAT_LABEL: usize = 0o200;
	pub const STAT_ALL: usize = 0o377;
}
