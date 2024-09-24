use crate::resources::level::sector::Sector;
use rand::Rng;

pub fn roll_design(rng: &mut impl Rng) -> Design {
	SECTOR_DESIGNS[rng.gen_range(0usize..=5)]
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
		}
	}
}
