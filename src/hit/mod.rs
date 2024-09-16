pub use damage_stat::*;

use crate::init::GameState;
use crate::level::add_exp;
use crate::message::sound_bell;
use crate::monster::{mon_name, Monster};
use crate::motion::{can_move, is_direction, one_move_rogue, MoveDirection};
use crate::objects::{get_armor_class, Object};
use crate::player::Player;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::prelude::AMULET_LEVEL;
use crate::random::rand_percent;
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::score::killed_by;
use crate::spec_hit::{check_imitator, cough_up, special_hit};
use crate::throw::Motion;

fn reduce_chance(chance: usize, reduction: isize) -> usize {
	let reduction: usize = reduction.max(0) as usize;
	if chance <= reduction { 0 } else { chance - reduction }
}

pub fn mon_hit(mon_id: u64, other: Option<&str>, flame: bool, game: &mut GameState) {
	if let Some(fight_id) = game.player.fight_monster {
		if mon_id == fight_id {
			game.player.fight_monster = None;
		}
	}
	game.mash.monster_mut(mon_id).clear_target_spot();
	let mut hit_chance: usize = if game.player.cur_depth >= (AMULET_LEVEL * 2) {
		100
	} else {
		let reduction = (2 * game.player.buffed_exp()) - game.player.debuf_exp();
		reduce_chance(game.mash.monster(mon_id).m_hit_chance(), reduction)
	};
	if game.player.wizard {
		hit_chance /= 2;
	}
	if game.player.fight_monster.is_none() {
		game.player.interrupted = true;
	}

	if other.is_some() {
		hit_chance = reduce_chance(hit_chance, game.player.buffed_exp() - game.player.debuf_exp());
	}

	let base_monster_name = mon_name(game.mash.monster(mon_id), &game.player, &game.level);
	let monster_name = if let Some(name) = other { name } else { &base_monster_name };
	if !rand_percent(hit_chance) {
		if game.player.fight_monster.is_none() {
			let msg = format!("{}the {} misses", game.player.hit_message, monster_name);
			game.player.hit_message.clear();
			game.player.interrupt_and_slurp();
			game.diary.add_entry(&msg);
		}
		return;
	}
	if game.player.fight_monster.is_none() {
		let msg = format!("{}the {} hit", game.player.hit_message, monster_name);
		game.player.hit_message.clear();
		game.player.interrupt_and_slurp();
		game.diary.add_entry(&msg);
	}
	let mut damage: isize = if !game.mash.monster_flags(mon_id).stationary {
		let mut damage = get_damage(game.mash.monster(mon_id).m_damage(), DamageEffect::Roll);
		if other.is_some() && flame {
			damage -= get_armor_class(game.player.armor());
			if damage < 0 {
				damage = 1;
			}
		}
		let minus: isize = if game.player.cur_depth >= AMULET_LEVEL * 2 {
			(game.player.cur_depth - AMULET_LEVEL * 2) * -1
		} else {
			let mut minus = get_armor_class(game.player.armor()) * 3;
			minus = (minus as f64 / 100.0 * damage as f64) as isize;
			minus
		};
		damage -= minus;
		damage
	} else {
		let original = game.mash.monster(mon_id).stationary_damage;
		game.mash.monster_mut(mon_id).stationary_damage += 1;
		original
	};
	if game.player.wizard {
		damage /= 3;
	}
	if damage > 0 {
		rogue_damage(damage, mon_id, game);
	}
	if game.mash.monster_flags(mon_id).special_hit() {
		special_hit(mon_id, game);
	}
}

pub fn rogue_hit(mon_id: u64, force_hit: bool, game: &mut GameState) {
	if check_imitator(mon_id, game) {
		return;
	}
	let hit_chance = if force_hit { 100 } else { get_hit_chance_player(game.player.weapon(), &game.player) };
	let hit_chance = if game.player.wizard { hit_chance * 2 } else { hit_chance };
	if !rand_percent(hit_chance) {
		if game.player.fight_monster.is_none() {
			game.player.hit_message = "you miss  ".to_string();
		}
	} else {
		let player_exp = game.player.buffed_exp();
		let player_debuf = game.player.debuf_exp();
		let player_str = game.player.buffed_strength();
		let damage = get_weapon_damage(game.player.weapon(), player_str, player_exp, player_debuf);
		let damage = if game.player.wizard { damage * 3 } else { damage };
		match mon_damage(mon_id, damage, game) {
			MonDamageEffect::MonsterDies(_) => {
				// mon_id is no longer in the mash.
				return;
			}
			MonDamageEffect::MonsterSurvives => {
				if game.player.fight_monster.is_none() {
					game.player.hit_message = "you hit  ".to_string();
				}
			}
		}
	}
	let monster = game.mash.monster_mut(mon_id);
	monster.m_flags.seeks_gold = false;
	monster.wake_up();
}

pub fn rogue_damage(d: isize, mon_id: u64, game: &mut GameState) {
	if d >= game.player.rogue.hp_current {
		game.player.rogue.hp_current = 0;
		game.stats_changed = true;
		killed_by(Ending::Monster(game.mash.monster(mon_id).name().to_string()), game);
	}
	game.player.rogue.hp_current -= d;
	game.stats_changed = true;
}


pub fn get_damage(damage_stats: &[DamageStat], effect: DamageEffect) -> isize {
	let mut total = 0;
	for stat in damage_stats {
		total += stat.roll_damage(effect);
	}
	total as isize
}

pub fn get_w_damage(obj: Option<&Object>) -> isize {
	if let Some(obj) = obj {
		if obj.what_is == Weapon {
			return get_damage(&[obj.enhanced_damage()], DamageEffect::Roll);
		}
	}
	-1
}

pub fn damage_for_strength(strength: isize) -> isize {
	if strength <= 6 {
		return strength - 5;
	}
	if strength <= 14 {
		return 1;
	}
	if strength <= 17 {
		return 3;
	}
	if strength <= 18 {
		return 4;
	}
	if strength <= 20 {
		return 5;
	}
	if strength <= 21 {
		return 6;
	}
	if strength <= 30 {
		return 7;
	}
	8
}

pub enum MonDamageEffect {
	MonsterDies(Monster),
	MonsterSurvives,
}

pub fn mon_damage(mon_id: u64, damage: isize, game: &mut GameState) -> MonDamageEffect {
	game.mash.monster_mut(mon_id).hp_to_kill -= damage;
	if game.mash.monster(mon_id).hp_to_kill <= 0 {
		{
			let spot = game.mash.monster(mon_id).spot;
			game.level.cell_mut(spot).set_monster(false);
			game.render_spot(spot);
		}
		game.player.fight_monster = None;
		cough_up(mon_id, game);
		{
			let monster = game.mash.monster(mon_id);
			let monster_name = mon_name(monster, &game.player, &game.level);
			let msg = format!("{}defeated the {}", game.player.hit_message, monster_name);
			game.player.hit_message.clear();
			game.player.interrupt_and_slurp();
			game.diary.add_entry(&msg);
		}
		add_exp(game.mash.monster(mon_id).kill_exp(), true, game);
		if game.mash.monster_flags(mon_id).holds {
			game.level.being_held = false;
		}
		let removed = game.mash.remove_monster(game.mash.monster(mon_id).id());
		return MonDamageEffect::MonsterDies(removed);
	}
	MonDamageEffect::MonsterSurvives
}

pub fn fight(to_the_death: bool, game: &mut GameState) {
	let mut first_miss: bool = true;
	let motion = {
		let mut ch: char;
		loop {
			ch = rgetchar() as u8 as char;
			if is_direction(ch) {
				break;
			}
			sound_bell();
			if first_miss {
				diary::show_prompt("direction?", &game.diary);
				first_miss = false;
			}
		}
		if ch == CANCEL_CHAR {
			return;
		}
		Motion::from(ch)
	};
	let fight_spot = match game.player.to_spot().after_motion(motion) {
		None => {
			return;
		}
		Some(on_screen_spot) => on_screen_spot,
	};
	let cannot_move = !can_move(game.player.rogue.row, game.player.rogue.col, fight_spot.row, fight_spot.col, &game.level);
	let not_a_monster = !game.has_non_imitating_monster_at(fight_spot);
	if not_a_monster || cannot_move {
		game.diary.add_entry("I see no monster there");
		return;
	}
	game.player.fight_monster = game.mash.monster_at_spot(fight_spot.row, fight_spot.col).map(|m| m.id());
	if game.player.fight_monster.is_none() {
		return;
	}

	let possible_damage = {
		let fight_id = game.player.fight_monster.expect("some fight-monster id");
		let fight_monster = game.mash.monster_mut(fight_id);
		if !fight_monster.m_flags.stationary {
			get_damage(fight_monster.m_damage(), DamageEffect::None) * 2 / 3
		} else {
			fight_monster.stationary_damage - 1
		}
	};
	while game.player.fight_monster.is_some() {
		let direction = MoveDirection::from(motion.to_char());
		one_move_rogue(direction, false, game);
		if (!to_the_death && game.player.rogue.hp_current <= possible_damage)
			|| game.player.interrupted
			|| game.player.cleaned_up.is_some()
			|| !game.cell_at(fight_spot).has_monster() {
			game.player.fight_monster = None;
		} else {
			let mon_id = game.mash.monster_id_at_spot(fight_spot.row, fight_spot.col);
			if mon_id != game.player.fight_monster {
				game.player.fight_monster = None;
			}
		}
	}
}

pub fn get_hit_chance_player(weapon: Option<&Object>, player: &Player) -> usize {
	let player_exp = player.buffed_exp();
	let player_debuf = player.debuf_exp();
	get_hit_chance(weapon, player_exp, player_debuf)
}

pub fn get_hit_chance(obj: Option<&Object>, buffed_exp: isize, debuf_exp: isize) -> usize {
	let mut hit_chance = 40isize;
	hit_chance += 3 * to_hit(obj) as isize;
	hit_chance += (2 * buffed_exp) - debuf_exp;
	hit_chance as usize
}

fn to_hit(obj: Option<&Object>) -> usize {
	if let Some(obj) = obj {
		obj.enhanced_damage().hits
	} else {
		1
	}
}

pub fn get_weapon_damage(weapon: Option<&Object>, buffed_str: isize, buffed_exp: isize, debuf_exp: isize) -> isize {
	let mut damage = get_w_damage(weapon);
	damage += damage_for_strength(buffed_str);
	damage += (buffed_exp - debuf_exp + 1) / 2;
	damage
}

mod damage_stat;

