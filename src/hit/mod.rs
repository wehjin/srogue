pub use damage_stat::*;
use rand::thread_rng;

use crate::init::{Dungeon, GameState};
use crate::level::add_exp;
use crate::message::sound_bell;
use crate::monster::{mon_name, Monster};
use crate::motion::{can_move, is_direction, one_move_rogue_legacy, MoveDirection};
use crate::objects::Object;
use crate::player::Player;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::random::rand_percent;
use crate::resources::avatar::Avatar;
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::resources::play::event::mon_hit::fight_report;
use crate::score::killed_by;
use crate::spec_hit::{check_imitator, cough_up};
use crate::throw::Motion;

pub fn rogue_hit(mon_id: u64, force_hit: bool, game: &mut GameState) {
	if check_imitator(mon_id, game) {
		return;
	}
	let hit_chance = if force_hit { 100 } else { get_hit_chance_player(game.player.weapon(), &game.player) };
	let hit_chance = if game.player.wizard { hit_chance * 2 } else { hit_chance };
	if !rand_percent(hit_chance) {
		if game.player.fight_monster.is_none() {
			let diary = game.as_diary_mut();
			diary.hit_message = Some("you miss".to_string());
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
					let diary = game.as_diary_mut();
					diary.hit_message = Some("you hit".to_string());
				}
			}
		}
	}
	let monster = game.mash.monster_mut(mon_id);
	monster.m_flags.seeks_gold = false;
	monster.wake_up();
}

pub fn rogue_damage(d: isize, mon_id: u64, game: &mut impl Dungeon) {
	if d >= game.as_fighter().hp_current {
		let fighter = game.as_fighter_mut();
		fighter.hp_current = 0;
		game.as_diary_mut().set_stats_changed(true);
		killed_by(Ending::Monster(game.as_monster(mon_id).name().to_string()), game);
	}
	game.as_fighter_mut().hp_current -= d;
	game.as_diary_mut().set_stats_changed(true);
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
			game.interrupt_and_slurp();
			let monster_name = mon_name(mon_id, game);
			let monster_report = format!("defeated the {}", monster_name);
			let fight_report = fight_report(monster_report, game);
			game.as_diary_mut().add_entry(&fight_report);
		}
		add_exp(game.mash.monster(mon_id).kill_exp(), true, game, &mut thread_rng());
		if game.mash.monster_flags(mon_id).holds {
			let health = game.as_health_mut();
			health.being_held = false;
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
				diary::show_prompt("direction?", &mut game.diary);
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
		one_move_rogue_legacy(direction, false, game, &mut thread_rng());
		if (!to_the_death && game.player.rogue.hp_current <= possible_damage)
			|| game.player.interrupted
			|| game.diary.cleaned_up.is_some()
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

