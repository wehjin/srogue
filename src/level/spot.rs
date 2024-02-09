use crate::prelude::DungeonCell;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum SpotFlag {
	Nothing = 0x0,
	Object = 0o1,
	Monster = 0o2,
	Stairs = 0o4,
	HorWall = 0o10,
	VertWall = 0o20,
	Door = 0o40,
	Floor = 0o100,
	Tunnel = 0o200,
	Trap = 0o400,
	Hidden = 0o1000,
}

impl SpotFlag {
	pub fn is_set(&self, value: u16) -> bool {
		match self {
			SpotFlag::Nothing => value == 0,
			_ => (value & self.as_u16()) != 0,
		}
	}
	pub fn as_u16(&self) -> u16 {
		match self {
			SpotFlag::Nothing => 0o0,
			SpotFlag::Object => 0o1,
			SpotFlag::Monster => 0o2,
			SpotFlag::Stairs => 0o4,
			SpotFlag::HorWall => 0o10,
			SpotFlag::VertWall => 0o20,
			SpotFlag::Door => 0o40,
			SpotFlag::Floor => 0o100,
			SpotFlag::Tunnel => 0o200,
			SpotFlag::Trap => 0o400,
			SpotFlag::Hidden => 0o1000,
		}
	}
}

