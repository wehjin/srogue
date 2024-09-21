use crate::random::get_rand;

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
	pub fn roll() -> Self {
		if get_rand(1, 91) <= 30 {
			Self::Scroll
		} else if get_rand(1, 91) <= 60 {
			Self::Potion
		} else if get_rand(1, 91) <= 64 {
			Self::Wand
		} else if get_rand(1, 91) <= 74 {
			Self::Weapon
		} else if get_rand(1, 91) <= 83 {
			Self::Armor
		} else if get_rand(1, 91) <= 88 {
			Self::Food
		} else {
			Self::Ring
		}
	}
}
