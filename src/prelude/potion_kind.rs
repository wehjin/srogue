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
	pub const ALL_KINDS: [PotionKind; POTIONS] = [
		IncreaseStrength, RestoreStrength, Healing, ExtraHealing, Poison, RaiseLevel, Blindness,
		Hallucination, DetectMonster, DetectObjects, Confusion, Levitation, HasteSelf, SeeInvisible
	];

	pub fn from_index(index: usize) -> Self {
		Self::ALL_KINDS[index]
	}

	pub fn to_index(&self) -> usize {
		Self::ALL_KINDS.iter().position(|x| x == self).expect("find potion-kind in ALL")
	}
}
