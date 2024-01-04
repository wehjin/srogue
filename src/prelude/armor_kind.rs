use serde::{Deserialize, Serialize};
use crate::prelude::armor_kind::ArmorKind::{Banded, Chain, Leather, Plate, Ringmail, Scale, Splint};

pub const LEATHER: u16 = 0;
pub const RINGMAIL: u16 = 1;
pub const SCALE: u16 = 2;
pub const CHAIN: u16 = 3;
pub const BANDED: u16 = 4;
pub const SPLINT: u16 = 5;
pub const PLATE: u16 = 6;
pub const ARMORS: usize = 7;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArmorKind {
	Leather,
	Ringmail,
	Scale,
	Chain,
	Banded,
	Splint,
	Plate,
}

impl ArmorKind {
	pub const ALL_KINDS: [ArmorKind; ARMORS] = [
		Leather, Ringmail, Scale, Chain, Banded, Splint, Plate,
	];
	pub fn from_index(index: usize) -> Self { Self::ALL_KINDS[index] }
	pub fn to_index(&self) -> usize { Self::ALL_KINDS.iter().position(|it| it == self).expect("position") }
	pub fn is_kind(&self, index: u16) -> bool { self.to_index() as u16 == index }

	pub const TITLE: [&'static str; ARMORS] = [
		"leather armor ", "ring mail ", "scale mail ", "chain mail ", "banded mail ", "splint mail ", "plate mail ",
	];
	pub fn title(&self) -> &'static str { &Self::TITLE[self.to_index()] }
}