use serde::{Deserialize, Serialize};
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
	pub const ALL_KINDS: [WeaponKind; WEAPONS] = [
		Bow, Dart, Arrow, Dagger, Shuriken, Mace, LongSword, TwoHandedSword,
	];
	pub fn from_index(index: usize) -> Self { Self::ALL_KINDS[index] }
	pub fn to_index(&self) -> usize { Self::ALL_KINDS.iter().position(|it| it == self).expect("position") }
	pub fn is_kind(&self, index: u16) -> bool { self.to_index() as u16 == index }

	pub const TITLE: [&'static str; WEAPONS] = [
		"short bow ", "darts ", "arrows ", "daggers ", "shurikens ", "mace ", "long sword ", "two-handed sword "
	];
	pub fn title(&self) -> &'static str { &Self::TITLE[self.to_index()] }
}

pub fn damage(kind: u16) -> &'static str {
	let damage = match kind {
		BOW | DART => "1d1",
		ARROW => "1d2",
		DAGGER => "1d3",
		SHURIKEN => "1d4",
		MACE => "2d3",
		LONG_SWORD => "3d4",
		TWO_HANDED_SWORD => "4d5",
		_ => unreachable!("invalid weapon kind")
	};
	damage
}
