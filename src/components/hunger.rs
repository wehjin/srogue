use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug, Hash, Default)]
pub enum HungerLevel {
	#[default]
	Normal,
	Hungry,
	Weak,
	Faint,
	Starved,
}

impl HungerLevel {
	pub fn as_str(&self) -> &'static str {
		match self {
			HungerLevel::Normal => "normal",
			HungerLevel::Hungry => "hungry",
			HungerLevel::Weak => "weak",
			HungerLevel::Faint => "faint",
			HungerLevel::Starved => "starved",
		}
	}
}

pub const HUNGRY_MOVES_LEFT: isize = 300;
pub const WEAK_MOVES_LEFT: isize = 150;
pub const FAINT_MOVES_LEFT: isize = 20;
pub const STARVE_MOVES_LEFT: isize = 0;
