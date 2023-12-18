#[derive(Copy, Clone, Default)]
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
	pub RUST_VANISHED: bool,
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

	pub fn a() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, rusts: true, ..Default::default() } }
	pub fn b() -> Self { MonsterFlags { asleep: true, wanders: true, flits: true, ..Default::default() } }
	pub fn c() -> Self { MonsterFlags { asleep: true, wanders: true, ..Default::default() } }
	pub fn d() -> Self { MonsterFlags { asleep: true, wakens: true, flames: true, ..Default::default() } }
	pub fn e() -> Self { MonsterFlags { asleep: true, wakens: true, ..Default::default() } }
	pub fn f() -> Self { MonsterFlags { holds: true, stationary: true, ..Default::default() } }
	pub fn g() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, flies: true, ..Default::default() } }
	pub fn h() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Default::default() } }
	pub fn i() -> Self { MonsterFlags { asleep: true, freezes: true, ..Default::default() } }
	pub fn j() -> Self { MonsterFlags { asleep: true, wanders: true, ..Default::default() } }
	pub fn k() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, flies: true, ..Default::default() } }
	pub fn l() -> Self { MonsterFlags { asleep: true, steals_gold: true, ..Default::default() } }
	pub fn m() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, confuses: true, ..Default::default() } }
	pub fn n() -> Self { MonsterFlags { asleep: true, steals_item: true, ..Default::default() } }
	pub fn o() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, seeks_gold: true, ..Default::default() } }
	pub fn p() -> Self { MonsterFlags { asleep: true, invisible: true, wanders: true, flits: true, ..Default::default() } }
	pub fn q() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Default::default() } }
	pub fn r() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, stings: true, ..Default::default() } }
	pub fn s() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Default::default() } }
	pub fn t() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Default::default() } }
	pub fn u() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Default::default() } }
	pub fn v() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, drains_life: true, ..Default::default() } }
	pub fn w() -> Self { MonsterFlags { asleep: true, wanders: true, drops_level: true, ..Default::default() } }
	pub fn x() -> Self { MonsterFlags { asleep: true, imitates: true, ..Default::default() } }
	pub fn y() -> Self { MonsterFlags { asleep: true, wanders: true, ..Default::default() } }
	pub fn z() -> Self { MonsterFlags { asleep: true, wakens: true, wanders: true, ..Default::default() } }
}

pub const MONSTERS: usize = 26;
pub const HASTED: u64 = 0o1;
pub const SLOWED: u64 = 0o2;
pub const INVISIBLE: u64 = 0o4;
pub const ASLEEP: u64 = 0o10;
pub const WAKENS: u64 = 0o20;
pub const WANDERS: u64 = 0o40;
pub const FLIES: u64 = 0o100;
pub const FLITS: u64 = 0o200;
pub const CAN_FLIT: u64 = 0o400;
pub const CONFUSED: u64 = 0o1000;
pub const RUSTS: u64 = 0o2000;
pub const HOLDS: u64 = 0o4000;
pub const FREEZES: u64 = 0o10000;
pub const STEALS_GOLD: u64 = 0o20000;
pub const STEALS_ITEM: u64 = 0o40000;
pub const STINGS: u64 = 0o100000;
pub const DRAINS_LIFE: u64 = 0o200000;
pub const DROPS_LEVEL: u64 = 0o400000;
pub const SEEKS_GOLD: u64 = 0o1000000;
pub const FREEZING_ROGUE: u64 = 0o2000000;
pub const RUST_VANISHED: u64 = 0o4000000;
pub const CONFUSES: u64 = 0o10000000;
pub const IMITATES: u64 = 0o20000000;
pub const FLAMES: u64 = 0o40000000;
pub const STATIONARY: u64 = 0o100000000;
pub const NAPPING: u64 = 0o200000000;
pub const ALREADY_MOVED: u64 = 0o400000000;
