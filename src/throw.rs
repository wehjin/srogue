#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{chtype, mvaddch, mvinch, refresh};
use crate::prelude::*;
use crate::prelude::item_usage::{BEING_USED, BEING_WIELDED, BEING_WORN, NOT_USED, ON_EITHER_HAND};
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Weapons;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Trap, Tunnel, VertWall};
use crate::prelude::stat_const::STAT_ARMOR;
use crate::prelude::weapon_kind::{ARROW, BOW, DAGGER, DART, SHURIKEN};
use crate::throw::Move::{Up, UpLeft, UpRight, Left, Right, Same, Down, DownLeft, DownRight};

pub unsafe fn throw() {
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
		mv_aquatars();
		unwear(rogue.armor);
		print_stats(STAT_ARMOR);
	} else if ((*weapon).in_use_flags & ON_EITHER_HAND) != 0 {
		un_put_on(weapon);
	}
	let monster = get_thrown_at_monster(weapon, dir, &mut row, &mut col);
	mvaddch(rogue.row as i32, rogue.col as i32, chtype::from(rogue.fchar));
	refresh();

	if rogue_can_see(row, col) && (row != rogue.row || col != rogue.col) {
		mvaddch(row as i32, col as i32, get_dungeon_char(row, col));
	}
	if !monster.is_null() {
		wake_up(&mut *monster);
		check_gold_seeker(&mut *monster);
		if !throw_at_monster(&mut *monster, &mut *weapon) {
			flop_weapon(&mut *weapon, row, col);
		}
	} else {
		flop_weapon(&mut *weapon, row, col);
	}
	vanish(&mut *weapon, true, &mut rogue.pack);
}

unsafe fn throw_at_monster(monster: &mut obj, weapon: &mut obj) -> bool {
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
		zap_monster(monster, weapon.which_kind);
	} else {
		mon_damage(monster, damage as usize);
	}
	return true;
}

unsafe fn rogue_weapon_is_bow() -> bool {
	!rogue.weapon.is_null() && (*rogue.weapon).which_kind == BOW
}


pub unsafe fn get_thrown_at_monster(obj: *mut object, dir: char, row: &mut i64, col: &mut i64) -> *mut object {
	let mut orow = *row;
	let mut ocol = *col;
	let ch = get_mask_char((*obj).what_is);
	for mut i in 0..24 {
		get_dir_rc(dir, row, col, false);
		if SpotFlag::is_empty(dungeon[*row as usize][*col as usize])
			|| (SpotFlag::is_any_set(&vec![HorWall, VertWall, Hidden], dungeon[*row as usize][*col as usize]) && !Trap.is_set(dungeon[*row as usize][*col as usize])) {
			*row = orow;
			*col = ocol;
			return 0 as *mut object;
		}
		if i != 0 && rogue_can_see(orow, ocol) {
			mvaddch(orow as i32, ocol as i32, get_dungeon_char(orow, ocol));
		}
		if rogue_can_see(*row, *col) {
			if !Monster.is_set(dungeon[*row as usize][*col as usize]) {
				mvaddch(*row as i32, *col as i32, chtype::from(ch));
			}
			refresh();
		}
		orow = *row;
		ocol = *col;
		if Monster.is_set(dungeon[*row as usize][*col as usize]) {
			if !imitating(*row, *col) {
				return object_at(&level_monsters, *row, *col);
			}
		}
		if Tunnel.is_set(dungeon[*row as usize][*col as usize]) {
			i += 2;
		}
	}
	return 0 as *mut object;
}

unsafe fn flop_weapon(weapon: &mut obj, row: i64, col: i64) {
	let mut i = 0;
	let mut found = false;
	let mut row = row;
	let mut col = col;
	while i < 9 && SpotFlag::are_others_set(&vec![Floor, Tunnel, Door, Monster], dungeon[row as usize][col as usize]) {
		let (new_row, new_col) = rand_around(i, row, col);
		i += 1;
		row = new_row;
		col = new_col;
		if row > (DROWS - 2) as i64 || row < MIN_ROW || col > (DCOLS - 1) as i64 || col < 0
			|| SpotFlag::is_empty(dungeon[row as usize][col as usize])
			|| SpotFlag::are_others_set(&vec![Floor, Tunnel, Door, Monster], dungeon[row as usize][col as usize]) {
			continue;
		}
		found = true;
		break;
	}

	if found || i == 0 {
		let new_weapon = alloc_object();
		*new_weapon = weapon.clone();
		(*new_weapon).in_use_flags = NOT_USED;
		(*new_weapon).quantity = 1;
		(*new_weapon).ichar = 'L';
		place_at(new_weapon, row, col);
		if rogue_can_see(row, col) && (row != rogue.row || col != rogue.col) {
			let mon = Monster.is_set(dungeon[row as usize][col as usize]);
			Monster.clear(&mut dungeon[row as usize][col as usize]);
			let dch = get_dungeon_char(row, col);
			if mon {
				let mch = mvinch(row as i32, col as i32) as u8 as char;
				let monster = object_at(&level_monsters, row, col);
				if !monster.is_null() {
					(*monster).set_trail_char(dch);
				}
				if (mch < 'A') || (mch > 'Z') {
					mvaddch(row as i32, col as i32, dch);
				}
			} else {
				mvaddch(row as i32, col as i32, dch);
			}
			Monster.set(&mut dungeon[row as usize][col as usize]);
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

pub unsafe fn rand_around(i: u8, r: i64, c: i64) -> (i64, i64) {
	static mut moves: [Move; 9] = [Left, Up, DownLeft, UpLeft, Right, Down, UpRight, Same, DownRight];
	static mut row: usize = 0;
	static mut col: usize = 0;

	if i == 0 {
		row = r as usize;
		col = c as usize;
		let o = get_rand(1, 8);
		for _j in 0..5 {
			let x = get_rand(0, 8) as usize;
			let y = (x + o) % 9;
			let t = moves[x];
			moves[x] = moves[y];
			moves[y] = t;
		}
	}
	moves[i as usize].apply(r, c)
}
