use serde::{Deserialize, Serialize};
use crate::hit::DamageStat;
use crate::weapons::constants::{ARROW, BOW, DAGGER, DART, LONG_SWORD, MACE, SHURIKEN, TWO_HANDED_SWORD, WEAPON_NAMES};
use crate::weapons::kind::WeaponKind::{Arrow, Bow, Dagger, Dart, LongSword, Mace, Shuriken, TwoHandedSword};

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
	pub fn name(&self) -> &'static str { &WEAPON_NAMES[self.to_index()] }
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
