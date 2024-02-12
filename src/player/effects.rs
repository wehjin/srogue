use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct TimeEffect(usize);

impl TimeEffect {
	pub fn is_active(&self) -> bool {
		self.0 > 0
	}
	pub fn is_inactive(&self) -> bool {
		!self.is_active()
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
	pub fn halve(&mut self) {
		if self.0 > 0 {
			self.0 = (self.0 / 2) + 1;
		}
	}
}
