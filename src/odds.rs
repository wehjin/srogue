use crate::level::Level;

pub const WAKE_PERCENT: usize = 45;
pub const FLIT_PERCENT: usize = 33;
pub const PARTY_WAKE_PERCENT: usize = 75;
pub const STEALTH_FACTOR: usize = 3;
pub const R_TELE_PERCENT: usize = 8;
pub const GOLD_PERCENT: usize = 46;

impl Level {
	pub fn room_wake_percent(&self, rn: usize) -> usize {
		if Some(rn) == self.party_room {
			PARTY_WAKE_PERCENT
		} else {
			WAKE_PERCENT
		}
	}
}