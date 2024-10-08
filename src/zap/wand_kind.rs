use serde::{Deserialize, Serialize};
use crate::objects::{Note, NoteStatus, Title};
use crate::zap::constants::WANDS;
use crate::zap::wand_kind::WandKind::{Cancellation, ConfuseMonster, DoNothing, HasteMonster, Invisibility, MagicMissile, Polymorph, PutToSleep, SlowMonster, TeleAway};

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
	pub const fn from_index(index: usize) -> Self {
		Self::ALL_WANDS[index]
	}

	pub const REAL_NAME: [&'static str; WANDS] = [
		"of teleport away ", "of slow monster ", "of confuse monster ", "of invisibility ", "of polymorph ",
		"of haste monster ", "of sleep ", "of magic missile ", "of cancellation ", "of do nothing ",
	];

	pub fn real_name(&self) -> &'static str {
		&Self::REAL_NAME[self.to_index()]
	}
	pub const fn to_id(self) -> Note {
		Note {
			title: Title::None,
			status: NoteStatus::Unidentified,
			is_wood: false,
		}
	}
}

impl WandKind {
	pub fn to_index(&self) -> usize {
		Self::ALL_WANDS.iter().position(|x| x == self).expect("found in ALL")
	}
}
