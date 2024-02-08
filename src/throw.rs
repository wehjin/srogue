#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{chtype, mvaddch, mvinch, refresh};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::monster;
use crate::prelude::*;
use crate::prelude::item_usage::{BEING_USED, BEING_WIELDED, BEING_WORN, NOT_USED, ON_EITHER_HAND};
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Weapons;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Trap, Tunnel, VertWall};
use crate::prelude::stat_const::STAT_ARMOR;
use crate::prelude::weapon_kind::{ARROW, BOW, DAGGER, DART, SHURIKEN};
use crate::throw::Move::{Up, UpLeft, UpRight, Left, Right, Same, Down, DownLeft, DownRight};

pub unsafe fn throw(depth: &RogueDepth, level: &Level) {
	let dir = get_dir_or_cancel();
	check_message();
	if dir == CANCEL {
		return;
	}
	let wch = pack_letter("throw what?", Weapons);
	if wch == CANCEL {
		return;
	}
	check_message();

	let weapon = get_letter_object(wch);
	if weapon.is_null() {
		message("no such item.", 0);
		return;
	}
	if ((*weapon).in_use_flags & BEING_USED) != 0 && (*weapon).is_cursed != 0 {
		message(CURSE_MESSAGE, 0);
		return;
	}
	let mut row = rogue.row;
	let mut col = rogue.col;
	if ((*weapon).in_use_flags & BEING_WIELDED) != 0 && (*weapon).quantity <= 1 {
		unwield(rogue.weapon);
	} else if ((*weapon).in_use_flags & BEING_WORN) != 0 {
		mv_aquatars(depth, level);
		unwear(rogue.armor);
		print_stats(STAT_ARMOR, depth.cur);
	} else if ((*weapon).in_use_flags & ON_EITHER_HAND) != 0 {
		un_put_on(weapon, depth.cur, level);
	}
	let monster_id = get_thrown_at_monster(weapon, dir, &mut row, &mut col, level);
	mvaddch(rogue.row as i32, rogue.col as i32, chtype::from(rogue.fchar));
	refresh();

	if rogue_can_see(row, col, level) && (row != rogue.row || col != rogue.col) {
		mvaddch(row as i32, col as i32, get_dungeon_char(row, col));
	}
	if let Some(monster_id) = monster_id {
		let monster = MASH.monster_with_id_mut(monster_id).expect("monster with id");
		monster.wake_up();
		clear_gold_seeker(&mut *monster);
		if !throw_at_monster(&mut *monster, &mut *weapon, depth, level) {
			flop_weapon(&mut *weapon, row, col, level);
		}
	} else {
		flop_weapon(&mut *weapon, row, col, level);
	}
	vanish(&mut *weapon, true, &mut rogue.pack, depth, level);
}

unsafe fn throw_at_monster(monster: &mut monster::Monster, weapon: &mut obj, depth: &RogueDepth, level: &Level) -> bool {
	let mut hit_chance = get_hit_chance(weapon);
	let mut damage = get_weapon_damage(weapon);
	if weapon.which_kind == ARROW && rogue_weapon_is_bow() {
		damage += get_weapon_damage(&*rogue.weapon);
		damage = (damage * 2) / 3;
		hit_chance += hit_chance / 3;
	} else if (weapon.in_use_flags & BEING_WIELDED) != 0
		&& (weapon.which_kind == DAGGER || weapon.which_kind == SHURIKEN || weapon.which_kind == DART) {
		damage = (damage * 3) / 2;
		hit_chance += hit_chance / 3;
	}

	let t = weapon.quantity;
	weapon.quantity = 1;
	hit_message = format!("the {}", name_of(weapon));
	weapon.quantity = t;

	if !rand_percent(hit_chance) {
		hit_message += "misses  ";
		return false;
	}
	hit_message += "hit  ";
	if weapon.what_is == Wand && rand_percent(75) {
		zap_monster(monster, weapon.which_kind, depth, level);
	} else {
		mon_damage(monster, damage as usize, depth);
	}
	return true;
}

unsafe fn rogue_weapon_is_bow() -> bool {
	!rogue.weapon.is_null() && (*rogue.weapon).which_kind == BOW
}


pub unsafe fn get_thrown_at_monster(obj: *mut object, dir: char, row: &mut i64, col: &mut i64, level: &Level) -> Option<u64> {
	let mut orow = *row;
	let mut ocol = *col;
	let ch = get_mask_char((*obj).what_is);
	for mut i in 0..24 {
		get_dir_rc(dir, row, col, false);
		if SpotFlag::is_nothing(DUNGEON[*row as usize][*col as usize])
			|| (SpotFlag::is_any_set(&vec![HorWall, VertWall, Hidden], DUNGEON[*row as usize][*col as usize]) && !Trap.is_set(DUNGEON[*row as usize][*col as usize])) {
			*row = orow;
			*col = ocol;
			return None;
		}
		if i != 0 && rogue_can_see(orow, ocol, level) {
			mvaddch(orow as i32, ocol as i32, get_dungeon_char(orow, ocol));
		}
		if rogue_can_see(*row, *col, level) {
			if !Monster.is_set(DUNGEON[*row as usize][*col as usize]) {
				mvaddch(*row as i32, *col as i32, chtype::from(ch));
			}
			refresh();
		}
		orow = *row;
		ocol = *col;
		if Monster.is_set(DUNGEON[*row as usize][*col as usize]) {
			if !imitating(*row, *col) {
				return MASH.monster_at_spot(*row, *col).map(|m| m.id());
			}
		}
		if Tunnel.is_set(DUNGEON[*row as usize][*col as usize]) {
			i += 2;
		}
	}
	return None;
}

unsafe fn flop_weapon(weapon: &mut obj, mut row: i64, mut col: i64, level: &Level) {
	let mut found = false;
	let mut walk = RandomWalk::new(row, col);
	for _ in 0..9 {
		if SpotFlag::are_others_set(&vec![Floor, Tunnel, Door, Monster], DUNGEON[walk.row as usize][walk.col as usize]) {
			break;
		}
		walk.step();
		let spot = walk.to_spot();
		if spot.is_out_of_bounds()
			|| SpotFlag::is_nothing(DUNGEON[spot.row as usize][spot.col as usize])
			|| SpotFlag::are_others_set(&vec![Floor, Tunnel, Door, Monster], DUNGEON[spot.row as usize][spot.col as usize]) {
			continue;
		}
		row = spot.row;
		col = spot.col;
		found = true;
		break;
	}

	if found || walk.index == 0 {
		let new_weapon = alloc_object();
		*new_weapon = weapon.clone();
		(*new_weapon).in_use_flags = NOT_USED;
		(*new_weapon).quantity = 1;
		(*new_weapon).ichar = 'L';
		place_at(&mut *new_weapon, row, col);
		if rogue_can_see(row, col, level) && (row != rogue.row || col != rogue.col) {
			let mon = Monster.is_set(DUNGEON[row as usize][col as usize]);
			Monster.clear(&mut DUNGEON[row as usize][col as usize]);
			let dch = get_dungeon_char(row, col);
			if mon {
				let mch = mvinch(row as i32, col as i32) as u8 as char;
				if let Some(monster) = MASH.monster_at_spot_mut(row, col) {
					monster.trail_char = dch;
				}
				if (mch < 'A') || (mch > 'Z') {
					mvaddch(row as i32, col as i32, dch);
				}
			} else {
				mvaddch(row as i32, col as i32, dch);
			}
			Monster.set(&mut DUNGEON[row as usize][col as usize]);
		}
	} else {
		let t = weapon.quantity;
		weapon.quantity = 1;
		let msg = format!("the {}vanishes as it hits the ground", name_of(weapon));
		weapon.quantity = t;
		message(&msg, 0);
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Move {
	DownRight,
	DownLeft,
	UpRight,
	UpLeft,
	Right,
	Down,
	Same,
	Up,
	Left,
}

impl Move {
	pub fn delta(&self) -> (isize, isize) {
		match self {
			DownRight => (1, 1),
			DownLeft => (1, -1),
			UpRight => (-1, 1),
			UpLeft => (-1, -1),
			Right => (0, 1),
			Down => (1, 0),
			Same => (0, 0),
			Up => (-1, 0),
			Left => (0, -1),
		}
	}
	pub fn random8() -> Self {
		match get_rand(1, 8) {
			1 => Up,
			2 => Down,
			3 => Right,
			4 => Left,
			5 => UpLeft,
			6 => UpRight,
			7 => DownLeft,
			8 => DownRight,
			_ => unreachable!("out of bounds")
		}
	}

	pub fn to_char(&self) -> char {
		match self {
			DownRight => 'n',
			DownLeft => 'b',
			UpRight => 'u',
			UpLeft => 'y',
			Right => 'l',
			Down => 'k',
			Same => ' ',
			Up => 'j',
			Left => 'h',
		}
	}
	pub fn from_char(ch: char) -> Self {
		match ch {
			'n' => DownRight,
			'b' => DownLeft,
			'u' => UpRight,
			'y' => UpLeft,
			'l' => Right,
			'k' => Down,
			'j' => Up,
			'h' => Left,
			_ => Same,
		}
	}

	pub fn apply(&self, row: i64, col: i64) -> (i64, i64) {
		let (r_delta, c_delta) = self.delta();
		(row + r_delta as i64, col + c_delta as i64)
	}
}

pub struct RandomWalk {
	pub row: i64,
	pub col: i64,
	moves: [Move; 9],
	pub index: usize,
}

impl RandomWalk {
	pub fn new(row: i64, col: i64) -> Self {
		let mut moves: [Move; 9] = [Left, Up, DownLeft, UpLeft, Right, Down, UpRight, Same, DownRight];
		moves.shuffle(&mut thread_rng());
		RandomWalk { row, col, moves, index: 0 }
	}
	pub fn step(&mut self) {
		if self.index < self.moves.len() {
			let (row, col) = self.moves[self.index].apply(self.row, self.col);
			self.row = row;
			self.col = col;
			self.index += 1;
		}
	}
	pub fn to_spot(&self) -> DungeonSpot {
		DungeonSpot { row: self.row, col: self.col }
	}
}
