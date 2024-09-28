use crate::init::GameState;
use crate::monster::{Monster, MonsterFlags, MonsterIndex};
use crate::resources::level::size::LevelSpot;
use crate::resources::play::state::RunState;

pub trait Arena {
	fn rogue_row(&self) -> i64;
	fn rogue_col(&self) -> i64;
	fn rogue_is_near(&self, row: i64, col: i64) -> bool {
		let (rogue_row, rogue_col) = (self.rogue_row(), self.rogue_col());
		let (near_rows, near_cols) = ((row - 1)..=(row + 1), (col - 1)..=(col + 1));
		near_rows.contains(&rogue_row) && near_cols.contains(&rogue_col)
	}

	fn set_monster_spot(&mut self, mon_id: MonsterIndex, row: i64, col: i64);

	fn as_monster(&self, mon_id: MonsterIndex) -> &Monster;
	fn as_monster_mut(&mut self, mon_id: MonsterIndex) -> &mut Monster;

	fn as_monster_flags(&self, mon_id: MonsterIndex) -> &MonsterFlags;
	fn as_monster_flags_mut(&mut self, mon_id: MonsterIndex) -> &mut MonsterFlags;
}

impl Arena for GameState {
	fn rogue_row(&self) -> i64 { self.player.rogue.row }
	fn rogue_col(&self) -> i64 { self.player.rogue.col }

	fn set_monster_spot(&mut self, mon_id: MonsterIndex, row: i64, col: i64) {
		let monster = self.as_monster_mut(mon_id);
		monster.spot.row = row;
		monster.spot.col = col;
	}

	fn as_monster(&self, mon_id: u64) -> &Monster { &self.mash.monster(mon_id) }
	fn as_monster_mut(&mut self, mon_id: u64) -> &mut Monster { self.mash.monster_mut(mon_id) }

	fn as_monster_flags(&self, mon_id: MonsterIndex) -> &MonsterFlags {
		self.mash.monster_flags(mon_id)
	}
	fn as_monster_flags_mut(&mut self, mon_id: MonsterIndex) -> &mut MonsterFlags {
		self.mash.monster_flags_mut(mon_id)
	}
}

impl Arena for RunState {
	fn rogue_row(&self) -> i64 {
		self.level.rogue.spot.as_spot().row.i64()
	}

	fn rogue_col(&self) -> i64 {
		self.level.rogue.spot.as_spot().col.i64()
	}

	fn set_monster_spot(&mut self, mon_id: MonsterIndex, row: i64, col: i64) {
		let new_spot = LevelSpot::from_i64(row, col);
		if let Some(monster_spot) = self.level.find_monster(mon_id) {
			if new_spot != monster_spot {
				let monster = self.level.take_monster(monster_spot).unwrap();
				self.level.put_monster(new_spot, monster)
			}
		}
	}

	fn as_monster(&self, mon_id: u64) -> &Monster {
		let spot = self.level.find_monster(mon_id).unwrap();
		self.level.as_monster(spot)
	}

	fn as_monster_mut(&mut self, mon_id: u64) -> &mut Monster {
		let spot = self.level.find_monster(mon_id).unwrap();
		self.level.as_monster_mut(spot)
	}

	fn as_monster_flags(&self, mon_id: MonsterIndex) -> &MonsterFlags {
		&self.as_monster(mon_id).m_flags
	}

	fn as_monster_flags_mut(&mut self, mon_id: MonsterIndex) -> &mut MonsterFlags {
		&mut self.as_monster_mut(mon_id).m_flags
	}
}