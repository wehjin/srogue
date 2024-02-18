use ncurses::chtype;
use rand::{RngCore, thread_rng};
use serde::{Deserialize, Serialize};

use crate::hit::DamageStat;
use crate::level::{DungeonCell, Level};
use crate::monster::{MonsterFlags, MonsterKind};
use crate::player::RoomMark;
use crate::prelude::DungeonSpot;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MonsterMash {
	pub monsters: Vec<Monster>,
	pub m_moves: usize,
	pub mon_disappeared: bool,
}

impl MonsterMash {
	pub const fn new() -> Self {
		MonsterMash {
			monsters: Vec::new(),
			m_moves: 0,
			mon_disappeared: false,
		}
	}

	pub fn clear(&mut self) {
		self.monsters.clear();
		self.m_moves = 0;
	}
}

impl MonsterMash {
	pub fn is_empty(&self) -> bool { self.monsters.is_empty() }
	pub fn monster_ids(&self) -> Vec<u64> {
		self.monsters.iter().map(Monster::id).collect()
	}
	pub fn add_monster(&mut self, monster: Monster) {
		self.monsters.push(monster);
	}
	pub fn remove_monster(&mut self, id: u64) -> Monster {
		let index = self.monsters.iter().position(|m| m.id == id);
		let index = index.expect("monster to remove must be in the list");
		self.monsters.remove(index)
	}
	pub fn monster_id_at_spot(&self, row: i64, col: i64) -> Option<u64> {
		match self.monster_index_at_spot(row, col) {
			Some(index) => Some(self.monsters[index].id),
			None => None,
		}
	}
	pub fn monster_at_spot(&self, row: i64, col: i64) -> Option<&Monster> {
		match self.monster_index_at_spot(row, col) {
			Some(index) => Some(&self.monsters[index]),
			None => None,
		}
	}
	pub fn monster_at_spot_mut(&mut self, row: i64, col: i64) -> Option<&mut Monster> {
		match self.monster_index_at_spot(row, col) {
			Some(index) => Some(&mut self.monsters[index]),
			None => None,
		}
	}
	fn monster_index_at_spot(&self, row: i64, col: i64) -> Option<usize> {
		for i in 0..self.monsters.len() {
			if self.monsters[i].spot.is_at(row, col) {
				return Some(i);
			}
		}
		return None;
	}
	pub fn monster_flags(&self, id: u64) -> &MonsterFlags {
		let monster = self.monster(id);
		let flags = &monster.m_flags;
		flags
	}
	pub fn monster_flags_mut(&mut self, id: u64) -> &mut MonsterFlags {
		let monster = self.monster_mut(id);
		let flags = &mut monster.m_flags;
		flags
	}
	pub fn test_monster(&self, id: u64, f: impl Fn(&Monster) -> bool) -> bool {
		let monster = self.monster(id);
		f(monster)
	}
	pub fn monster(&self, id: u64) -> &Monster {
		let index = self.monster_index(id);
		&self.monsters[index]
	}

	pub fn monster_mut(&mut self, id: u64) -> &mut Monster {
		let index = self.monster_index(id);
		&mut self.monsters[index]
	}
	fn monster_index(&self, id: u64) -> usize {
		for i in 0..self.monsters.len() {
			if self.monsters[i].id == id {
				return i;
			}
		}
		panic!("id not in monsters")
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
	pub fn flies(&self) -> bool { self.m_flags.flies }
	pub fn is_napping(&self) -> bool { self.m_flags.napping }
	pub fn is_asleep(&self) -> bool { self.m_flags.asleep }
	pub fn is_invisible(&self) -> bool { self.m_flags.invisible }
	pub fn is_hasted(&self) -> bool { self.m_flags.hasted }
	pub fn is_slowed(&self) -> bool { self.m_flags.slowed }
	pub fn is_confused(&self) -> bool { self.m_flags.confused }
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
			drop_percent: kind.drop_percent(),
		}
	}
	pub fn cell_mut<'a>(&self, level: &'a mut Level) -> &'a mut DungeonCell {
		&mut level.dungeon[self.spot.row as usize][self.spot.col as usize]
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
	pub fn cur_room(&self, level: &Level) -> RoomMark {
		level.room(self.spot.row, self.spot.col)
	}
	pub fn sees(&self, row: i64, col: i64, level: &Level) -> bool {
		let spot_room = level.room(row, col);
		if spot_room == self.cur_room(level)
			&& spot_room.is_area()
			&& !spot_room.is_maze(level) {
			return true;
		}
		let row_diff = row - self.spot.row;
		let col_diff = col - self.spot.col;
		row_diff >= -1 && row_diff <= 1 && col_diff >= -1 && col_diff <= 1
	}
	pub fn id(&self) -> u64 { self.id }
	pub fn kill_exp(&self) -> isize { self.kind.kill_exp() }
	pub fn m_hit_chance(&self) -> usize { self.kind.m_hit_chance() }
	pub fn m_damage(&self) -> &'static [DamageStat] {
		self.kind.damage()
	}
	pub fn wanders_or_wakens(&self) -> bool { self.m_flags.wakens || self.m_flags.wanders }
	pub fn name(&self) -> &'static str { self.kind.name() }
	pub fn in_room(&self, rn: usize, level: &Level) -> bool {
		if let RoomMark::Cavern(mon_room) = self.cur_room(level) {
			mon_room == rn
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
