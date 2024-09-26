use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TimeEffect(usize);

impl TimeEffect {
	pub fn is_active(&self) -> bool {
		self.0 > 0
	}
	pub fn is_inactive(&self) -> bool {
		!self.is_active()
	}

	pub fn is_half_active(&self) -> bool {
		(self.0 % 2) != 0
	}
	pub fn clear(&mut self) {
		self.0 = 0;
	}
	pub fn decr(&mut self) {
		if self.0 > 0 {
			self.0 -= 1;
		}
	}
	pub fn extend(&mut self, amount: usize) {
		self.0 += amount;
	}
	pub fn ensure_half_active(&mut self) {
		if !self.is_half_active() {
			self.extend(1);
		}
	}
	pub fn halve(&mut self) {
		if self.0 > 0 {
			self.0 = (self.0 / 2) + 1;
		}
	}
}
