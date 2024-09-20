use crate::random::{get_rand, rand_percent};
use crate::resources::level::sector::Sector;
use crate::resources::level::LevelType;

pub fn roll_design(level_type: LevelType) -> Design {
	match level_type {
		LevelType::PartyAlways => Design::BigRoom,
		LevelType::PartyRoll if rand_percent(1) => Design::BigRoom,
		_ => SECTOR_DESIGNS[get_rand(0usize, 5)]
	}
}

pub const SECTOR_DESIGNS: [Design; 6] = [
	Design::RequireTop, Design::RequireMiddle, Design::RequireBottom,
	Design::RequireLeft, Design::RequireCenter, Design::RequireRight
];

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Design {
	RequireTop = 0,
	RequireMiddle = 1,
	RequireBottom = 2,
	RequireLeft = 3,
	RequireCenter = 4,
	RequireRight = 5,
	BigRoom = 6,
}

impl Design {
	pub fn requires_room_in_sector(&self, sector: Sector) -> bool {
		match self {
			Design::RequireTop => sector.is_top(),
			Design::RequireMiddle => sector.is_middle(),
			Design::RequireBottom => sector.is_bottom(),
			Design::RequireLeft => sector.is_left(),
			Design::RequireCenter => sector.is_center(),
			Design::RequireRight => sector.is_right(),
			Design::BigRoom => false,
		}
	}
}
