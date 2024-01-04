use serde::{Deserialize, Serialize};
use crate::prelude::ring_kind::RingKind::{AddStrength, Adornment, Dexterity, MaintainArmor, Regeneration, RSeeInvisible, RTeleport, Searching, SlowDigest, Stealth, SustainStrength};

pub const RINGS: usize = 11;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RingKind {
	Stealth,
	RTeleport,
	Regeneration,
	SlowDigest,
	AddStrength,
	SustainStrength,
	Dexterity,
	Adornment,
	RSeeInvisible,
	MaintainArmor,
	Searching,
}

impl RingKind {
	pub const ALL_RINGS: [RingKind; RINGS] = [
		Stealth, RTeleport, Regeneration, SlowDigest, AddStrength, SustainStrength,
		Dexterity, Adornment, RSeeInvisible, MaintainArmor, Searching,
	];
	pub const REAL_NAME: [&'static str; RINGS] = [
		"of stealth ", "of teleportation ", "of regeneration ", "of slow digestion ", "of add strength ", "of sustain strength ",
		"of dexterity ", "of adornment ", "of see invisible ", "of maintain armor ", "of searching ",
	];
	pub fn from_index(index: usize) -> Self {
		Self::ALL_RINGS[index]
	}

	pub fn to_index(&self) -> usize {
		Self::ALL_RINGS.iter().position(|x| x == self).expect("found in ALL")
	}

	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
}

pub const STEALTH: u16 = 0;
pub const R_TELEPORT: u16 = 1;
pub const REGENERATION: u16 = 2;
pub const SLOW_DIGEST: u16 = 3;
pub const ADD_STRENGTH: u16 = 4;
pub const SUSTAIN_STRENGTH: u16 = 5;
pub const DEXTERITY: u16 = 6;
pub const ADORNMENT: u16 = 7;
pub const R_SEE_INVISIBLE: u16 = 8;
pub const MAINTAIN_ARMOR: u16 = 9;
pub const SEARCHING: u16 = 10;
