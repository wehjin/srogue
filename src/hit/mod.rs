pub use damage_stat::*;
use rand::thread_rng;

use crate::init::{Dungeon, GameState};
use crate::message::sound_bell;
use crate::monster::mon_name;
use crate::motion::{can_move, is_direction, one_move_rogue_legacy, MoveDirection};
use crate::objects::Object;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::resources::level::size::LevelSpot;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::message::Message;
use crate::resources::play::state::RunState;
use crate::score::killed_by;
use crate::spec_hit::cough_up;
use crate::throw::Motion;

pub fn rogue_hit(mut game: RunState, mon_id: u64, force_hit: bool, ctx: &mut RunContext) -> RunState {
	match roll_strike(mon_id, force_hit, &mut game) {
		RogueStrike::DefeatsTarget => {
			game.level.rogue.fight_to_death = None;
			cough_up(mon_id, &mut game);
			{
				let diary = game.as_diary_mut();
				diary.hit_message = None;
				let report = format!("defeated the {}", mon_name(mon_id, &game));
				game = Message::run_await_exit(game, report, true, ctx);
			}
			let removed = game.level.take_monster(LevelSpot::from(game.as_monster(mon_id).spot)).unwrap();
			if removed.m_flags.holds {
				let health = game.as_health_mut();
				health.being_held = false;
			}
			game.as_fighter_mut().exp.add_points(removed.kill_exp());
			game.as_diary_mut().set_stats_changed(true);
			game
		}
		RogueStrike::RevealsImitator => {
			game.as_monster_mut(mon_id).wake_up();
			if game.as_health().blind.is_inactive() {
				let report = format!("wait, that's a {}!", mon_name(mon_id, &game));
				game = Message::run_await_exit(game, report, true, ctx);
				game
			} else {
				game
			}
		}
		RogueStrike::DamagesTarget => {
			// Surviving monster.
			if game.level.rogue.fight_to_death.is_none() {
				let diary = game.as_diary_mut();
				diary.hit_message = Some("you hit".to_string());
			}
			alert_surviving_monster(game, mon_id)
		}
		RogueStrike::MissesTarget => {
			if game.level.rogue.fight_to_death.is_none() {
				let diary = game.as_diary_mut();
				diary.hit_message = Some("you miss".to_string());
			}
			alert_surviving_monster(game, mon_id)
		}
	}
}

pub enum RogueStrike {
	DefeatsTarget,
	RevealsImitator,
	DamagesTarget,
	MissesTarget,
}

fn roll_strike(mon_id: u64, force_hit: bool, game: &mut RunState) -> RogueStrike {
	match game.as_monster(mon_id).imitates() {
		true => RogueStrike::RevealsImitator,
		false => match roll_hit_success(force_hit, game) {
			true => {
				game.as_monster_mut(mon_id).hp_to_kill -= get_strike_damage(&game);
				match game.as_monster(mon_id).hp_to_kill <= 0 {
					true => RogueStrike::DefeatsTarget,
					false => RogueStrike::DamagesTarget
				}
			}
			false => RogueStrike::MissesTarget
		},
	}
}

pub fn alert_surviving_monster(mut game: RunState, mon_id: u64) -> RunState {
	let monster = game.as_monster_mut(mon_id);
	monster.m_flags.seeks_gold = false;
	monster.wake_up();
	game
}

pub fn get_strike_damage(game: &RunState) -> isize {
	let player_exp = game.buffed_exp();
	let player_debuf = game.debuf_exp();
	let player_str = game.buffed_strength();
	let damage = get_weapon_damage(game.weapon(), player_str, player_exp, player_debuf);
	let damage = if game.wizard() { damage * 3 } else { damage };
	damage
}

pub fn roll_hit_success(force_hit: bool, game: &mut RunState) -> bool {
	let hit_chance = if force_hit { 100 } else { get_hit_chance_player(&game) };
	let hit_chance = if game.wizard() { hit_chance * 2 } else { hit_chance };
	let is_hit = game.roll_chance(hit_chance);
	is_hit
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

pub fn get_hit_chance_player(state: &RunState) -> usize {
	let player_exp = state.buffed_exp();
	let player_debuf = state.debuf_exp();
	get_hit_chance(state.weapon(), player_exp, player_debuf)
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

