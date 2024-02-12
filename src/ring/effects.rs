use serde::{Deserialize, Serialize};
use crate::odds;

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
	add_strength: isize,
}

impl RingEffects {
	pub fn add_strength(&self) -> isize { self.add_strength }
	pub fn has_add_strength(&self) -> bool { self.add_strength() > 0 }
	pub fn clear_add_strength(&mut self) {
		self.add_strength = 0;
	}
	pub fn increase_add_strength(&mut self, amount: isize) {
		self.add_strength += amount;
	}
	pub fn apply_add_strength(&self, str: isize) -> isize {
		str + self.add_strength
	}
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
