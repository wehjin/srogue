use crate::init::GameState;
use crate::prelude::DungeonSpot;

impl GameState {
	pub fn has_imitating_monster_at_spot(&self, spot: DungeonSpot) -> bool {
		match self.mash.monster_at_spot(spot.row, spot.col) {
			None => false,
			Some(monster) if monster.m_flags.imitates => true,
			Some(_) => false,
		}
	}
}

