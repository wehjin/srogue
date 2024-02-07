use std::collections::HashSet;

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
	pub fn union(flags: &Vec<SpotFlag>) -> u16 {
		flags.iter().fold(0, |it, more| it & more.code())
	}
	pub fn is_any_set(flags: &Vec<SpotFlag>, value: u16) -> bool {
		for flag in flags {
			if flag.is_set(value) {
				return true;
			}
		}
		return false;
	}

	pub fn is_nothing(value: u16) -> bool {
		value == 0
	}
	pub fn set_nothing(value: &mut u16) {
		*value = 0;
	}
	pub fn are_others_set(flags: &Vec<SpotFlag>, value: u16) -> bool {
		let all = vec![crate::prelude::SpotFlag::Object, crate::prelude::SpotFlag::Monster, crate::prelude::SpotFlag::Stairs, crate::prelude::SpotFlag::HorWall, crate::prelude::SpotFlag::VertWall, crate::prelude::SpotFlag::Door, crate::prelude::SpotFlag::Floor, crate::prelude::SpotFlag::Tunnel, crate::prelude::SpotFlag::Trap, crate::prelude::SpotFlag::Hidden];
		let all_set = all.into_iter().collect::<HashSet<_>>();
		let exclude_set = flags.iter().cloned().collect::<HashSet<_>>();
		let difference_set = all_set.difference(&exclude_set).cloned().collect::<Vec<_>>();
		SpotFlag::is_any_set(&difference_set, value)
	}

	pub fn is_set(&self, value: u16) -> bool {
		match self {
			SpotFlag::Nothing => value == 0,
			_ => (value & self.code()) != 0,
		}
	}
	pub fn is_only(&self, value: u16) -> bool {
		value == self.code()
	}
	pub fn clear(&self, value: &mut u16) {
		let code = self.code();
		*value &= !code;
	}
	pub fn set(&self, value: &mut u16) {
		let code = self.code();
		*value |= code;
	}
	pub fn code(&self) -> u16 {
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

