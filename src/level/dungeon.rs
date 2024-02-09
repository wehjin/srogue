use std::ops::{Index, IndexMut};
use serde::{Deserialize, Serialize};
use crate::level::constants::{DCOLS, DROWS, MAX_ROOM, MAX_TRAP};
use crate::level::DungeonCell;
use crate::room::Room;
use crate::trap::Trap;

const SERIALIZE_MAX: usize = 32;

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct DungeonRow {
	cols0_32: [DungeonCell; SERIALIZE_MAX],
	cols32_64: [DungeonCell; SERIALIZE_MAX],
	cols64_DCOLS: [DungeonCell; DCOLS % SERIALIZE_MAX],
}

impl Index<usize> for DungeonRow {
	type Output = DungeonCell;
	fn index(&self, index: usize) -> &Self::Output {
		match index / SERIALIZE_MAX {
			0 => &self.cols0_32[index % SERIALIZE_MAX],
			1 => &self.cols32_64[index % SERIALIZE_MAX],
			2 => &self.cols64_DCOLS[index % SERIALIZE_MAX],
			_ => unimplemented!("DROWS greater that 96")
		}
	}
}

impl IndexMut<usize> for DungeonRow {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index / SERIALIZE_MAX {
			0 => &mut self.cols0_32[index % SERIALIZE_MAX],
			1 => &mut self.cols32_64[index % SERIALIZE_MAX],
			2 => &mut self.cols64_DCOLS[index % SERIALIZE_MAX],
			_ => unimplemented!("DROWS greater that 96")
		}
	}
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Level {
	pub rooms: [Room; MAX_ROOM],
	pub traps: [Trap; MAX_TRAP],
	pub dungeon: [DungeonRow; DROWS],
}
