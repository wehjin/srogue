use std::ops::{Index, IndexMut};
use serde::{Deserialize, Serialize};
use crate::level::constants::{DCOLS};
use crate::level::DungeonCell;

const SERIALIZE_MAX: usize = 32;

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct DungeonRow {
	cols0_32: [DungeonCell; SERIALIZE_MAX],
	cols32_64: [DungeonCell; SERIALIZE_MAX],
	cols64_dcols: [DungeonCell; DCOLS % SERIALIZE_MAX],
}

impl Index<usize> for DungeonRow {
	type Output = DungeonCell;
	fn index(&self, index: usize) -> &Self::Output {
		match index / SERIALIZE_MAX {
			0 => &self.cols0_32[index % SERIALIZE_MAX],
			1 => &self.cols32_64[index % SERIALIZE_MAX],
			2 => &self.cols64_dcols[index % SERIALIZE_MAX],
			_ => unimplemented!("DROWS greater that 96")
		}
	}
}

impl IndexMut<usize> for DungeonRow {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index / SERIALIZE_MAX {
			0 => &mut self.cols0_32[index % SERIALIZE_MAX],
			1 => &mut self.cols32_64[index % SERIALIZE_MAX],
			2 => &mut self.cols64_dcols[index % SERIALIZE_MAX],
			_ => unimplemented!("DROWS greater that 96")
		}
	}
}
