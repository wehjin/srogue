use std::collections::HashSet;
use libc::{c_ushort};
pub use crate::message::*;
pub use crate::level::*;
pub use crate::monster::*;
pub use crate::hit::*;
pub use crate::init::*;
pub use crate::instruct::*;
pub use crate::inventory::*;
pub use crate::machdep::*;
pub use crate::r#move::*;
pub use crate::objects::*;
pub use crate::pack::*;
pub use crate::play::*;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Object, Stairs, Trap, Tunnel, VertWall};
pub use crate::random::*;
pub use crate::ring::*;
pub use crate::room::*;
pub use crate::save::*;
pub use crate::score::*;
pub use crate::spec_hit::*;
pub use crate::throw::*;
pub use crate::trap::*;
pub use crate::r#use::*;
pub use crate::zap::*;


pub const MAXROOMS: i64 = 9;
pub const NO_ROOM: i64 = -1;
pub const PASSAGE: i64 = -3;
pub const BIG_ROOM: usize = 10;
pub const R_ROOM: c_ushort = 2;
pub const MIN_ROW: i64 = 1;
pub const DCOLS: usize = 80;
pub const DROWS: usize = 24;
pub const MAX_TITLE_LENGTH: usize = 30;
pub const MORE: &'static str = "-more-";
pub const MAXSYLLABLES: usize = 40;
pub const MAX_METAL: usize = 14;
pub const WAND_MATERIALS: usize = 30;
pub const GEMS: usize = 14;
pub const COL1: i64 = 26;
pub const COL2: i64 = 52;
pub const ROW1: i64 = 7;
pub const ROW2: i64 = 15;
pub const HIDE_PERCENT: usize = 12;

pub const AMULET_LEVEL: isize = 26;


pub const MAX_EXP_LEVEL: isize = 21;
pub const MAX_EXP: isize = 10000000;
pub const MAX_GOLD: isize = 900000;
pub const MAX_ARMOR: isize = 99;
pub const MAX_HP: isize = 800;
pub const MAX_STRENGTH: isize = 99;
pub const LAST_DUNGEON: usize = 99;
pub const INIT_HP: isize = 12;
pub const PARTY_TIME: isize = 10;   /* one party somewhere in each 10 level span */

#[derive(Copy, Clone)]
pub struct DungeonSpot {
	pub col: i64,
	pub row: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum SpotFlag {
	Nothing = 0x0,
	Object = 0o1,
	Monster = 0o2,
	Stairs = 0o4,
	HorWall = 0o10,
	VertWall = 0o20,
	Door = 0o40,
	Floor = 0o100,
	Tunnel = 0o200,
	Trap = 0o400,
	Hidden = 0o1000,
}

impl SpotFlag {
	pub fn union(flags: &Vec<SpotFlag>) -> c_ushort {
		flags.iter().fold(0, |it, more| it & more.code())
	}
	pub fn is_any_set(flags: &Vec<SpotFlag>, value: c_ushort) -> bool {
		for flag in flags {
			if flag.is_set(value) {
				return true;
			}
		}
		return false;
	}

	pub fn are_others_set(flags: &Vec<SpotFlag>, value: c_ushort) -> bool {
		let all = vec![Object, Monster, Stairs, HorWall, VertWall, Door, Floor, Tunnel, Trap, Hidden];
		let a = all.into_iter().collect::<HashSet<_>>();
		let b = flags.iter().cloned().collect::<HashSet<_>>();
		let c = a.difference(&b).cloned().collect::<Vec<_>>();
		SpotFlag::is_any_set(&c, value)
	}

	pub fn is_set(&self, value: c_ushort) -> bool {
		match self {
			SpotFlag::Nothing => value == 0,
			_ => (value & self.code()) != 0,
		}
	}
	pub fn is_only(&self, value: c_ushort) -> bool {
		value == self.code()
	}
	pub fn clear(&self, value: &mut c_ushort) {
		let code = self.code();
		*value &= !code;
	}
	pub fn set(&self, value: &mut c_ushort) {
		let code = self.code();
		*value |= code;
	}
	pub fn code(&self) -> c_ushort {
		match self {
			SpotFlag::Nothing => 0o0,
			SpotFlag::Object => 0o1,
			SpotFlag::Monster => 0o2,
			SpotFlag::Stairs => 0o4,
			SpotFlag::HorWall => 0o10,
			SpotFlag::VertWall => 0o20,
			SpotFlag::Door => 0o40,
			SpotFlag::Floor => 0o100,
			SpotFlag::Tunnel => 0o200,
			SpotFlag::Trap => 0o400,
			SpotFlag::Hidden => 0o1000,
		}
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

pub mod scroll_kind {
	pub const PROTECT_ARMOR: u16 = 0;
	pub const HOLD_MONSTER: u16 = 1;
	pub const ENCH_WEAPON: u16 = 2;
	pub const ENCH_ARMOR: u16 = 3;
	pub const IDENTIFY: u16 = 4;
	pub const TELEPORT: u16 = 5;
	pub const SLEEP: u16 = 6;
	pub const SCARE_MONSTER: u16 = 7;
	pub const REMOVE_CURSE: u16 = 8;
	pub const CREATE_MONSTER: u16 = 9;
	pub const AGGRAVATE_MONSTER: u16 = 10;
	pub const MAGIC_MAPPING: u16 = 11;
	pub const SCROLLS: usize = 12;
}

pub mod wand_kind {
	pub const TELE_AWAY: u16 = 0;
	pub const SLOW_MONSTER: u16 = 1;
	pub const CONFUSE_MONSTER: u16 = 2;
	pub const INVISIBILITY: u16 = 3;
	pub const POLYMORPH: u16 = 4;
	pub const HASTE_MONSTER: u16 = 5;
	pub const PUT_TO_SLEEP: u16 = 6;
	pub const MAGIC_MISSILE: u16 = 7;
	pub const CANCELLATION: u16 = 8;
	pub const DO_NOTHING: u16 = 9;
	pub const WANDS: usize = 10;
}

pub mod food_kind {
	pub const RATION: u16 = 0;
	pub const FRUIT: u16 = 1;
}

pub mod weapon_kind {
	pub const BOW: u16 = 0;
	pub const DART: u16 = 1;
	pub const ARROW: u16 = 2;
	pub const DAGGER: u16 = 3;
	pub const SHURIKEN: u16 = 4;
	pub const MACE: u16 = 5;
	pub const LONG_SWORD: u16 = 6;
	pub const TWO_HANDED_SWORD: u16 = 7;
	pub const WEAPONS: usize = 8;
}

pub mod armor_kind {
	pub const LEATHER: u16 = 0;
	pub const RINGMAIL: u16 = 1;
	pub const SCALE: u16 = 2;
	pub const CHAIN: u16 = 3;
	pub const BANDED: u16 = 4;
	pub const SPLINT: u16 = 5;
	pub const PLATE: u16 = 6;
	pub const ARMORS: usize = 7;
}

pub mod potion_kind {
	pub const INCREASE_STRENGTH: u16 = 0;
	pub const RESTORE_STRENGTH: u16 = 1;
	pub const HEALING: u16 = 2;
	pub const EXTRA_HEALING: u16 = 3;
	pub const POISON: u16 = 4;
	pub const RAISE_LEVEL: u16 = 5;
	pub const BLINDNESS: u16 = 6;
	pub const HALLUCINATION: u16 = 7;
	pub const DETECT_MONSTER: u16 = 8;
	pub const DETECT_OBJECTS: u16 = 9;
	pub const CONFUSION: u16 = 10;
	pub const LEVITATION: u16 = 11;
	pub const HASTE_SELF: u16 = 12;
	pub const SEE_INVISIBLE: u16 = 13;
	pub const POTIONS: usize = 14;
}

pub mod ring_kind;

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
