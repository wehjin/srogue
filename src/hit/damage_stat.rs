use rand::{Rng, thread_rng};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DamageEffect { Roll, None }

impl DamageEffect {
	pub fn apply(&self, damage: usize) -> usize {
		match self {
			DamageEffect::Roll => match damage {
				0 => thread_rng().gen_range(0..=1),
				1 => 1,
				_ => thread_rng().gen_range(1..=damage),
			},
			DamageEffect::None => damage,
		}
	}
}

#[derive(Copy, Clone)]
pub struct DamageStat {
	pub hits: usize,
	pub damage: usize,
}

impl DamageStat {
	pub fn roll_damage(&self, effect: DamageEffect) -> usize {
		let mut total = 0;
		for _hn in 0..self.hits {
			total += effect.apply(self.damage);
		}
		total
	}
}

