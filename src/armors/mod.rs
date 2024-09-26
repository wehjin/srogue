use serde::{Deserialize, Serialize};
use crate::armors::constants::ARMOR_NAMES;

pub(crate) mod constants;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
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
	pub fn from_index(index: usize) -> Self { Self::ALL_KINDS[index] }
	pub fn to_index(&self) -> usize { Self::ALL_KINDS.iter().position(|it| it == self).expect("position") }
	pub fn is_kind(&self, index: u16) -> bool { self.to_index() as u16 == index }
	pub fn name(&self) -> &'static str {
		&ARMOR_NAMES[self.to_index()]
	}
}
