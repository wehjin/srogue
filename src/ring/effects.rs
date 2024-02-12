use serde::{Deserialize, Serialize};
use crate::odds;

pub static mut add_strength: isize = 0;
pub static mut sustain_strength: bool = false;
pub static mut ring_exp: isize = 0;
pub static mut r_see_invisible: bool = false;
pub static mut maintain_armor: bool = false;
pub static mut auto_search: libc::c_short = 0;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct RingEffects {
	stealthy: usize,
	r_teleport: bool,
	regeneration: isize,
	e_rings: isize,
}

impl RingEffects {
	pub fn calorie_burn(&self) -> isize {
		self.e_rings
	}
	pub fn clear_calorie_burn(&mut self) {
		self.e_rings = 0;
	}
	pub fn incr_calorie_burn(&mut self) {
		self.e_rings += 1;
	}
	pub fn slow_calorie_burn(&mut self) {
		self.e_rings -= 2;
	}
}

impl RingEffects {
	pub fn regeneration(&self) -> isize {
		self.regeneration
	}
	pub fn clear_regeneration(&mut self) {
		self.regeneration = 0;
	}
	pub fn incr_regeneration(&mut self) {
		self.regeneration += 1;
	}
}

impl RingEffects {
	pub fn has_teleport(&self) -> bool { self.r_teleport }
	pub fn set_teleport(&mut self, enable: bool) {
		self.r_teleport = enable;
	}
}

impl RingEffects {
	pub fn stealthy(&self) -> usize { self.stealthy }
	pub fn is_stealthy(&self) -> bool { self.stealthy() > 0 }
	pub fn clear_stealthy(&mut self) {
		self.stealthy = 0;
	}
	pub fn incr_stealthy(&mut self) {
		self.stealthy += 1;
	}
	pub fn apply_stealthy(&self, chance: usize) -> usize {
		if self.is_stealthy() {
			chance / (odds::STEALTH_FACTOR + self.stealthy)
		} else {
			chance
		}
	}
}
