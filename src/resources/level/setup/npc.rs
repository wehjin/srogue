use crate::monster::{Monster, MonsterKind, StuckCounter};
use crate::prelude::DungeonSpot;
use crate::random::{coin_toss, get_rand};
use crate::resources::level::setup::npc::disguise::{roll_disguise, Disguise};
use crate::resources::level::DungeonLevel;
use rand::{thread_rng, RngCore};

pub fn roll_monsters(level: &mut DungeonLevel) {
	let depth = level.depth;
	let count = roll_monster_count();
	for _ in 0..count {
		let mut monster = roll_monster(depth, 0);
		if monster.wanders() && coin_toss() {
			monster.wake_up();
		}
		let spot = level.roll_vacant_spot(true, false, true);
		level.put_monster(spot, monster);
	}
}

fn roll_monster_count() -> usize {
	get_rand(4, 6)
}

pub fn roll_monster(depth: usize, first_level_boost: usize) -> Monster {
	let kind = roll_monster_kind(depth, first_level_boost);
	let flags = kind.depth_adjusted_flags(depth);
	let disguise = if flags.imitates { roll_disguise() } else { Disguise::None };
	Monster {
		id: thread_rng().next_u64(),
		kind: kind.clone(),
		m_flags: flags,
		spot: DungeonSpot::default(),
		disguise,
		slowed_toggle: false,
		target_spot: None,
		nap_length: 0,
		stuck_counter: StuckCounter::default(),
		moves_confused: 0,
		stationary_damage: 0,
		hp_to_kill: kind.hp_to_kill(),
		killed: false,
		drop_percent: kind.drop_percent(),
	}
}

fn roll_monster_kind(depth: usize, first_level_boost: usize) -> MonsterKind {
	loop {
		let kind = MonsterKind::random_any();
		let first_level = (kind.first_level() as isize - first_level_boost as isize).max(0) as usize;
		if depth >= first_level && depth <= kind.last_level() {
			return kind;
		}
	}
}

pub mod disguise {
	use crate::random::get_rand;
	use crate::render_system::{ARMOR_CHAR, FOOD_CHAR, GOLD_CHAR, NOT_CHAR, POTION_CHAR, RING_CHAR, SCROLL_CHAR, STAIRS_CHAR, WAND_CHAR, WEAPON_CHAR};
	use serde::{Deserialize, Serialize};

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub enum Disguise {
		None,
		Stairs,
		Potion,
		Scroll,
		Armor,
		Ring,
		Wand,
		Weapon,
		Food,
		Gold,
	}

	impl Disguise {
		pub fn char(&self) -> char {
			match self {
				Disguise::None => NOT_CHAR,
				Disguise::Stairs => STAIRS_CHAR,
				Disguise::Potion => POTION_CHAR,
				Disguise::Scroll => SCROLL_CHAR,
				Disguise::Armor => ARMOR_CHAR,
				Disguise::Ring => RING_CHAR,
				Disguise::Wand => WAND_CHAR,
				Disguise::Weapon => WEAPON_CHAR,
				Disguise::Food => FOOD_CHAR,
				Disguise::Gold => GOLD_CHAR,
			}
		}
	}

	pub fn roll_disguise() -> Disguise {
		let index = get_rand(1, 9);
		ALL_DISGUISE[index as usize]
	}
	const ALL_DISGUISE: [Disguise; 10] = [
		Disguise::None, Disguise::Stairs, Disguise::Potion, Disguise::Scroll, Disguise::Armor,
		Disguise::Ring, Disguise::Wand, Disguise::Weapon, Disguise::Food, Disguise::Gold,
	];
}
