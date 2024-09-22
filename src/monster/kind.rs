use serde::{Deserialize, Serialize};

use crate::hit::DamageStat;
use crate::monster::MonsterFlags;
use crate::prelude::AMULET_LEVEL;
use crate::random::get_rand;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MonsterKind {
	Aquator,
	Bat,
	Centaur,
	Dragon,
	Emu,
	FlyTrap,
	Griffin,
	Hobgoblin,
	IceMonster,
	Jabberwock,
	Kestrel,
	Leprechaun,
	Medusa,
	Nymph,
	Orc,
	Phantom,
	Quagga,
	Rattlesnake,
	Snake,
	Troll,
	Unicorn,
	Vampire,
	Wraith,
	Xeroc,
	Yeti,
	Zombie,
}

impl From<usize> for MonsterKind {
	fn from(value: usize) -> Self { MonsterKind::LIST[value] }
}

impl MonsterKind {
	pub fn random_any() -> Self {
		Self::LIST[get_rand(0, MONSTERS - 1)]
	}

	pub fn random_name() -> &'static str {
		let random_kind = MonsterKind::LIST[get_rand(0, MonsterKind::LIST.len() - 1)];
		random_kind.name()
	}
	pub fn index(&self) -> usize {
		match self {
			MonsterKind::Aquator => 0,
			MonsterKind::Bat => 1,
			MonsterKind::Centaur => 2,
			MonsterKind::Dragon => 3,
			MonsterKind::Emu => 4,
			MonsterKind::FlyTrap => 5,
			MonsterKind::Griffin => 6,
			MonsterKind::Hobgoblin => 7,
			MonsterKind::IceMonster => 8,
			MonsterKind::Jabberwock => 9,
			MonsterKind::Kestrel => 10,
			MonsterKind::Leprechaun => 11,
			MonsterKind::Medusa => 12,
			MonsterKind::Nymph => 13,
			MonsterKind::Orc => 14,
			MonsterKind::Phantom => 15,
			MonsterKind::Quagga => 16,
			MonsterKind::Rattlesnake => 17,
			MonsterKind::Snake => 18,
			MonsterKind::Troll => 19,
			MonsterKind::Unicorn => 20,
			MonsterKind::Vampire => 21,
			MonsterKind::Wraith => 22,
			MonsterKind::Xeroc => 23,
			MonsterKind::Yeti => 24,
			MonsterKind::Zombie => 25,
		}
	}
	pub fn name(&self) -> &'static str {
		match self {
			MonsterKind::Aquator => "aquator",
			MonsterKind::Bat => "bat",
			MonsterKind::Centaur => "centaur",
			MonsterKind::Dragon => "dragon",
			MonsterKind::Emu => "emu",
			MonsterKind::FlyTrap => "venus fly-trap",
			MonsterKind::Griffin => "griffin",
			MonsterKind::Hobgoblin => "hobgoblin",
			MonsterKind::IceMonster => "ice monster",
			MonsterKind::Jabberwock => "jabberwock",
			MonsterKind::Kestrel => "kestrel",
			MonsterKind::Leprechaun => "leprechaun",
			MonsterKind::Medusa => "medusa",
			MonsterKind::Nymph => "nymph",
			MonsterKind::Orc => "orc",
			MonsterKind::Phantom => "phantom",
			MonsterKind::Quagga => "quagga",
			MonsterKind::Rattlesnake => "rattlesnake",
			MonsterKind::Snake => "snake",
			MonsterKind::Troll => "troll",
			MonsterKind::Unicorn => "black unicorn",
			MonsterKind::Vampire => "vampire",
			MonsterKind::Wraith => "wraith",
			MonsterKind::Xeroc => "xeroc",
			MonsterKind::Yeti => "yeti",
			MonsterKind::Zombie => "zombie",
		}
	}
	pub const LIST: [MonsterKind; 26] = [
		MonsterKind::Aquator, MonsterKind::Bat, MonsterKind::Centaur, MonsterKind::Dragon, MonsterKind::Emu, MonsterKind::FlyTrap,
		MonsterKind::Griffin, MonsterKind::Hobgoblin, MonsterKind::IceMonster, MonsterKind::Jabberwock, MonsterKind::Kestrel, MonsterKind::Leprechaun,
		MonsterKind::Medusa, MonsterKind::Nymph, MonsterKind::Orc, MonsterKind::Phantom, MonsterKind::Quagga, MonsterKind::Rattlesnake,
		MonsterKind::Snake, MonsterKind::Troll, MonsterKind::Unicorn, MonsterKind::Vampire, MonsterKind::Wraith, MonsterKind::Xeroc,
		MonsterKind::Yeti, MonsterKind::Zombie,
	];

	pub fn depth_adjusted_flags(&self, depth: usize) -> MonsterFlags {
		let mut flags = self.flags();
		if depth > (AMULET_LEVEL + 2) as usize {
			flags.hasted = true;
		}
		flags
	}

	fn flags(&self) -> MonsterFlags {
		match self {
			MonsterKind::Aquator => MonsterFlags::a(),
			MonsterKind::Bat => MonsterFlags::b(),
			MonsterKind::Centaur => MonsterFlags::c(),
			MonsterKind::Dragon => MonsterFlags::d(),
			MonsterKind::Emu => MonsterFlags::e(),
			MonsterKind::FlyTrap => MonsterFlags::f(),
			MonsterKind::Griffin => MonsterFlags::g(),
			MonsterKind::Hobgoblin => MonsterFlags::h(),
			MonsterKind::IceMonster => MonsterFlags::i(),
			MonsterKind::Jabberwock => MonsterFlags::j(),
			MonsterKind::Kestrel => MonsterFlags::k(),
			MonsterKind::Leprechaun => MonsterFlags::l(),
			MonsterKind::Medusa => MonsterFlags::m(),
			MonsterKind::Nymph => MonsterFlags::n(),
			MonsterKind::Orc => MonsterFlags::o(),
			MonsterKind::Phantom => MonsterFlags::p(),
			MonsterKind::Quagga => MonsterFlags::q(),
			MonsterKind::Rattlesnake => MonsterFlags::r(),
			MonsterKind::Snake => MonsterFlags::s(),
			MonsterKind::Troll => MonsterFlags::t(),
			MonsterKind::Unicorn => MonsterFlags::u(),
			MonsterKind::Vampire => MonsterFlags::v(),
			MonsterKind::Wraith => MonsterFlags::w(),
			MonsterKind::Xeroc => MonsterFlags::x(),
			MonsterKind::Yeti => MonsterFlags::y(),
			MonsterKind::Zombie => MonsterFlags::z(),
		}
	}
	pub fn damage(&self) -> &'static [DamageStat] {
		match self {
			MonsterKind::Aquator => &[DamageStat { hits: 0, damage: 0 }],
			MonsterKind::Bat => &[DamageStat { hits: 1, damage: 3 }],
			MonsterKind::Centaur => &[DamageStat { hits: 3, damage: 3 }, DamageStat { hits: 2, damage: 5 }],
			MonsterKind::Dragon => &[DamageStat { hits: 4, damage: 6 }, DamageStat { hits: 4, damage: 9 }],
			MonsterKind::Emu => &[DamageStat { hits: 1, damage: 3 }],
			MonsterKind::FlyTrap => &[DamageStat { hits: 5, damage: 5 }],
			MonsterKind::Griffin => &[DamageStat { hits: 5, damage: 5 }, DamageStat { hits: 5, damage: 5 }],
			MonsterKind::Hobgoblin => &[DamageStat { hits: 1, damage: 3 }, DamageStat { hits: 1, damage: 2 }],
			MonsterKind::IceMonster => &[DamageStat { hits: 0, damage: 0 }],
			MonsterKind::Jabberwock => &[DamageStat { hits: 3, damage: 10 }, DamageStat { hits: 4, damage: 5 }],
			MonsterKind::Kestrel => &[DamageStat { hits: 1, damage: 4 }],
			MonsterKind::Leprechaun => &[DamageStat { hits: 0, damage: 0 }],
			MonsterKind::Medusa => &[DamageStat { hits: 4, damage: 4 }, DamageStat { hits: 3, damage: 7 }],
			MonsterKind::Nymph => &[DamageStat { hits: 0, damage: 0 }],
			MonsterKind::Orc => &[DamageStat { hits: 1, damage: 6 }],
			MonsterKind::Phantom => &[DamageStat { hits: 5, damage: 4 }],
			MonsterKind::Quagga => &[DamageStat { hits: 3, damage: 5 }],
			MonsterKind::Rattlesnake => &[DamageStat { hits: 2, damage: 5 }],
			MonsterKind::Snake => &[DamageStat { hits: 1, damage: 3 }],
			MonsterKind::Troll => &[DamageStat { hits: 4, damage: 6 }, DamageStat { hits: 1, damage: 4 }],
			MonsterKind::Unicorn => &[DamageStat { hits: 4, damage: 10 }],
			MonsterKind::Vampire => &[DamageStat { hits: 1, damage: 14 }, DamageStat { hits: 1, damage: 4 }],
			MonsterKind::Wraith => &[DamageStat { hits: 2, damage: 8 }],
			MonsterKind::Xeroc => &[DamageStat { hits: 4, damage: 6 }],
			MonsterKind::Yeti => &[DamageStat { hits: 3, damage: 6 }],
			MonsterKind::Zombie => &[DamageStat { hits: 1, damage: 7 }],
		}
	}
	pub fn hp_to_kill(&self) -> isize {
		match self {
			MonsterKind::Aquator => 25,
			MonsterKind::Bat => 10,
			MonsterKind::Centaur => 32,
			MonsterKind::Dragon => 145,
			MonsterKind::Emu => 11,
			MonsterKind::FlyTrap => 73,
			MonsterKind::Griffin => 115,
			MonsterKind::Hobgoblin => 15,
			MonsterKind::IceMonster => 15,
			MonsterKind::Jabberwock => 132,
			MonsterKind::Kestrel => 10,
			MonsterKind::Leprechaun => 25,
			MonsterKind::Medusa => 97,
			MonsterKind::Nymph => 25,
			MonsterKind::Orc => 25,
			MonsterKind::Phantom => 76,
			MonsterKind::Quagga => 30,
			MonsterKind::Rattlesnake => 19,
			MonsterKind::Snake => 8,
			MonsterKind::Troll => 75,
			MonsterKind::Unicorn => 90,
			MonsterKind::Vampire => 55,
			MonsterKind::Wraith => 45,
			MonsterKind::Xeroc => 42,
			MonsterKind::Yeti => 35,
			MonsterKind::Zombie => 21,
		}
	}
	pub fn screen_char(&self) -> char {
		match self {
			MonsterKind::Aquator => 'A',
			MonsterKind::Bat => 'B',
			MonsterKind::Centaur => 'C',
			MonsterKind::Dragon => 'D',
			MonsterKind::Emu => 'E',
			MonsterKind::FlyTrap => 'F',
			MonsterKind::Griffin => 'G',
			MonsterKind::Hobgoblin => 'H',
			MonsterKind::IceMonster => 'I',
			MonsterKind::Jabberwock => 'J',
			MonsterKind::Kestrel => 'K',
			MonsterKind::Leprechaun => 'L',
			MonsterKind::Medusa => 'M',
			MonsterKind::Nymph => 'N',
			MonsterKind::Orc => 'O',
			MonsterKind::Phantom => 'P',
			MonsterKind::Quagga => 'Q',
			MonsterKind::Rattlesnake => 'R',
			MonsterKind::Snake => 'S',
			MonsterKind::Troll => 'T',
			MonsterKind::Unicorn => 'U',
			MonsterKind::Vampire => 'V',
			MonsterKind::Wraith => 'W',
			MonsterKind::Xeroc => 'X',
			MonsterKind::Yeti => 'Y',
			MonsterKind::Zombie => 'Z',
		}
	}
	pub fn kill_exp(&self) -> isize {
		match self {
			MonsterKind::Aquator => 20,
			MonsterKind::Bat => 2,
			MonsterKind::Centaur => 15,
			MonsterKind::Dragon => 5000,
			MonsterKind::Emu => 2,
			MonsterKind::FlyTrap => 91,
			MonsterKind::Griffin => 2000,
			MonsterKind::Hobgoblin => 3,
			MonsterKind::IceMonster => 5,
			MonsterKind::Jabberwock => 3000,
			MonsterKind::Kestrel => 2,
			MonsterKind::Leprechaun => 21,
			MonsterKind::Medusa => 28,
			MonsterKind::Nymph => 39,
			MonsterKind::Orc => 5,
			MonsterKind::Phantom => 120,
			MonsterKind::Quagga => 20,
			MonsterKind::Rattlesnake => 10,
			MonsterKind::Snake => 2,
			MonsterKind::Troll => 125,
			MonsterKind::Unicorn => 27,
			MonsterKind::Vampire => 39,
			MonsterKind::Wraith => 55,
			MonsterKind::Xeroc => 110,
			MonsterKind::Yeti => 50,
			MonsterKind::Zombie => 8,
		}
	}
	pub fn first_level(&self) -> usize {
		match self {
			MonsterKind::Aquator => 9,
			MonsterKind::Bat => 1,
			MonsterKind::Centaur => 7,
			MonsterKind::Dragon => 21,
			MonsterKind::Emu => 1,
			MonsterKind::FlyTrap => 12,
			MonsterKind::Griffin => 20,
			MonsterKind::Hobgoblin => 1,
			MonsterKind::IceMonster => 2,
			MonsterKind::Jabberwock => 21,
			MonsterKind::Kestrel => 1,
			MonsterKind::Leprechaun => 6,
			MonsterKind::Medusa => 126,
			MonsterKind::Nymph => 10,
			MonsterKind::Orc => 4,
			MonsterKind::Phantom => 15,
			MonsterKind::Quagga => 8,
			MonsterKind::Rattlesnake => 3,
			MonsterKind::Snake => 1,
			MonsterKind::Troll => 13,
			MonsterKind::Unicorn => 26,
			MonsterKind::Vampire => 126,
			MonsterKind::Wraith => 14,
			MonsterKind::Xeroc => 16,
			MonsterKind::Yeti => 11,
			MonsterKind::Zombie => 5,
		}
	}
	pub fn last_level(&self) -> usize {
		match self {
			MonsterKind::Aquator => 18,
			MonsterKind::Bat => 8,
			MonsterKind::Centaur => 16,
			MonsterKind::Dragon => 126,
			MonsterKind::Emu => 7,
			MonsterKind::FlyTrap => 126,
			MonsterKind::Griffin => 126,
			MonsterKind::Hobgoblin => 10,
			MonsterKind::IceMonster => 11,
			MonsterKind::Jabberwock => 126,
			MonsterKind::Kestrel => 6,
			MonsterKind::Leprechaun => 16,
			MonsterKind::Medusa => 85,
			MonsterKind::Nymph => 19,
			MonsterKind::Orc => 13,
			MonsterKind::Phantom => 24,
			MonsterKind::Quagga => 17,
			MonsterKind::Rattlesnake => 12,
			MonsterKind::Snake => 9,
			MonsterKind::Troll => 22,
			MonsterKind::Unicorn => 85,
			MonsterKind::Vampire => 85,
			MonsterKind::Wraith => 23,
			MonsterKind::Xeroc => 25,
			MonsterKind::Yeti => 20,
			MonsterKind::Zombie => 14,
		}
	}
	pub fn m_hit_chance(&self) -> usize {
		match self {
			MonsterKind::Aquator => 100,
			MonsterKind::Bat => 60,
			MonsterKind::Centaur => 85,
			MonsterKind::Dragon => 100,
			MonsterKind::Emu => 65,
			MonsterKind::FlyTrap => 80,
			MonsterKind::Griffin => 85,
			MonsterKind::Hobgoblin => 67,
			MonsterKind::IceMonster => 68,
			MonsterKind::Jabberwock => 100,
			MonsterKind::Kestrel => 60,
			MonsterKind::Leprechaun => 75,
			MonsterKind::Medusa => 85,
			MonsterKind::Nymph => 75,
			MonsterKind::Orc => 70,
			MonsterKind::Phantom => 80,
			MonsterKind::Quagga => 78,
			MonsterKind::Rattlesnake => 70,
			MonsterKind::Snake => 50,
			MonsterKind::Troll => 75,
			MonsterKind::Unicorn => 85,
			MonsterKind::Vampire => 85,
			MonsterKind::Wraith => 75,
			MonsterKind::Xeroc => 75,
			MonsterKind::Yeti => 80,
			MonsterKind::Zombie => 69,
		}
	}
	pub fn drop_percent(&self) -> usize {
		match self {
			MonsterKind::Aquator => 0,
			MonsterKind::Bat => 0,
			MonsterKind::Centaur => 10,
			MonsterKind::Dragon => 90,
			MonsterKind::Emu => 0,
			MonsterKind::FlyTrap => 0,
			MonsterKind::Griffin => 10,
			MonsterKind::Hobgoblin => 0,
			MonsterKind::IceMonster => 0,
			MonsterKind::Jabberwock => 0,
			MonsterKind::Kestrel => 0,
			MonsterKind::Leprechaun => 0,
			MonsterKind::Medusa => 25,
			MonsterKind::Nymph => 100,
			MonsterKind::Orc => 10,
			MonsterKind::Phantom => 50,
			MonsterKind::Quagga => 20,
			MonsterKind::Rattlesnake => 0,
			MonsterKind::Snake => 0,
			MonsterKind::Troll => 33,
			MonsterKind::Unicorn => 33,
			MonsterKind::Vampire => 18,
			MonsterKind::Wraith => 0,
			MonsterKind::Xeroc => 0,
			MonsterKind::Yeti => 20,
			MonsterKind::Zombie => 0,
		}
	}
}

pub const MONSTERS: usize = 26;
