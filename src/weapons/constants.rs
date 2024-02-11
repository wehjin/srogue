use crate::weapons::WeaponKind;

pub const BOW: u16 = 0;
pub const DART: u16 = 1;
pub const ARROW: u16 = 2;
pub const DAGGER: u16 = 3;
pub const SHURIKEN: u16 = 4;
pub const MACE: u16 = 5;
pub const LONG_SWORD: u16 = 6;
pub const TWO_HANDED_SWORD: u16 = 7;
pub const WEAPONS: usize = 8;

impl WeaponKind {
	pub const fn value(&self) -> i16 {
		match self {
			WeaponKind::Bow => 150,
			WeaponKind::Dart => 8,
			WeaponKind::Arrow => 15,
			WeaponKind::Dagger => 27,
			WeaponKind::Shuriken => 35,
			WeaponKind::Mace => 360,
			WeaponKind::LongSword => 470,
			WeaponKind::TwoHandedSword => 580,
		}
	}
}

