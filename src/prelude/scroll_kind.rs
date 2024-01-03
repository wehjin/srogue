use serde::{Deserialize, Serialize};
use crate::prelude::scroll_kind::ScrollKind::{AggravateMonster, CreateMonster, EnchArmor, EnchWeapon, HoldMonster, Identify, MagicMapping, ProtectArmor, RemoveCurse, ScareMonster, Sleep, Teleport};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScrollKind {
	ProtectArmor,
	HoldMonster,
	EnchWeapon,
	EnchArmor,
	Identify,
	Teleport,
	Sleep,
	ScareMonster,
	RemoveCurse,
	CreateMonster,
	AggravateMonster,
	MagicMapping,
}

pub const SCROLLS: usize = 12;

impl ScrollKind {
	pub const ALL_SCROLLS: [ScrollKind; SCROLLS] = [
		ProtectArmor, HoldMonster, EnchWeapon, EnchArmor, Identify, Teleport,
		Sleep, ScareMonster, RemoveCurse, CreateMonster, AggravateMonster, MagicMapping
	];

	pub fn from_index(index: usize) -> Self {
		Self::ALL_SCROLLS[index]
	}
	pub fn to_index(&self) -> usize {
		Self::ALL_SCROLLS.iter().position(|it| it == self).expect("position")
	}

	pub fn is_kind(&self, index: u16) -> bool {
		self.to_index() as u16 == index
	}
}
