use crate::hit;
use crate::hit::DamageEffect;
use crate::init::Dungeon;
use crate::monster::mon_name;
use crate::objects::get_armor_class;
use crate::prelude::AMULET_LEVEL;
use crate::random::rand_percent;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::message::Message;
use crate::resources::play::state::RunState;
use crate::spec_hit::special_hit;

pub fn mon_hit(mut game: RunState, mon_id: u64, flame: Option<&'static str>, ctx: &mut RunContext) -> RunState {
	if let Some(fight_id) = game.fight_to_death() {
		if mon_id != fight_id {
			game.set_fight_to_death(None);
		}
	}
	game.as_monster_mut(mon_id).clear_target_reset_stuck();
	let mut hit_chance: usize = if game.rogue_depth() >= (AMULET_LEVEL * 2) {
		100
	} else {
		let reduction = (2 * game.buffed_exp()) - game.debuf_exp();
		reduce_chance(game.as_monster(mon_id).m_hit_chance(), reduction)
	};
	if game.wizard() {
		hit_chance /= 2;
	}
	if game.fight_to_death().is_none() {
		let diary = game.as_diary_mut();
		diary.interrupted = true;
	}

	if flame.is_some() {
		hit_chance = reduce_chance(hit_chance, game.buffed_exp() - game.debuf_exp());
	}

	let base_monster_name = mon_name(mon_id, &game);
	let monster_name = if let Some(name) = flame { name } else { &base_monster_name };
	if !rand_percent(hit_chance) {
		if game.fight_to_death().is_none() {
			let monster_miss = format!("the {} misses", monster_name);
			let fight_report = fight_report(monster_miss, &mut game);
			game = Message::run_await_exit(game, fight_report, true, ctx);
		}
		return game;
	}
	if game.fight_to_death().is_none() {
		let monster_hit = format!("the {} hit", monster_name);
		let fight_report = fight_report(monster_hit, &mut game);
		game = Message::run_await_exit(game, fight_report, true, ctx);
	}
	let mut damage: isize = if !game.as_monster_flags(mon_id).stationary {
		let mut damage = hit::get_damage(game.as_monster(mon_id).m_damage(), DamageEffect::Roll);
		if flame.is_some() {
			damage -= get_armor_class(game.armor());
			if damage < 0 {
				damage = 1;
			}
		}
		let rogue_depth = game.rogue_depth();
		let minus: isize = if rogue_depth >= (AMULET_LEVEL * 2) {
			((AMULET_LEVEL * 2) - rogue_depth) as isize
		} else {
			let mut minus = get_armor_class(game.armor()) * 3;
			minus = (minus as f64 / 100.0 * damage as f64) as isize;
			minus
		};
		damage -= minus;
		damage
	} else {
		let original = game.as_monster(mon_id).stationary_damage;
		game.as_monster_mut(mon_id).stationary_damage += 1;
		original
	};
	if game.wizard() {
		damage /= 3;
	}
	if damage > 0 {
		hit::rogue_damage(damage, mon_id, &mut game);
	}
	if game.as_monster_flags(mon_id).special_hit() {
		special_hit(mon_id, &mut game);
	}
	game
}

pub fn fight_report(monster_message: String, game: &mut impl Dungeon) -> String {
	let player_message = game.as_diary_mut().hit_message.take().unwrap_or_else(String::new);
	if player_message.is_empty() {
		format!("{monster_message}")
	} else {
		format!("{player_message}  {monster_message}")
	}
}

fn reduce_chance(chance: usize, reduction: isize) -> usize {
	let reduction: usize = reduction.max(0) as usize;
	if chance <= reduction { 0 } else { chance - reduction }
}
