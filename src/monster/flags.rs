use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct MonsterFlags {
	pub hasted: bool,
	pub slowed: bool,
	pub invisible: bool,
	pub asleep: bool,
	pub wakens: bool,
	pub wanders: bool,
	pub flies: bool,
	pub flits: bool,
	pub can_flit: bool,
	pub confused: bool,
	pub rusts: bool,
	pub holds: bool,
	pub freezes: bool,
	pub steals_gold: bool,
	pub steals_item: bool,
	pub stings: bool,
	pub drains_life: bool,
	pub drops_level: bool,
	pub seeks_gold: bool,
	pub freezing_rogue: bool,
	pub rust_vanished: bool,
	pub confuses: bool,
	pub imitates: bool,
	pub flames: bool,
	pub stationary: bool,
	pub napping: bool,
	pub already_moved: bool,
}

impl MonsterFlags {
	pub fn wake_up(&mut self) {
		if !self.napping {
			self.asleep = false;
			self.imitates = false;
			self.wakens = false;
		}
	}

	pub fn special_hit(&self) -> bool {
		self.rusts || self.holds || self.freezes || self.steals_gold || self.steals_item || self.stings || self.drains_life || self.drops_level
	}
	pub fn set_special_hit(&mut self, value: bool) {
		self.rusts = value;
		self.holds = value;
		self.freezes = value;
		self.steals_gold = value;
		self.steals_item = value;
		self.stings = value;
		self.drains_life = value;
		self.drops_level = value;
	}

	pub const fn a() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, rusts: true, ..Self::empty() } }
	pub const fn b() -> Self { MonsterFlags { asleep: true, wanders: true, flits: true, ..Self::empty() } }
	pub const fn c() -> Self { MonsterFlags { asleep: true, wanders: true, ..Self::empty() } }
	pub const fn d() -> Self { MonsterFlags { asleep: true, wakens: true, flames: true, ..Self::empty() } }
	pub const fn e() -> Self { MonsterFlags { asleep: true, wakens: true, ..Self::empty() } }
	pub const fn f() -> Self { MonsterFlags { holds: true, stationary: true, ..Self::empty() } }
	pub const fn g() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, flies: true, ..Self::empty() } }
	pub const fn h() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Self::empty() } }
	pub const fn i() -> Self { MonsterFlags { asleep: true, freezes: true, ..Self::empty() } }
	pub const fn j() -> Self { MonsterFlags { asleep: true, wanders: true, ..Self::empty() } }
	pub const fn k() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, flies: true, ..Self::empty() } }
	pub const fn l() -> Self { MonsterFlags { asleep: true, steals_gold: true, ..Self::empty() } }
	pub const fn m() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, confuses: true, ..Self::empty() } }
	pub const fn n() -> Self { MonsterFlags { asleep: true, steals_item: true, ..Self::empty() } }
	pub const fn o() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, seeks_gold: true, ..Self::empty() } }
	pub const fn p() -> Self { MonsterFlags { asleep: true, invisible: true, wanders: true, flits: true, ..Self::empty() } }
	pub const fn q() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Self::empty() } }
	pub const fn r() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, stings: true, ..Self::empty() } }
	pub const fn s() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Self::empty() } }
	pub const fn t() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Self::empty() } }
	pub const fn u() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Self::empty() } }
	pub const fn v() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, drains_life: true, ..Self::empty() } }
	pub const fn w() -> Self { MonsterFlags { asleep: true, wanders: true, drops_level: true, ..Self::empty() } }
	pub const fn x() -> Self { MonsterFlags { asleep: true, imitates: true, ..Self::empty() } }
	pub const fn y() -> Self { MonsterFlags { asleep: true, wanders: true, ..Self::empty() } }
	pub const fn z() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Self::empty() } }

	pub const fn empty() -> Self {
		MonsterFlags {
			hasted: false,
			slowed: false,
			invisible: false,
			asleep: false,
			wakens: false,
			wanders: false,
			flies: false,
			flits: false,
			can_flit: false,
			confused: false,
			rusts: false,
			holds: false,
			freezes: false,
			steals_gold: false,
			steals_item: false,
			stings: false,
			drains_life: false,
			drops_level: false,
			seeks_gold: false,
			freezing_rogue: false,
			rust_vanished: false,
			confuses: false,
			imitates: false,
			flames: false,
			stationary: false,
			napping: false,
			already_moved: false,
		}
	}
}

