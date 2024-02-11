use serde::{Deserialize, Serialize};
use crate::armors::ArmorKind::{Banded, Chain, Leather, Plate, Ringmail, Scale, Splint};
use crate::armors::constants::ARMORS;

pub(crate) mod constants;

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
