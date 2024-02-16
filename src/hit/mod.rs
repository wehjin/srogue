#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::c_char;

pub use damage_stat::*;

use crate::init::GameState;
use crate::level::add_exp;
use crate::level::constants::{DCOLS, DROWS};
use crate::message::{CANCEL, print_stats, rgetchar, sound_bell};
use crate::monster::{mon_name, Monster};
use crate::objects::{get_armor_class, Object};
use crate::player::Player;
use crate::prelude::{AMULET_LEVEL, MIN_ROW};
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::prelude::stat_const::STAT_HP;
use crate::r#move::{can_move, is_direction, one_move_rogue};
use crate::random::rand_percent;
use crate::room::get_dungeon_char;
use crate::score::killed_by;
use crate::spec_hit::{check_imitator, clear_gold_seeker, cough_up, special_hit};

pub static mut HIT_MESSAGE: String = String::new();

fn reduce_chance(chance: usize, reduction: isize) -> usize {
	let reduction: usize = reduction.max(0) as usize;
	if chance <= reduction { 0 } else { chance - reduction }
}

pub unsafe fn mon_hit(mon_id: u64, other: Option<&str>, flame: bool, game: &mut GameState) {
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
			HIT_MESSAGE = format!("{}the {} misses", HIT_MESSAGE, monster_name);
			game.player.interrupt_and_slurp();
			game.dialog.message(&HIT_MESSAGE, 1);
			HIT_MESSAGE.clear();
		}
		return;
	}
	if game.player.fight_monster.is_none() {
		HIT_MESSAGE = format!("{}the {} hit", HIT_MESSAGE, monster_name);
		game.player.interrupt_and_slurp();
		game.dialog.message(&HIT_MESSAGE, 1);
		HIT_MESSAGE.clear();
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

pub unsafe fn rogue_hit(mon_id: u64, force_hit: bool, game: &mut GameState) {
	if check_imitator(mon_id, game) {
		return;
	}
	let hit_chance = if force_hit { 100 } else { get_hit_chance_player(game.player.weapon(), &game.player) };
	let hit_chance = if game.player.wizard { hit_chance * 2 } else { hit_chance };
	if !rand_percent(hit_chance) {
		if game.player.fight_monster.is_none() {
			HIT_MESSAGE = "you miss  ".to_string();
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
					HIT_MESSAGE = "you hit  ".to_string();
				}
			}
		}
	}
	let monster = game.mash.monster_mut(mon_id);
	clear_gold_seeker(monster);
	monster.wake_up();
}

pub unsafe fn rogue_damage(d: isize, mon_id: u64, game: &mut GameState) {
	if d >= game.player.rogue.hp_current {
		game.player.rogue.hp_current = 0;
		print_stats(STAT_HP, &mut game.player);
		killed_by(Ending::Monster(game.mash.monster(mon_id).name().to_string()), game);
	}
	game.player.rogue.hp_current -= d;
	print_stats(STAT_HP, &mut game.player);
}


pub fn get_damage(damage_stats: &[DamageStat], effect: DamageEffect) -> isize {
	let mut total = 0;
	for stat in damage_stats {
		total += stat.roll_damage(effect);
	}
	return total as isize;
}

pub fn get_w_damage(obj: Option<&Object>) -> isize {
	if let Some(obj) = obj {
		if obj.what_is == Weapon {
			return get_damage(&[obj.enhanced_damage()], DamageEffect::Roll);
		}
	}
	-1
}

pub unsafe fn get_number(s: *const c_char) -> usize {
	let mut total = 0;
	let mut i = 0;
	loop {
		let c = *s.offset(i) as u8 as char;
		if c < '0' || c > '9' {
			break;
		}
		total = (10 * total) + c.to_digit(10).expect("digit") as usize;
		i += 1;
	}
	return total;
}

pub unsafe fn damage_for_strength(strength: isize) -> isize {
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
	return 8;
}

pub enum MonDamageEffect {
	MonsterDies(Monster),
	MonsterSurvives,
}

pub unsafe fn mon_damage(mon_id: u64, damage: isize, game: &mut GameState) -> MonDamageEffect {
	game.mash.monster_mut(mon_id).hp_to_kill -= damage;
	if game.mash.monster(mon_id).hp_to_kill <= 0 {
		{
			let monster = game.mash.monster(mon_id);
			let row = monster.spot.row;
			let col = monster.spot.col;
			game.level.dungeon[row as usize][col as usize].set_monster(false);
			ncurses::mvaddch(row as i32, col as i32, get_dungeon_char(row, col, game));
		}
		game.player.fight_monster = None;
		cough_up(mon_id, game);
		{
			let monster = game.mash.monster(mon_id);
			let monster_name = mon_name(monster, &game.player, &game.level);
			HIT_MESSAGE = format!("{}defeated the {}", HIT_MESSAGE, monster_name);
			game.player.interrupt_and_slurp();
			game.dialog.message(&HIT_MESSAGE, 1);
			HIT_MESSAGE.clear();
		}
		add_exp(game.mash.monster(mon_id).kill_exp(), true, game);
		if game.mash.monster_flags(mon_id).holds {
			game.level.being_held = false;
		}
		let removed = game.mash.remove_monster(game.mash.monster(mon_id).id());
		return MonDamageEffect::MonsterDies(removed);
	}
	return MonDamageEffect::MonsterSurvives;
}

pub unsafe fn fight(to_the_death: bool, game: &mut GameState) {
	let mut first_miss: bool = true;
	let mut ch: char;
	loop {
		ch = rgetchar() as u8 as char;
		if is_direction(ch) {
			break;
		}
		sound_bell();
		if first_miss {
			game.dialog.message("direction?", 0);
			first_miss = false;
		}
	}
	game.dialog.clear_message();
	if ch == CANCEL {
		return;
	}
	let mut row = game.player.rogue.row;
	let mut col = game.player.rogue.col;
	get_dir_rc(ch, &mut row, &mut col, false);
	let c = ncurses::mvinch(row as i32, col as i32);
	{
		let not_a_monster = (c as i64) < 'A' as i64 || c as i64 > 'Z' as i64;
		let cannot_move = !can_move(game.player.rogue.row, game.player.rogue.col, row, col, &game.level);
		if not_a_monster || cannot_move {
			game.dialog.message("I see no monster there", 0);
			return;
		}
	}
	game.player.fight_monster = game.mash.monster_at_spot(row, col).map(|m| m.id());
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
		one_move_rogue(ch, false, game);
		if (!to_the_death && game.player.rogue.hp_current <= possible_damage)
			|| game.player.interrupted
			|| !game.level.dungeon[row as usize][col as usize].has_monster() {
			game.player.fight_monster = None;
		} else {
			let monster_id = game.mash.monster_at_spot(row, col).map(|m| m.id());
			if monster_id != game.player.fight_monster {
				game.player.fight_monster = None;
			}
		}
	}
}

pub fn get_dir_rc(dir: char, row: &mut i64, col: &mut i64, allow_off_screen: bool) {
	match dir {
		'h' => {
			if allow_off_screen || (*col > 0) {
				*col -= 1;
			}
		}
		'j' => {
			if allow_off_screen || (*row < (DROWS - 2) as i64) {
				*row += 1
			}
		}
		'k' => {
			if allow_off_screen || (*row > MIN_ROW) {
				*row -= 1;
			}
		}
		'l' => {
			if allow_off_screen || (*col < (DCOLS - 1) as i64) {
				*col += 1;
			}
		}
		'y' => {
			if allow_off_screen || ((*row > MIN_ROW) && (*col > 0)) {
				*row -= 1;
				*col -= 1;
			}
		}
		'u' => {
			if allow_off_screen || ((*row > MIN_ROW) & &(*col < (DCOLS - 1) as i64)) {
				*row -= 1;
				*col += 1;
			}
		}
		'n' => {
			if allow_off_screen || ((*row < (DROWS - 2) as i64) && (*col < (DCOLS - 1) as i64)) {
				*row += 1;
				*col += 1;
			}
		}
		'b' => {
			if allow_off_screen || ((*row < (DROWS - 2) as i64) && (*col > 0)) {
				*row += 1;
				*col -= 1;
			}
		}
		_ => unreachable!("invalid direction"),
	}
}

pub unsafe fn get_hit_chance_player(weapon: Option<&Object>, player: &Player) -> usize {
	let player_exp = player.buffed_exp();
	let player_debuf = player.debuf_exp();
	get_hit_chance(weapon, player_exp, player_debuf)
}

pub unsafe fn get_hit_chance(obj: Option<&Object>, buffed_exp: isize, debuf_exp: isize) -> usize {
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

pub unsafe fn get_weapon_damage(weapon: Option<&Object>, buffed_str: isize, buffed_exp: isize, debuf_exp: isize) -> isize {
	let mut damage = get_w_damage(weapon);
	damage += damage_for_strength(buffed_str);
	damage += (buffed_exp - debuf_exp + 1) / 2;
	damage
}

mod damage_stat;

