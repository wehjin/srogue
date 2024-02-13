use serde::{Deserialize, Serialize};

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
	pub fn from_index(index: usize) -> Self {
		Self::ALL_SCROLLS[index]
	}
	pub fn to_index(&self) -> usize {
		Self::ALL_SCROLLS.iter().position(|it| it == self).expect("position")
	}
	pub fn is_kind(&self, index: u16) -> bool {
		self.to_index() as u16 == index
	}
	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
}
