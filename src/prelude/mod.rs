use serde::{Deserialize, Serialize};

use crate::hit::get_dir_rc;
use crate::level::constants::{DCOLS, DROWS};
use crate::room::RoomBounds;
use crate::throw::Motion;

pub const NO_ROOM: i64 = -1;
pub const PASSAGE: i64 = -3;
pub const BIG_ROOM: usize = 10;
pub const MIN_ROW: i64 = 1;
pub const MAX_TITLE_LENGTH: usize = 30;
pub const COL1: i64 = 26;
pub const COL2: i64 = 52;
pub const ROW1: i64 = 7;
pub const ROW2: i64 = 15;
pub const HIDE_PERCENT: usize = 12;
pub const AMULET_LEVEL: isize = 26;
pub const MAX_EXP_LEVEL: usize = 21;
pub const MAX_EXP: isize = 10000000;
pub const MAX_GOLD: usize = 900000;
pub const MAX_ARMOR: isize = 99;
pub const MAX_HP: isize = 800;
pub const MAX_STRENGTH: isize = 99;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default, Hash)]
pub struct DungeonSpot {
	pub col: i64,
	pub row: i64,
}

impl From<(usize, usize)> for DungeonSpot {
	fn from((row, col): (usize, usize)) -> Self {
		DungeonSpot::new(row, col)
	}
}

impl From<(i64, i64)> for DungeonSpot {
	fn from((row, col): (i64, i64)) -> Self {
		Self { row, col }
	}
}

impl From<(i32, i32)> for DungeonSpot {
	fn from((row, col): (i32, i32)) -> Self {
		Self { row: row as i64, col: col as i64 }
	}
}

impl DungeonSpot {
	pub fn new(row: usize, col: usize) -> Self {
		Self { col: col as i64, row: row as i64 }
	}
}

impl DungeonSpot {
	pub fn approach(&mut self, other: DungeonSpot) {
		if self.row < other.row {
			self.row += 1;
		} else if self.row > other.row {
			self.row -= 1;
		}
		if self.col < other.col {
			self.col += 1;
		} else if self.col > other.col {
			self.col -= 1;
		}
	}
	pub fn path_to(&self, other: DungeonSpot) -> Vec<DungeonSpot> {
		let mut path = Vec::new();
		let mut spot = self.clone();
		loop {
			spot.approach(other);
			path.push(spot);
			if spot == other {
				break;
			}
		}
		path
	}
	pub fn has_attack_vector_to(&self, other: DungeonSpot) -> bool {
		let delta_row = self.row - other.row;
		let delta_col = self.col - other.col;
		delta_row == 0 || delta_col == 0 || delta_row.abs() == delta_col.abs()
	}
	pub fn is_near(&self, other: DungeonSpot) -> bool {
		self.distance_from(other) < 2
	}
	pub fn distance_from(&self, other: DungeonSpot) -> i64 {
		let row_delta = other.row - self.row;
		let col_delta = other.col - self.col;
		let max_delta = row_delta.abs().max(col_delta.abs());
		max_delta
	}
	pub fn after_motion(&self, motion: Motion) -> Option<Self> {
		let mut after = self.clone();
		get_dir_rc(motion.to_char(), &mut after.row, &mut after.col, true);
		if after.is_out_of_bounds() { None } else { Some(after) }
	}
	pub fn from_usize(row: usize, col: usize) -> Self {
		DungeonSpot { row: row as i64, col: col as i64 }
	}
	pub fn shares_axis(&self, other: &Self) -> bool {
		self.row == other.row || self.col == other.col
	}
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
	pub fn is_within_bounds(&self, bounds: &RoomBounds) -> bool {
		self.row >= bounds.top && self.row <= bounds.bottom
			&& self.col >= bounds.left && self.col <= bounds.right
	}
	pub fn set(&mut self, row: i64, col: i64) {
		self.row = row;
		self.col = col;
	}
}

pub mod item_usage {
	pub const NOT_USED: u16 = 0o0;
	pub const BEING_WIELDED: u16 = 0o1;
	pub const BEING_WORN: u16 = 0o2;
	pub const ON_LEFT_HAND: u16 = 0o4;
	pub const ON_RIGHT_HAND: u16 = 0o10;
	pub const ON_EITHER_HAND: u16 = ON_LEFT_HAND | ON_RIGHT_HAND;
	pub const BEING_USED: u16 = BEING_WIELDED | BEING_WORN | ON_EITHER_HAND;
}

pub mod ending;

pub mod object_what {
	use serde::{Deserialize, Serialize};

	use crate::render_system::{AMULET_CHAR, ARMOR_CHAR, FOOD_CHAR, GOLD_CHAR, NOT_CHAR, POTION_CHAR, RING_CHAR, SCROLL_CHAR, WAND_CHAR, WEAPON_CHAR};

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

	impl ObjectWhat {
		pub fn is_object(&self) -> bool {
			self != &ObjectWhat::None
		}
		pub fn to_char(&self) -> char {
			match self {
				ObjectWhat::Scroll => SCROLL_CHAR,
				ObjectWhat::Potion => POTION_CHAR,
				ObjectWhat::Gold => GOLD_CHAR,
				ObjectWhat::Food => FOOD_CHAR,
				ObjectWhat::Wand => WAND_CHAR,
				ObjectWhat::Armor => ARMOR_CHAR,
				ObjectWhat::Weapon => WEAPON_CHAR,
				ObjectWhat::Ring => RING_CHAR,
				ObjectWhat::Amulet => AMULET_CHAR,
				ObjectWhat::None => NOT_CHAR,
			}
		}
	}

	impl Default for ObjectWhat {
		fn default() -> Self { Self::None }
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
}

pub mod food_kind {
	pub const RATION: u16 = 0;
	pub const FRUIT: u16 = 1;
}
