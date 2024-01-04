use serde::{Deserialize, Serialize};
use crate::prelude::potion_kind::PotionKind::{Blindness, Confusion, DetectMonster, DetectObjects, ExtraHealing, Hallucination, HasteSelf, Healing, IncreaseStrength, Levitation, Poison, RaiseLevel, RestoreStrength, SeeInvisible};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PotionKind {
	IncreaseStrength,
	RestoreStrength,
	Healing,
	ExtraHealing,
	Poison,
	RaiseLevel,
	Blindness,
	Hallucination,
	DetectMonster,
	DetectObjects,
	Confusion,
	Levitation,
	HasteSelf,
	SeeInvisible,
}

pub const POTIONS: usize = 14;

impl PotionKind {
	pub const ALL_POTIONS: [PotionKind; POTIONS] = [
		IncreaseStrength, RestoreStrength, Healing, ExtraHealing, Poison, RaiseLevel, Blindness,
		Hallucination, DetectMonster, DetectObjects, Confusion, Levitation, HasteSelf, SeeInvisible
	];
	pub const REAL_NAME: [&'static str; POTIONS] = [
		"of increase strength ",
		"of restore strength ",
		"of healing ",
		"of extra healing ",
		"of poison ",
		"of raise level ",
		"of blindness ",
		"of hallucination ",
		"of detect monster ",
		"of detect things ",
		"of confusion ",
		"of levitation ",
		"of haste self ",
		"of see invisible ",
	];

	pub fn from_index(index: usize) -> Self {
		Self::ALL_POTIONS[index]
	}

	pub fn to_index(&self) -> usize {
		Self::ALL_POTIONS.iter().position(|x| x == self).expect("find potion-kind in ALL")
	}

	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
}
