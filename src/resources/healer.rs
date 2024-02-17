use serde::{Deserialize, Serialize};

use crate::message::print_stats;
use crate::player::Player;
use crate::prelude::stat_const::STAT_HP;

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

impl Healer {
	pub fn heal(&mut self, player: &mut Player) {
		if player.rogue.hp_current == player.rogue.hp_max {
			self.regen_clock = 0;
			return;
		}
		if player.rogue.exp != self.heal_level {
			self.heal_level = player.rogue.exp;
			self.time_between_heals = match self.heal_level {
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
		self.regen_clock += 1;
		if self.regen_clock >= self.time_between_heals {
			self.regen_clock = 0;
			self.double_healing_toggle = !self.double_healing_toggle;

			player.rogue.hp_current += 1;
			if self.double_healing_toggle {
				player.rogue.hp_current += 1;
			}
			player.rogue.hp_current += player.ring_effects.regeneration();
			if player.rogue.hp_current > player.rogue.hp_max {
				player.rogue.hp_current = player.rogue.hp_max;
			}
			print_stats(STAT_HP, player);
		}
	}
}
