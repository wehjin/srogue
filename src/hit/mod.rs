#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
}

use libc::{c_char, c_short};
use crate::monster;
use crate::prelude::*;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: libc::c_short,
	pub _pad_x: libc::c_short,
	pub _pad_top: libc::c_short,
	pub _pad_left: libc::c_short,
	pub _pad_bottom: libc::c_short,
	pub _pad_right: libc::c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = ncurses::chtype;


#[no_mangle]
pub static mut fight_monster: *mut object = 0 as *const object as *mut object;
pub static mut detect_monster: bool = false;
pub static mut hit_message: String = String::new();

pub unsafe fn mon_hit(monster: *mut object, other: Option<&str>, flame: bool) {
	if !fight_monster.is_null() && monster != fight_monster {
		fight_monster = 0 as *mut object;
	}
	(*monster).trow = NO_ROOM;
	let mut hit_chance: usize = if cur_level >= (AMULET_LEVEL * 2) {
		100
	} else {
		let hit_chance = (*monster).m_hit_chance();
		hit_chance - (2 * rogue.exp + 2 * ring_exp - r_rings) as usize
	};
	if wizard {
		hit_chance /= 2;
	}

	if fight_monster.is_null() {
		interrupted = true;
	}
	if other.is_some() {
		hit_chance -= (rogue.exp + ring_exp - r_rings) as usize;
	}

	let base_monster_name = mon_name(monster);
	let monster_name = if let Some(name) = other { name } else { &base_monster_name };
	if !rand_percent(hit_chance) {
		if fight_monster.is_null() {
			hit_message = format!("{}the {} misses", hit_message, monster_name);
			message(&hit_message, 1);
			hit_message.clear();
		}
		return;
	}
	if fight_monster.is_null() {
		hit_message = format!("{}the {} hit", hit_message, monster_name);
		message(&hit_message, 1);
		hit_message.clear();
	}
	let mut damage: isize = if (*monster).m_flags.stationary {
		let stationary_damage = (*monster).stationary_damage();
		(*monster).set_stationary_damage(stationary_damage + 1);
		stationary_damage
	} else {
		let mut damage = get_damage((*monster).damage, DamageEffect::Roll);
		if other.is_some() && flame {
			damage -= get_armor_class(&*rogue.armor);
			if damage < 0 {
				damage = 1;
			}
		}
		let minus: isize = if cur_level >= AMULET_LEVEL * 2 {
			AMULET_LEVEL * 2 - cur_level
		} else {
			let mut minus = get_armor_class(&*rogue.armor) * 3;
			minus = (minus as f64 / 100.0 * damage as f64) as isize;
			minus
		};
		damage -= minus;
		damage
	};
	if wizard {
		damage /= 3;
	}
	if damage > 0 {
		rogue_damage(damage, &mut *monster);
	}
	if (*monster).m_flags.special_hit() {
		special_hit(monster);
	}
}

#[no_mangle]
pub unsafe extern "C" fn rogue_hit(mut monster: *mut object, force_hit: bool) {
	if !monster.is_null() {
		if check_imitator(monster) {
			return;
		}
		let hit_chance = if force_hit { 100 } else { get_hit_chance(&mut *rogue.weapon) };
		let hit_chance = if wizard { hit_chance * 2 } else { hit_chance } as usize;
		if !rand_percent(hit_chance) {
			if fight_monster.is_null() {
				hit_message = "you miss  ".to_string();
			}
		} else {
			let damage = get_weapon_damage(&mut *rogue.weapon);
			let damage = if wizard { damage * 3 } else { damage };
			if mon_damage(&mut *monster, damage as usize) {
				if fight_monster.is_null() {
					hit_message = "you hit  ".to_string();
				}
			}
		}
		check_gold_seeker(&mut *monster);
		wake_up(&mut *monster);
	}
}

pub unsafe fn rogue_damage(d: isize, monster: &mut object) {
	if d >= rogue.hp_current {
		rogue.hp_current = 0;
		print_stats(STAT_HP);
		killed_by(monster, 0);
	}
	rogue.hp_current -= d;
	print_stats(STAT_HP);
}

pub unsafe fn get_number(s: *const c_char) -> usize {
	let mut total = 0;
	let mut i = 0 as isize;
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

pub fn lget_number(s: &[u8]) -> u64 {
	let mut total: u64 = 0;
	let mut i: usize = 0;
	while s[i] >= '0' as u8 && s[i] <= '9' as u8 {
		total = 10 * total + (s[i] - '0' as u8) as u64;
		i += 1;
	}
	return total;
}

#[no_mangle]
pub unsafe extern "C" fn to_hit(mut obj: *mut object) -> usize {
	if obj.is_null() {
		return 1;
	}
	let hits = DamageStat::parse_first((*obj).damage).hits;
	return hits + (*obj).hit_enchant as usize;
}

#[no_mangle]
pub unsafe extern "C" fn damage_for_strength() -> i64 {
	let mut strength: libc::c_short = 0;
	strength = (rogue.str_current as i64 + add_strength as i64)
		as libc::c_short;
	if strength as i64 <= 6 as i64 {
		return strength as i64 - 5 as i64;
	}
	if strength as i64 <= 14 as i64 {
		return 1;
	}
	if strength as i64 <= 17 as i64 {
		return 3 as i64;
	}
	if strength as i64 <= 18 as i64 {
		return 4 as i64;
	}
	if strength as i64 <= 20 as i64 {
		return 5 as i64;
	}
	if strength as i64 <= 21 {
		return 6 as i64;
	}
	if strength as i64 <= 30 as i64 {
		return 7 as i64;
	}
	return 8 as i64;
}

pub unsafe fn mon_damage(monster: &mut object, damage: usize) -> bool {
	monster.set_hp_to_kill(monster.hp_to_kill() - damage as libc::c_short);
	if monster.hp_to_kill() <= 0 {
		let row = monster.row as i64;
		let col = monster.col as i64;
		SpotFlag::Monster.clear(&mut dungeon[row as usize][col as usize]);
		ncurses::mvaddch(row as i32, col as i32, get_dungeon_char(row, col));

		fight_monster = 0 as *const object as *mut object;
		cough_up(monster);
		let mn = monster::mon_name(monster);
		hit_message = format!("{}defeated the {}", hit_message, mn);
		message(&hit_message, 1);
		hit_message.clear();
		add_exp(monster.kill_exp, true);
		take_from_pack(monster, &mut level_monsters);

		if monster.m_flags.holds {
			being_held = false;
		}
		free_object(monster);
		return false;
	}
	return true;
}

#[no_mangle]
pub unsafe extern "C" fn fight(to_the_death: bool) {
	let mut first_miss: libc::c_char = 1 as libc::c_char;
	let mut monster: *mut object = 0 as *mut object;
	let mut ch: char = 0 as char;
	loop {
		ch = rgetchar() as u8 as char;
		if !(is_direction(ch as i32) == 0) {
			break;
		}
		sound_bell();
		if first_miss != 0 {
			message("direction?", 0);
			first_miss = 0 as i64 as libc::c_char;
		}
	}
	check_message();
	if ch as u32 == '\u{1b}' as u32 {
		return;
	}
	let mut row = rogue.row;
	let mut col = rogue.col;
	get_dir_rc(ch, &mut row, &mut col, false);
	let c = ncurses::mvinch(row as i32, col as i32);
	{
		let not_a_monster = (c as i64) < 'A' as i64 || c as i64 > 'Z' as i64;
		let cannot_move = !can_move(rogue.row as i64, rogue.col as i64, row , col );
		if not_a_monster || cannot_move {
			message("I see no monster there", 0);
			return;
		}
	}
	fight_monster = object_at(&mut level_monsters, row, col);
	if fight_monster.is_null() {
		return;
	}
	let possible_damage = if !(*fight_monster).m_flags.stationary {
		get_damage((*fight_monster).damage, DamageEffect::None) * 2 / 3
	} else {
		(*fight_monster).stationary_damage() - 1
	};
	while !fight_monster.is_null() {
		one_move_rogue(ch, false);
		if (!to_the_death && rogue.hp_current <= possible_damage)
			|| interrupted
			|| !Monster.is_set(dungeon[row as usize][col as usize]) {
			fight_monster = 0 as *mut object;
		} else {
			monster = object_at(&mut level_monsters, row, col);
			if monster != fight_monster {
				fight_monster = 0 as *mut object;
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
			if allow_off_screen || (*row > MIN_ROW as i64) {
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

pub unsafe fn get_hit_chance(weapon: &mut object) -> c_short {
	let mut hit_chance = 40isize;
	hit_chance += 3 * to_hit(weapon) as isize;
	hit_chance += ((2 * rogue.exp as isize) + (2 * ring_exp as isize)) - r_rings as isize;
	hit_chance as c_short
}

pub unsafe fn get_weapon_damage(weapon: &mut object) -> c_short {
	let mut damage = get_w_damage(weapon).expect("damage") as isize;
	damage += damage_for_strength() as isize;
	damage += (((rogue.exp as isize + ring_exp as isize) - r_rings as isize) + 1) / 2;
	damage as c_short
}

mod safe;

pub use safe::*;
use crate::prelude::SpotFlag::Monster;
use crate::prelude::stat_const::STAT_HP;
