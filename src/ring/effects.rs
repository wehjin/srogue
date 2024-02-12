use serde::{Deserialize, Serialize};
use crate::odds;

pub static mut r_teleport: bool = false;
pub static mut regeneration: isize = 0;
pub static mut e_rings: libc::c_short = 0;
pub static mut add_strength: isize = 0;
pub static mut sustain_strength: bool = false;
pub static mut ring_exp: isize = 0;
pub static mut r_see_invisible: bool = false;
pub static mut maintain_armor: bool = false;
pub static mut auto_search: libc::c_short = 0;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct RingEffects {
	stealthy: usize,
}

impl RingEffects {
	pub fn is_stealthy(&self) -> bool { self.stealthy() > 0 }
	pub fn stealthy(&self) -> usize { self.stealthy }
	pub fn incr_stealthy(&mut self) {
		self.stealthy += 1;
	}
	pub fn clear_stealthy(&mut self) {
		self.stealthy = 0;
	}
	pub fn apply_stealthy(&self, chance: usize) -> usize {
		if self.is_stealthy() {
			chance / (odds::STEALTH_FACTOR + self.stealthy)
		} else {
			chance
		}
	}
}
