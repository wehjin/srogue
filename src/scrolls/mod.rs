use serde::{Deserialize, Serialize};
use crate::scrolls::constants::SCROLLS;
use crate::scrolls::ScrollKind::{AggravateMonster, CreateMonster, EnchArmor, EnchWeapon, HoldMonster, Identify, MagicMapping, ProtectArmor, RemoveCurse, ScareMonster, Sleep, Teleport};

pub(crate) mod constants;

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

	pub const REAL_NAME: [&'static str; SCROLLS] = [
		"of protect armor ", "of hold monster ", "of enchant weapon ", "of enchant armor ", "of identify ", "of teleportation ",
		"of sleep ", "of scare monster ", "of remove curse ", "of create monster ", "of aggravate monster ", "of magic mapping "
	];
	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
}
