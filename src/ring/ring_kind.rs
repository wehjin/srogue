use serde::{Deserialize, Serialize};
use crate::objects::{Note, NoteStatus, Title};
use crate::ring::constants::RINGS;
use crate::ring::ring_kind::RingKind::{AddStrength, Adornment, Dexterity, MaintainArmor, Regeneration, RSeeInvisible, RTeleport, Searching, SlowDigest, Stealth, SustainStrength};

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
	pub fn to_index(&self) -> usize {
		Self::ALL_RINGS.iter().position(|x| x == self).expect("found in ALL")
	}
	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
}

impl RingKind {
	pub const ALL_RINGS: [RingKind; RINGS] = [
		Stealth, RTeleport, Regeneration, SlowDigest, AddStrength, SustainStrength,
		Dexterity, Adornment, RSeeInvisible, MaintainArmor, Searching,
	];
	pub const fn from_index(index: usize) -> Self {
		Self::ALL_RINGS[index]
	}
	pub const REAL_NAME: [&'static str; RINGS] = [
		"of stealth ", "of teleportation ", "of regeneration ", "of slow digestion ", "of add strength ", "of sustain strength ",
		"of dexterity ", "of adornment ", "of see invisible ", "of maintain armor ", "of searching ",
	];
	pub const fn to_id(self) -> Note {
		Note {
			title: Title::None,
			status: NoteStatus::Unidentified,
			is_wood: false,
		}
	}
}
