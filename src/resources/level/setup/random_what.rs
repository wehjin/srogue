use rand::Rng;

pub enum RandomWhat {
	Scroll,
	Potion,
	Wand,
	Weapon,
	Armor,
	Food,
	Ring,
}
impl RandomWhat {
	pub fn roll(rng: &mut impl Rng) -> Self {
		let roll = rng.gen_range(1usize..=91);
		match roll {
			1..=30 => Self::Scroll,
			31..=60 => Self::Potion,
			61..=64 => Self::Wand,
			65..=74 => Self::Weapon,
			75..=83 => Self::Armor,
			84..=88 => Self::Food,
			_ => Self::Ring,
		}
	}
}
