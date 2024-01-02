use rand::{Rng, thread_rng};
use crate::objects::object;
use crate::prelude::object_what::ObjectWhat::Weapon;

pub enum DamageEffect {
	Roll,
	None,
}

#[derive(Copy, Clone)]
pub struct DamageStat {
	pub hits: usize,
	pub damage: usize,
}

impl DamageStat {
	pub fn parse(damage_str: &str) -> Vec<Self> {
		let mut stats = Vec::new();
		let weapons = damage_str.split('/');
		for weapon in weapons {
			if let Some(d_pos) = weapon.find('d') {
				let (hits, damage) = weapon.split_at(d_pos);
				let hits = hits.parse::<usize>().expect("parse strikes");
				let damage = damage[1..].parse::<usize>().expect("parse damage");
				stats.push(DamageStat { hits, damage });
			}
		}
		stats
	}
	pub fn parse_first(damage_str: &str) -> Self {
		Self::parse(damage_str).first().cloned().expect("damage stat")
	}
}

pub fn get_damage(damage_str: &str, effect: DamageEffect) -> isize {
	let mut total = 0;
	for DamageStat { hits, damage } in DamageStat::parse(damage_str) {
		for _ in 0..hits {
			let damage = match effect {
				DamageEffect::Roll => {
					// Replicate the original (perhaps buggy) damage calculation
					match damage {
						0 => thread_rng().gen_range(0..=1),
						1 => 1,
						_ => thread_rng().gen_range(1..=damage),
					}
				}
				DamageEffect::None => damage,
			};
			total += damage;
		}
	}
	return total as isize;
}

pub fn get_w_damage(obj: &object) -> Option<isize> {
	if obj.what_is != Weapon {
		return None;
	}
	let first = DamageStat::parse(&obj.damage).first().expect("base damage stats for weapon");
	let DamageStat { hits, damage } = first;
	let hits = *hits + obj.hit_enchant as usize;
	let damage = *damage + obj.d_enchant as usize;
	let new_stat = format!("{}d{}", hits, damage);
	Some(get_damage(new_stat.as_str(), DamageEffect::Roll))
}

