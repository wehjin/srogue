#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::{c_char};

pub static mut FIGHT_MONSTER: Option<u64> = None;
pub static mut HIT_MESSAGE: String = String::new();

fn reduce_chance(chance: usize, reduction: isize) -> usize {
	let reduction: usize = reduction.max(0) as usize;
	if chance <= reduction { 0 } else { chance - reduction }
}

pub unsafe fn mon_hit(monster: &mut monster::Monster, other: Option<&str>, flame: bool, player: &mut Player, level: &mut Level) {
	if let Some(monster_id) = FIGHT_MONSTER {
		if monster.id() == monster_id {
			FIGHT_MONSTER = None;
		}
	}
	monster.clear_target_spot();
	let mut hit_chance: usize = if player.cur_depth >= (AMULET_LEVEL * 2) {
		100
	} else {
		reduce_chance(monster.m_hit_chance(), 2 * player.rogue.exp + 2 * ring_exp - r_rings)
	};
	if wizard {
		hit_chance /= 2;
	}
	if FIGHT_MONSTER.is_none() {
		interrupted = true;
	}

	if other.is_some() {
		hit_chance = reduce_chance(hit_chance, player.rogue.exp + ring_exp - r_rings);
	}

	let base_monster_name = mon_name(&*monster, level);
	let monster_name = if let Some(name) = other { name } else { &base_monster_name };
	if !rand_percent(hit_chance) {
		if FIGHT_MONSTER.is_none() {
			HIT_MESSAGE = format!("{}the {} misses", HIT_MESSAGE, monster_name);
			message(&HIT_MESSAGE, 1);
			HIT_MESSAGE.clear();
		}
		return;
	}
	if FIGHT_MONSTER.is_none() {
		HIT_MESSAGE = format!("{}the {} hit", HIT_MESSAGE, monster_name);
		message(&HIT_MESSAGE, 1);
		HIT_MESSAGE.clear();
	}
	let mut damage: isize = if !monster.m_flags.stationary {
		let mut damage = get_damage(monster.m_damage(), DamageEffect::Roll);
		if other.is_some() && flame {
			damage -= get_armor_class(player.armor());
			if damage < 0 {
				damage = 1;
			}
		}
		let minus: isize = if player.cur_depth >= AMULET_LEVEL * 2 {
			(player.cur_depth - AMULET_LEVEL * 2) as isize * -1
		} else {
			let mut minus = get_armor_class(player.armor()) * 3;
			minus = (minus as f64 / 100.0 * damage as f64) as isize;
			minus
		};
		damage -= minus;
		damage
	} else {
		let original = monster.stationary_damage;
		monster.stationary_damage += 1;
		original
	};
	if wizard {
		damage /= 3;
	}
	if damage > 0 {
		rogue_damage(damage, monster, player);
	}
	if monster.m_flags.special_hit() {
		special_hit(monster, player, level);
	}
}

pub unsafe fn rogue_hit(monster: &mut monster::Monster, force_hit: bool, player: &mut Player, level: &mut Level) {
	if check_imitator(monster, level) {
		return;
	}
	let hit_chance = if force_hit { 100 } else { get_hit_chance_player(player.weapon(), player) };
	let hit_chance = if wizard { hit_chance * 2 } else { hit_chance };
	if !rand_percent(hit_chance) {
		if FIGHT_MONSTER.is_none() {
			HIT_MESSAGE = "you miss  ".to_string();
		}
	} else {
		let player_exp = player.exp();
		let player_str = player.cur_strength();
		let damage = get_weapon_damage(player.weapon(), player_str, player_exp);
		let damage = if wizard { damage * 3 } else { damage };
		if mon_damage(monster, damage, player, level) {
			if FIGHT_MONSTER.is_none() {
				HIT_MESSAGE = "you hit  ".to_string();
			}
		}
	}
	clear_gold_seeker(monster);
	monster.wake_up();
}

pub unsafe fn rogue_damage(d: isize, monster: &monster::Monster, player: &mut Player) {
	if d >= player.rogue.hp_current {
		player.rogue.hp_current = 0;
		print_stats(STAT_HP, player);
		killed_by(Ending::Monster(monster), player);
	}
	player.rogue.hp_current -= d;
	print_stats(STAT_HP, player);
}


pub fn get_damage(damage_stats: &[DamageStat], effect: DamageEffect) -> isize {
	let mut total = 0;
	for stat in damage_stats {
		total += stat.roll_damage(effect);
	}
	return total as isize;
}

pub fn get_w_damage(obj: Option<&object>) -> isize {
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

pub unsafe fn damage_for_strength(player_strength: isize) -> isize {
	let strength = player_strength + add_strength;
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

pub unsafe fn mon_damage(monster: &mut monster::Monster, damage: isize, player: &mut Player, level: &mut Level) -> bool {
	monster.hp_to_kill -= damage;
	if monster.hp_to_kill <= 0 {
		let row = monster.spot.row;
		let col = monster.spot.col;
		level.dungeon[row as usize][col as usize].remove_kind(CellKind::Monster);
		ncurses::mvaddch(row as i32, col as i32, get_dungeon_char(row, col, level));

		FIGHT_MONSTER = None;
		cough_up(monster, player, level);
		let mn = mon_name(monster, level);
		HIT_MESSAGE = format!("{}defeated the {}", HIT_MESSAGE, mn);
		message(&HIT_MESSAGE, 1);
		HIT_MESSAGE.clear();
		add_exp(monster.kill_exp(), true, player);
		if monster.m_flags.holds {
			level.being_held = false;
		}
		MASH.remove_monster(monster.id());
		return false;
	}
	return true;
}

pub unsafe fn fight(to_the_death: bool, player: &mut Player, level: &mut Level) {
	let mut first_miss: bool = true;
	let mut ch: char;
	loop {
		ch = rgetchar() as u8 as char;
		if is_direction(ch) {
			break;
		}
		sound_bell();
		if first_miss {
			message("direction?", 0);
			first_miss = false;
		}
	}
	check_message();
	if ch == CANCEL {
		return;
	}
	let mut row = player.rogue.row;
	let mut col = player.rogue.col;
	get_dir_rc(ch, &mut row, &mut col, false);
	let c = ncurses::mvinch(row as i32, col as i32);
	{
		let not_a_monster = (c as i64) < 'A' as i64 || c as i64 > 'Z' as i64;
		let cannot_move = !can_move(player.rogue.row, player.rogue.col, row, col, level);
		if not_a_monster || cannot_move {
			message("I see no monster there", 0);
			return;
		}
	}
	FIGHT_MONSTER = MASH.monster_at_spot(row, col).map(|m| m.id());
	if FIGHT_MONSTER.is_none() {
		return;
	}
	let possible_damage = {
		let fight_id = FIGHT_MONSTER.expect("some fight-monster id");
		let fight_monster = MASH.monster_with_id(fight_id).expect("fight monster");
		if !fight_monster.m_flags.stationary {
			get_damage(fight_monster.m_damage(), DamageEffect::None) * 2 / 3
		} else {
			fight_monster.stationary_damage - 1
		}
	};
	while FIGHT_MONSTER.is_some() {
		one_move_rogue(ch, false, player, level);
		if (!to_the_death && player.rogue.hp_current <= possible_damage)
			|| interrupted
			|| !level.dungeon[row as usize][col as usize].is_monster() {
			FIGHT_MONSTER = None;
		} else {
			let monster_id = MASH.monster_at_spot(row, col).map(|m| m.id());
			if monster_id != FIGHT_MONSTER {
				FIGHT_MONSTER = None;
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

pub unsafe fn get_hit_chance_player(weapon: Option<&object>, player: &Player) -> usize {
	let player_exp = player.exp();
	get_hit_chance(weapon, player_exp)
}

pub unsafe fn get_hit_chance(obj: Option<&object>, player_exp: isize) -> usize {
	let mut hit_chance = 40isize;
	hit_chance += 3 * to_hit(obj) as isize;
	hit_chance += ((2 * player_exp) + (2 * ring_exp)) - r_rings;
	hit_chance as usize
}

fn to_hit(obj: Option<&object>) -> usize {
	if let Some(obj) = obj {
		obj.enhanced_damage().hits
	} else {
		1
	}
}

pub unsafe fn get_weapon_damage(weapon: Option<&object>, strength: isize, exp: isize) -> isize {
	let mut damage = get_w_damage(weapon);
	damage += damage_for_strength(strength) as isize;
	damage += (((exp + ring_exp) - r_rings) + 1) / 2;
	damage
}

mod damage_stat;

pub use damage_stat::*;
use crate::level::{add_exp, Level};
use crate::level::constants::{DCOLS, DROWS};
use crate::message::{CANCEL, check_message, message, print_stats, rgetchar, sound_bell};
use crate::monster;
use crate::monster::{MASH, mon_name};
use crate::objects::{get_armor_class, object};
use crate::play::interrupted;
use crate::player::Player;
use crate::prelude::{AMULET_LEVEL, CellKind, MIN_ROW};
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::Weapon;
use crate::prelude::stat_const::STAT_HP;
use crate::r#move::{can_move, is_direction, one_move_rogue};
use crate::random::rand_percent;
use crate::ring::{add_strength, r_rings, ring_exp};
use crate::room::get_dungeon_char;
use crate::score::killed_by;
use crate::spec_hit::{clear_gold_seeker, check_imitator, cough_up, special_hit};
use crate::zap::wizard;
