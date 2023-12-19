#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;

	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_monsters: object;
	static mut cur_level: libc::c_short;
	static mut add_strength: libc::c_short;
	static mut ring_exp: libc::c_short;
	static mut r_rings: libc::c_short;
	static mut being_held: libc::c_char;
	static mut wizard: libc::c_char;
}

use libc::{c_char, c_short};
use crate::monster;
use crate::prelude::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: libc::c_short,
	pub _curx: libc::c_short,
	pub _maxy: libc::c_short,
	pub _maxx: libc::c_short,
	pub _begy: libc::c_short,
	pub _begx: libc::c_short,
	pub _flags: libc::c_short,
	pub _attrs: attr_t,
	pub _bkgd: chtype,
	pub _notimeout: libc::c_int,
	pub _clear: libc::c_int,
	pub _leaveok: libc::c_int,
	pub _scroll: libc::c_int,
	pub _idlok: libc::c_int,
	pub _idcok: libc::c_int,
	pub _immed: libc::c_int,
	pub _sync: libc::c_int,
	pub _use_keypad: libc::c_int,
	pub _delay: libc::c_int,
	pub _line: *mut ldat,
	pub _regtop: libc::c_short,
	pub _regbottom: libc::c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: libc::c_short,
}

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
pub type attr_t = chtype;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: libc::c_short,
	pub hp_max: libc::c_short,
	pub str_current: libc::c_short,
	pub str_max: libc::c_short,
	pub pack: object,
	pub gold: libc::c_long,
	pub exp: libc::c_short,
	pub exp_points: libc::c_long,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub fchar: libc::c_short,
	pub moves_left: libc::c_short,
}

pub type fighter = fight;

#[no_mangle]
pub static mut fight_monster: *mut object = 0 as *const object as *mut object;
#[no_mangle]
pub static mut detect_monster: libc::c_char = 0;
pub static mut hit_message: String = String::new();

#[no_mangle]
pub unsafe extern "C" fn mon_hit(mut monster: *mut object, other: Option<&str>, mut flame: libc::c_char) {
	let mut damage: libc::c_short = 0;
	let mut hit_chance: libc::c_short = 0;
	let mut minus: libc::c_int = 0;
	if !fight_monster.is_null() && monster != fight_monster {
		fight_monster = 0 as *mut object;
	}
	(*monster).trow = -(1 as libc::c_int) as libc::c_short;
	if cur_level as libc::c_int >= 26 as libc::c_int * 2 as libc::c_int {
		hit_chance = 100 as libc::c_int as libc::c_short;
	} else {
		hit_chance = (*monster).class;
		hit_chance = (hit_chance as libc::c_int
			- (2 as libc::c_int * rogue.exp as libc::c_int
			+ 2 as libc::c_int * ring_exp as libc::c_int - r_rings as libc::c_int)
		) as c_short;
	}
	if wizard != 0 {
		hit_chance = (hit_chance as libc::c_int / 2 as libc::c_int) as libc::c_short;
	}
	if fight_monster.is_null() {
		interrupted = true;
	}
	if other.is_some() {
		hit_chance = (hit_chance as libc::c_int
			- (rogue.exp as libc::c_int + ring_exp as libc::c_int
			- r_rings as libc::c_int)) as libc::c_short;
	}

	let base_monster_name = mon_name(monster);
	let monster_name = if let Some(name) = other { name } else { &base_monster_name; };
	if !rand_percent(hit_chance as libc::c_int) {
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
	if !(*monster).m_flags.stationary {
		damage = get_damage((*monster).damage, DamageEffect::Roll) as libc::c_short;
		if !other.is_null() {
			if flame != 0 {
				damage = (damage as libc::c_int - get_armor_class(rogue.armor))
					as libc::c_short;
				if (damage as libc::c_int) < 0 as libc::c_int {
					damage = 1 as libc::c_int as libc::c_short;
				}
			}
		}
		if cur_level as libc::c_int >= 26 as libc::c_int * 2 as libc::c_int {
			minus = 26 as libc::c_int * 2 as libc::c_int - cur_level as libc::c_int;
		} else {
			minus = (get_armor_class(rogue.armor) as libc::c_double * 3.00f64)
				as libc::c_int;
			minus = minus / 100 as libc::c_int * damage as libc::c_int;
		}
		damage = (damage as libc::c_int - minus as libc::c_short as libc::c_int)
			as libc::c_short;
	} else {
		let fresh0 = (*monster).identified;
		(*monster).identified = (*monster).identified + 1;
		damage = fresh0;
	}
	if wizard != 0 {
		damage = (damage as libc::c_int / 3 as libc::c_int) as libc::c_short;
	}
	if damage as libc::c_int > 0 as libc::c_int {
		rogue_damage(damage as usize, &mut *monster);
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
		let hit_chance = if wizard != 0 { hit_chance * 2 } else { hit_chance };
		if !rand_percent(hit_chance as libc::c_int) {
			if fight_monster.is_null() {
				hit_message = "you miss  ".to_string();
			}
		} else {
			let damage = get_weapon_damage(&mut *rogue.weapon);
			let damage = if wizard != 0 { damage * 3 } else { damage };
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

pub unsafe fn rogue_damage(d: usize, monster: &mut object) {
	let d = d as libc::c_short;
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

#[no_mangle]
pub unsafe extern "C" fn lget_number(mut s: *mut libc::c_char) -> libc::c_long {
	let mut i: libc::c_long = 0 as libc::c_int as libc::c_long;
	let mut total: libc::c_long = 0 as libc::c_int as libc::c_long;
	while *s.offset(i as isize) as libc::c_int >= '0' as i32
		&& *s.offset(i as isize) as libc::c_int <= '9' as i32
	{
		total = 10 as libc::c_int as libc::c_long * total
			+ (*s.offset(i as isize) as libc::c_int - '0' as i32) as libc::c_long;
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
pub unsafe extern "C" fn damage_for_strength() -> libc::c_int {
	let mut strength: libc::c_short = 0;
	strength = (rogue.str_current as libc::c_int + add_strength as libc::c_int)
		as libc::c_short;
	if strength as libc::c_int <= 6 as libc::c_int {
		return strength as libc::c_int - 5 as libc::c_int;
	}
	if strength as libc::c_int <= 14 as libc::c_int {
		return 1 as libc::c_int;
	}
	if strength as libc::c_int <= 17 as libc::c_int {
		return 3 as libc::c_int;
	}
	if strength as libc::c_int <= 18 as libc::c_int {
		return 4 as libc::c_int;
	}
	if strength as libc::c_int <= 20 as libc::c_int {
		return 5 as libc::c_int;
	}
	if strength as libc::c_int <= 21 as libc::c_int {
		return 6 as libc::c_int;
	}
	if strength as libc::c_int <= 30 as libc::c_int {
		return 7 as libc::c_int;
	}
	return 8 as libc::c_int;
}

pub unsafe fn mon_damage(monster: &mut object, damage: usize) -> bool {
	monster.set_hp_to_kill(monster.hp_to_kill() - damage as libc::c_short);
	if monster.hp_to_kill() <= 0 {
		let row = monster.row as usize;
		let col = monster.col as usize;
		SpotFlag::Monster.clear(&mut dungeon[row][col]);
		ncurses::mvaddch(row as i32, col as i32, get_dungeon_char(row, col));

		fight_monster = 0 as *const object as *mut object;
		cough_up(monster);
		let mn = monster::mon_name(monster);
		hit_message = format!("{}defeated the {}", hit_message, mn);
		message(&hit_message, 1);
		hit_message.clear();
		add_exp(monster.kill_exp as libc::c_int, true);
		take_from_pack(monster, &mut level_monsters);

		if monster.m_flags.holds {
			being_held = 0;
		}
		free_object(monster);
		return false;
	}
	return true;
}

#[no_mangle]
pub unsafe extern "C" fn fight(to_the_death: bool, settings: &Settings) {
	let mut first_miss: libc::c_char = 1 as libc::c_int as libc::c_char;
	let mut monster: *mut object = 0 as *mut object;
	let mut ch: libc::c_short = 0;
	loop {
		ch = rgetchar() as libc::c_short;
		if !(is_direction(ch as libc::c_int) == 0) {
			break;
		}
		sound_bell();
		if first_miss != 0 {
			message("direction?", 0);
			first_miss = 0 as libc::c_int as libc::c_char;
		}
	}
	check_message();
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	let mut row = rogue.row;
	let mut col = rogue.col;
	get_dir_rc(ch, &mut row, &mut col, false);
	let c = ncurses::mvinch(row as i32, col as i32);
	{
		let not_a_monster = (c as libc::c_int) < 'A' as i32 || c as libc::c_int > 'Z' as i32;
		let cannot_move = !can_move(rogue.row as usize, rogue.col as usize, row as usize, col as usize);
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
		one_move_rogue(ch, false, settings);
		if (!to_the_death && rogue.hp_current <= possible_damage as i16)
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

pub fn get_dir_rc(dir: c_short, row: &mut c_short, col: &mut c_short, allow_off_screen: bool) {
	let dir = char::from(dir as u8);
	match dir {
		'h' => {
			if allow_off_screen || (*col > 0) {
				*col -= 1;
			}
		}
		'j' => {
			if allow_off_screen || (*row < (DROWS - 2) as i16) {
				*row += 1
			}
		}
		'k' => {
			if allow_off_screen || (*row > MIN_ROW as i16) {
				*row -= 1;
			}
		}
		'l' => {
			if allow_off_screen || (*col < (DCOLS - 1) as i16) {
				*col += 1;
			}
		}
		'y' => {
			if allow_off_screen || ((*row > MIN_ROW as i16) && (*col > 0)) {
				*row -= 1;
				*col -= 1;
			}
		}
		'u' => {
			if allow_off_screen || ((*row > MIN_ROW as i16) & &(*col < (DCOLS - 1) as i16)) {
				*row -= 1;
				*col += 1;
			}
		}
		'n' => {
			if allow_off_screen || ((*row < (DROWS - 2) as i16) && (*col < (DCOLS - 1) as i16)) {
				*row += 1;
				*col += 1;
			}
		}
		'b' => {
			if allow_off_screen || ((*row < (DROWS - 2) as i16) && (*col > 0)) {
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
