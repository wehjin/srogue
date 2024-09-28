use serde::{Deserialize, Serialize};

use crate::init::{Dungeon, GameState};
use crate::resources::avatar::Avatar;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Healer {
	pub heal_level: isize,
	pub time_between_heals: isize,
	pub regen_clock: isize,
	pub double_healing_toggle: bool,
}

impl Default for Healer {
	fn default() -> Self {
		Self {
			heal_level: -1,
			time_between_heals: 0,
			regen_clock: 0,
			double_healing_toggle: false,
		}
	}
}

impl GameState {
	pub fn heal_player(&mut self) {
		if self.player.rogue.hp_current == self.player.rogue.hp_max {
			self.healer.regen_clock = 0;
			return;
		}
		let fighter_exp_level = self.as_fighter().exp.level as isize;
		if fighter_exp_level != self.healer.heal_level {
			self.healer.heal_level = fighter_exp_level;
			self.healer.time_between_heals = match self.healer.heal_level {
				1 => 20,
				2 => 18,
				3 => 17,
				4 => 14,
				5 => 13,
				6 => 10,
				7 => 9,
				8 => 8,
				9 => 7,
				10 => 4,
				11 => 3,
				_ => 2,
			}
		}
		self.healer.regen_clock += 1;
		if self.healer.regen_clock >= self.healer.time_between_heals {
			self.healer.regen_clock = 0;
			self.healer.double_healing_toggle = !self.healer.double_healing_toggle;

			self.player.rogue.hp_current += 1;
			if self.healer.double_healing_toggle {
				self.player.rogue.hp_current += 1;
			}
			self.player.rogue.hp_current += self.player.ring_effects.regeneration();
			if self.player.rogue.hp_current > self.player.rogue.hp_max {
				self.player.rogue.hp_current = self.player.rogue.hp_max;
			}

			let diary = self.as_diary_mut();
			diary.stats_changed = true;
		}
	}
}
