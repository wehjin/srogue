use ncurses::chtype;
use rand::{RngCore, thread_rng};
use serde::{Deserialize, Serialize};
use crate::monster::{MonsterFlags, MonsterKind};
use crate::prelude::{DungeonSpot};
use crate::room::get_opt_room_number;

pub static mut MASH: MonsterMash = MonsterMash::new();

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MonsterMash {
	pub monsters: Vec<Monster>,
}

impl MonsterMash {
	pub const fn new() -> Self { MonsterMash { monsters: Vec::new() } }

	pub fn clear(&mut self) {
		self.monsters.clear();
	}
	pub fn add_monster(&mut self, monster: Monster) {
		self.monsters.push(monster);
	}
	pub fn remove_monster(&mut self, id: u64) {
		let index = self.monsters.iter().position(|m| m.id == id);
		if let Some(index) = index {
			self.monsters.remove(index);
		}
	}
	pub fn is_empty(&self) -> bool { self.monsters.is_empty() }
	fn monsters_index_at_spot(&self, row: i64, col: i64) -> Option<usize> {
		for i in 0..self.monsters.len() {
			if self.monsters[i].spot.is_at(row, col) {
				return Some(i);
			}
		}
		return None;
	}
	pub fn monster_at_spot(&self, row: i64, col: i64) -> Option<&Monster> {
		match self.monsters_index_at_spot(row, col) {
			Some(index) => Some(&self.monsters[index]),
			None => None,
		}
	}
	pub fn monster_at_spot_mut(&mut self, row: i64, col: i64) -> Option<&mut Monster> {
		match self.monsters_index_at_spot(row, col) {
			Some(index) => Some(&mut self.monsters[index]),
			None => None,
		}
	}
	fn monsters_index_with_id(&self, id: u64) -> Option<usize> {
		for i in 0..self.monsters.len() {
			if self.monsters[i].id == id {
				return Some(i);
			}
		}
		return None;
	}

	pub fn monster_with_id(&self, id: u64) -> Option<&Monster> {
		match self.monsters_index_with_id(id) {
			Some(index) => Some(&self.monsters[index]),
			None => None,
		}
	}
	pub fn monster_with_id_mut(&mut self, id: u64) -> Option<&mut Monster> {
		match self.monsters_index_with_id(id) {
			Some(index) => Some(&mut self.monsters[index]),
			None => None,
		}
	}
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct StuckCounter {
	pub row: i64,
	pub col: i64,
	pub count: usize,
}

impl StuckCounter {
	pub fn log_row_col(&mut self, row: i64, col: i64) {
		if self.row == row && self.col == col {
			self.count += 1;
		} else {
			self.row = row;
			self.col = col;
			self.count = 0;
		}
	}
	pub fn reset(&mut self) {
		self.count = 0;
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Monster {
	id: u64,
	pub kind: MonsterKind,
	pub m_flags: MonsterFlags,
	pub spot: DungeonSpot,
	pub trail_char: chtype,
	pub disguise_char: chtype,
	pub slowed_toggle: bool,
	pub target_spot: Option<DungeonSpot>,
	pub nap_length: isize,
	pub stuck_counter: StuckCounter,
	pub moves_confused: isize,
	pub stationary_damage: isize,
	pub hp_to_kill: isize,
	pub killed: bool,
	pub drop_percent: usize,
}

impl Monster {
	pub fn create(kind: MonsterKind) -> Self {
		Monster {
			id: thread_rng().next_u64(),
			kind: kind.clone(),
			m_flags: kind.flags(),
			spot: DungeonSpot::default(),
			trail_char: 0,
			disguise_char: 0,
			slowed_toggle: false,
			target_spot: None,
			nap_length: 0,
			stuck_counter: StuckCounter::default(),
			moves_confused: 0,
			stationary_damage: 0,
			hp_to_kill: kind.hp_to_kill(),
			killed: false,
			drop_percent: 0,
		}
	}
	pub fn do_nap(&mut self) {
		self.nap_length -= 1;
		if self.nap_length <= 0 {
			self.m_flags.napping = false;
			self.m_flags.asleep = false;
		}
	}
	pub fn set_spot(&mut self, row: i64, col: i64) {
		self.spot.row = row;
		self.spot.col = col;
	}
	pub fn set_target_spot(&mut self, row: i64, col: i64) {
		self.target_spot = Some(DungeonSpot { row, col })
	}
	pub fn clear_target_spot(&mut self) {
		self.target_spot = None;
		self.stuck_counter.reset();
	}
	pub fn clear_target_spot_if_reached(&mut self) {
		if self.at_target_spot() {
			self.target_spot = None
		}
	}
	pub fn decrement_moves_confused(&mut self) {
		self.moves_confused -= 1;
		if self.moves_confused <= 0 {
			self.m_flags.confuses = false;
		}
	}
	pub fn id(&self) -> u64 { self.id }
	pub fn kill_exp(&self) -> isize { self.kind.kill_exp() }
	pub fn m_hit_chance(&self) -> usize { self.kind.m_hit_chance() }
	pub fn m_damage(&self) -> &'static str { self.kind.damage() }
	pub fn wanders_or_wakens(&self) -> bool { self.m_flags.wakens || self.m_flags.wanders }
	pub fn is_invisible(&self) -> bool { self.m_flags.invisible }
	pub fn name(&self) -> &'static str { self.kind.name() }
	pub fn in_room(&self, rn: i64) -> bool {
		let monster_rn = get_opt_room_number(self.spot.row, self.spot.col);
		if let Some(monster_rn) = monster_rn {
			monster_rn == (rn as usize)
		} else {
			false
		}
	}
	pub fn target_spot_or(&self, fallback_spot: DungeonSpot) -> DungeonSpot {
		if let Some(target_spot) = self.target_spot {
			target_spot
		} else {
			fallback_spot
		}
	}
	pub fn at_target_spot(&self) -> bool {
		self.target_spot == Some(self.spot)
	}
	pub fn m_char(&self) -> chtype {
		self.kind.m_char()
	}
	pub fn flip_slowed_toggle(&mut self) {
		if self.slowed_toggle {
			self.slowed_toggle = false;
		} else {
			self.slowed_toggle = true;
		}
	}
	pub fn slowed_toggle(&self) -> bool {
		self.slowed_toggle
	}
	pub fn wake_up(&mut self) {
		self.m_flags.wake_up();
	}
}
