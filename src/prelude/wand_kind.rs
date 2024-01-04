use serde::{Deserialize, Serialize};
use crate::prelude::wand_kind::WandKind::{Cancellation, ConfuseMonster, DoNothing, HasteMonster, Invisibility, MagicMissile, Polymorph, PutToSleep, SlowMonster, TeleAway};

pub const TELE_AWAY: u16 = 0;
pub const SLOW_MONSTER: u16 = 1;
pub const CONFUSE_MONSTER: u16 = 2;
pub const INVISIBILITY: u16 = 3;
pub const POLYMORPH: u16 = 4;
pub const HASTE_MONSTER: u16 = 5;
pub const PUT_TO_SLEEP: u16 = 6;
pub const MAGIC_MISSILE: u16 = 7;
pub const CANCELLATION: u16 = 8;
pub const DO_NOTHING: u16 = 9;
pub const WANDS: usize = 10;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum WandKind {
	TeleAway,
	SlowMonster,
	ConfuseMonster,
	Invisibility,
	Polymorph,
	HasteMonster,
	PutToSleep,
	MagicMissile,
	Cancellation,
	DoNothing,
}

impl WandKind {
	pub const ALL_WANDS: [WandKind; WANDS] = [
		TeleAway, SlowMonster, ConfuseMonster, Invisibility, Polymorph,
		HasteMonster, PutToSleep, MagicMissile, Cancellation, DoNothing,
	];
	pub const REAL_NAME: [&'static str; WANDS] = [
		"of teleport away ", "of slow monster ", "of confuse monster ", "of invisibility ", "of polymorph ",
		"of haste monster ", "of sleep ", "of magic missile ", "of cancellation ", "of do nothing ",
	];
	pub fn from_index(index: usize) -> Self {
		Self::ALL_WANDS[index]
	}

	pub fn to_index(&self) -> usize {
		Self::ALL_WANDS.iter().position(|x| x == self).expect("found in ALL")
	}

	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
}