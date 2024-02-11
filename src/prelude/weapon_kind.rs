use serde::{Deserialize, Serialize};
use crate::hit::DamageStat;
use crate::prelude::weapon_kind::WeaponKind::{Arrow, Bow, Dagger, Dart, LongSword, Mace, Shuriken, TwoHandedSword};

pub const BOW: u16 = 0;
pub const DART: u16 = 1;
pub const ARROW: u16 = 2;
pub const DAGGER: u16 = 3;
pub const SHURIKEN: u16 = 4;
pub const MACE: u16 = 5;
pub const LONG_SWORD: u16 = 6;
pub const TWO_HANDED_SWORD: u16 = 7;
pub const WEAPONS: usize = 8;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum WeaponKind {
	Bow,
	Dart,
	Arrow,
	Dagger,
	Shuriken,
	Mace,
	LongSword,
	TwoHandedSword,
}

impl WeaponKind {
	pub fn is_arrow_or_throwing_weapon(&self) -> bool {
		*self == Arrow || self.is_throwing_weapon()
	}
	pub fn is_throwing_weapon(&self) -> bool {
		match self {
			Dart => true,
			Dagger => true,
			Shuriken => true,
			_ => false,
		}
	}
	pub fn to_index(&self) -> usize { Self::ALL_KINDS.iter().position(|it| it == self).expect("position") }
	pub fn is_kind(&self, index: u16) -> bool { self.to_index() as u16 == index }
	pub const TITLE: [&'static str; WEAPONS] = [
		"short bow ", "darts ", "arrows ", "daggers ", "shurikens ", "mace ", "long sword ", "two-handed sword "
	];

	pub fn title(&self) -> &'static str { &Self::TITLE[self.to_index()] }
	pub fn damage(&self) -> DamageStat {
		match self {
			Bow | Dart => DamageStat { hits: 1, damage: 1 },
			Arrow => DamageStat { hits: 1, damage: 2 },
			Dagger => DamageStat { hits: 1, damage: 3 },
			Shuriken => DamageStat { hits: 1, damage: 4 },
			Mace => DamageStat { hits: 2, damage: 3 },
			LongSword => DamageStat { hits: 3, damage: 4 },
			TwoHandedSword => DamageStat { hits: 4, damage: 5 },
		}
	}

	pub const ALL_KINDS: [WeaponKind; WEAPONS] = [
		Bow, Dart, Arrow, Dagger, Shuriken, Mace, LongSword, TwoHandedSword,
	];
}

impl From<u16> for WeaponKind {
	fn from(which_kind: u16) -> Self {
		match which_kind {
			BOW => Bow,
			DART => Dart,
			ARROW => Arrow,
			DAGGER => Dagger,
			SHURIKEN => Shuriken,
			MACE => Mace,
			LONG_SWORD => LongSword,
			TWO_HANDED_SWORD => TwoHandedSword,
			_ => unreachable!("invalid weapon kind")
		}
	}
}
