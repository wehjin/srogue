use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash, Default)]
pub enum RogueEnergy {
	#[default]
	Normal,
	Hungry,
	Weak,
	Faint,
	Starved,
}

impl RogueEnergy {
	pub fn from_moves(moves_left: isize) -> Self {
		let moves_left = moves_left as u64;
		match moves_left {
			0..MIN_FAINT => Self::Starved,
			MIN_FAINT..MIN_WEAK => Self::Faint,
			MIN_WEAK..MIN_HUNGRY => Self::Weak,
			MIN_HUNGRY..MIN_SATIATED => Self::Hungry,
			MIN_SATIATED..=u64::MAX => Self::Normal,
		}
	}
	pub fn as_stat(&self) -> &'static str {
		if self == &RogueEnergy::Normal {
			""
		} else {
			self.as_report()
		}
	}
	pub fn as_report(&self) -> &'static str {
		match self {
			RogueEnergy::Normal => "normal",
			RogueEnergy::Hungry => "hungry",
			RogueEnergy::Weak => "weak",
			RogueEnergy::Faint => "faint",
			RogueEnergy::Starved => "starved",
		}
	}
	pub const MAX_HUNGRY: isize = (MIN_SATIATED - 1) as isize;
	pub const MAX_FAINT: isize = (MIN_WEAK - 1) as isize;
}

const MIN_FAINT: u64 = 1;
const MIN_WEAK: u64 = 21;
const MIN_HUNGRY: u64 = 151;
const MIN_SATIATED: u64 = 301;
