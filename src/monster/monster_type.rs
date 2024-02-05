use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum MonsterType {
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

impl From<usize> for MonsterType {
	fn from(value: usize) -> Self { MonsterType::LIST[value] }
}

impl MonsterType {
	pub const LIST: [MonsterType; 26] = [
		MonsterType::Aquator, MonsterType::Bat, MonsterType::Centaur, MonsterType::Dragon, MonsterType::Emu, MonsterType::FlyTrap,
		MonsterType::Griffin, MonsterType::Hobgoblin, MonsterType::IceMonster, MonsterType::Jabberwock, MonsterType::Kestrel, MonsterType::Leprechaun,
		MonsterType::Medusa, MonsterType::Nymph, MonsterType::Orc, MonsterType::Phantom, MonsterType::Quagga, MonsterType::Rattlesnake,
		MonsterType::Snake, MonsterType::Troll, MonsterType::Unicorn, MonsterType::Vampire, MonsterType::Wraith, MonsterType::Xeroc,
		MonsterType::Yeti, MonsterType::Zombie,
	];
	pub fn name(&self) -> &'static str {
		match self {
			MonsterType::Aquator => "aquator",
			MonsterType::Bat => "bat",
			MonsterType::Centaur => "centaur",
			MonsterType::Dragon => "dragon",
			MonsterType::Emu => "emu",
			MonsterType::FlyTrap => "venus fly-trap",
			MonsterType::Griffin => "griffin",
			MonsterType::Hobgoblin => "hobgoblin",
			MonsterType::IceMonster => "ice monster",
			MonsterType::Jabberwock => "jabberwock",
			MonsterType::Kestrel => "kestrel",
			MonsterType::Leprechaun => "leprechaun",
			MonsterType::Medusa => "medusa",
			MonsterType::Nymph => "nymph",
			MonsterType::Orc => "orc",
			MonsterType::Phantom => "phantom",
			MonsterType::Quagga => "quagga",
			MonsterType::Rattlesnake => "rattlesnake",
			MonsterType::Snake => "snake",
			MonsterType::Troll => "troll",
			MonsterType::Unicorn => "black unicorn",
			MonsterType::Vampire => "vampire",
			MonsterType::Wraith => "wraith",
			MonsterType::Xeroc => "xeroc",
			MonsterType::Yeti => "yeti",
			MonsterType::Zombie => "zombie",
		}
	}
}
