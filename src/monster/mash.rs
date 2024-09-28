use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::hit::DamageStat;
use crate::level::{DungeonCell, Level};
use crate::monster::{MonsterFlags, MonsterKind};
use crate::player::RoomMark;
use crate::prelude::DungeonSpot;
use crate::resources::level::setup::npc::disguise::Disguise;
use crate::resources::level::size::LevelSpot;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MonsterMash {
	pub monsters: HashMap<MonsterIndex, Monster>,
	pub m_moves: usize,
	pub mon_disappeared: bool,
}

pub type MonsterIndex = u64;

impl MonsterMash {
	pub fn clear(&mut self) {
		self.monsters.clear();
		self.m_moves = 0;
	}
}

impl MonsterMash {
	pub fn is_empty(&self) -> bool { self.monsters.is_empty() }
	pub fn monster_ids(&self) -> Vec<MonsterIndex> {
		self.monsters.keys().cloned().collect()
	}
	pub fn monsters(&self) -> Vec<&Monster> {
		self.monsters.values().collect::<Vec<_>>()
	}
	pub fn add_monster(&mut self, monster: Monster) {
		self.monsters.insert(monster.id(), monster);
	}
	pub fn remove_monster(&mut self, id: MonsterIndex) -> Monster {
		let removed = self.monsters.remove(&id).expect("monster with removal id");
		removed
	}
	pub fn monster_id_at_spot(&self, row: i64, col: i64) -> Option<MonsterIndex> {
		let spot = DungeonSpot { row, col };
		for (id, mon) in &self.monsters {
			if mon.spot == spot {
				return Some(*id);
			}
		}
		None
	}
	pub fn monster_at_spot(&self, row: i64, col: i64) -> Option<&Monster> {
		match self.monster_id_at_spot(row, col) {
			Some(id) => self.monsters.get(&id),
			None => None,
		}
	}
	pub fn monster_at_spot_mut(&mut self, row: i64, col: i64) -> Option<&mut Monster> {
		match self.monster_id_at_spot(row, col) {
			Some(id) => self.monsters.get_mut(&id),
			None => None,
		}
	}
	pub fn monster_flags(&self, id: MonsterIndex) -> &MonsterFlags {
		let monster = self.monster(id);
		let flags = &monster.m_flags;
		flags
	}
	pub fn monster_flags_mut(&mut self, id: MonsterIndex) -> &mut MonsterFlags {
		let monster = self.monster_mut(id);
		let flags = &mut monster.m_flags;
		flags
	}
	pub fn test_monster(&self, id: MonsterIndex, f: impl Fn(&Monster) -> bool) -> bool {
		let monster = self.monster(id);
		f(monster)
	}
	pub fn monster_to_spot(&self, id: MonsterIndex) -> DungeonSpot {
		self.monster(id).spot
	}
	pub fn try_monster(&self, id: MonsterIndex) -> Option<&Monster> {
		self.monsters.get(&id)
	}
	pub fn monster(&self, id: MonsterIndex) -> &Monster {
		&self.monsters[&id]
	}
	pub fn monster_mut(&mut self, id: MonsterIndex) -> &mut Monster {
		self.monsters.get_mut(&id).expect("id is in monsters")
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Monster {
	pub id: u64,
	pub kind: MonsterKind,
	pub m_flags: MonsterFlags,
	pub spot: DungeonSpot,
	pub disguise: Disguise,
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
	pub fn imitates(&self) -> bool { self.m_flags.imitates }
	pub fn flies(&self) -> bool { self.m_flags.flies }
	pub fn is_napping(&self) -> bool { self.m_flags.napping }
	pub fn is_asleep(&self) -> bool { self.m_flags.asleep }
	pub fn is_invisible(&self) -> bool { self.m_flags.invisible }
	pub fn is_hasted(&self) -> bool { self.m_flags.hasted }
	pub fn is_slowed(&self) -> bool { self.m_flags.slowed }
	pub fn is_confused(&self) -> bool { self.m_flags.confused }
	pub fn wanders(&self) -> bool { self.m_flags.wanders }
}

impl Monster {
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
	pub fn set_target(&mut self, spot: LevelSpot) {
		let (row, col) = spot.i64();
		self.target_spot = Some(DungeonSpot { row, col })
	}
	pub fn clear_target_reset_stuck(&mut self) {
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
	pub fn id(&self) -> u64 { self.id }
	pub fn kill_exp(&self) -> usize { self.kind.kill_exp() }
	pub fn m_hit_chance(&self) -> usize { self.kind.m_hit_chance() }
	pub fn m_damage(&self) -> &'static [DamageStat] {
		self.kind.damage()
	}
	pub fn wakens(&self) -> bool { self.m_flags.wakens }
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
	pub fn as_char(&self) -> char {
		self.kind.screen_char()
	}
	pub fn flip_slowed_toggle(&mut self) {
		if self.slowed_toggle {
			self.slowed_toggle = false;
		} else {
			self.slowed_toggle = true;
		}
	}
	pub fn is_slowed_toggle(&self) -> bool {
		self.slowed_toggle
	}
	pub fn wake_up(&mut self) {
		self.m_flags.wake_up();
	}
}
