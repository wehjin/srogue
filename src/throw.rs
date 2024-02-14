#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch, mvinch, refresh};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::hit::{get_dir_rc, get_hit_chance, get_weapon_damage, HIT_MESSAGE, mon_damage};
use crate::level::{CellKind, Level};
use crate::message::{CANCEL, check_message, message, print_stats};
use crate::monster::{MASH, Monster, mv_aquatars};
use crate::objects::{obj, ObjectId, place_at};
use crate::pack::{CURSE_MESSAGE, pack_letter, unwear, unwield};
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::item_usage::NOT_USED;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Weapons;
use crate::prelude::stat_const::STAT_ARMOR;
use crate::r#move::get_dir_or_cancel;
use crate::r#use::vanish;
use crate::random::{get_rand, rand_percent};
use crate::ring::un_put_hand;
use crate::room::{get_dungeon_char, get_mask_char};
use crate::spec_hit::{clear_gold_seeker, imitating};
use crate::throw::Move::{Down, DownLeft, DownRight, Left, Right, Same, Up, UpLeft, UpRight};
use crate::weapons::constants::ARROW;
use crate::weapons::kind::WeaponKind;
use crate::zap::zap_monster;

pub unsafe fn throw(player: &mut Player, level: &mut Level) {
	let dir = get_dir_or_cancel();
	check_message();
	if dir == CANCEL {
		return;
	}
	let wch = pack_letter("throw what?", Weapons, player);
	if wch == CANCEL {
		return;
	}
	check_message();
	match player.object_id_with_letter(wch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			if player.check_object(obj_id, |it| it.is_being_used() && it.is_cursed()) {
				message(CURSE_MESSAGE, 0);
				return;
			}
			if player.check_object(obj_id, |it| it.is_being_wielded() && it.quantity <= 1) {
				unwield(player);
			} else if player.check_object(obj_id, |it| it.is_being_worn()) {
				mv_aquatars(player, level);
				unwear(player);
				print_stats(STAT_ARMOR, player);
			} else if let Some(hand) = player.ring_hand(obj_id) {
				un_put_hand(hand, player, level);
			}

			let obj_what = player.object_what(obj_id);
			let rogue_spot = player.to_spot();
			let rogue_char = player.to_curses_char();
			let (monster_id, spot) = {
				let mut row = rogue_spot.row;
				let mut col = rogue_spot.col;
				let monster_id = get_thrown_at_monster(obj_what, dir, &mut row, &mut col, player, level);
				(monster_id, DungeonSpot { row, col })
			};
			mvaddch(rogue_spot.row as i32, rogue_spot.col as i32, rogue_char);
			refresh();

			let row = spot.row;
			let col = spot.col;
			if player.can_see(row, col, level)
				&& !(spot == rogue_spot) {
				mvaddch(spot.row as i32, spot.col as i32, get_dungeon_char(spot.row, spot.col, player, level));
			}
			if let Some(monster_id) = monster_id {
				let monster = MASH.monster_with_id_mut(monster_id).expect("monster with id");
				monster.wake_up();
				clear_gold_seeker(monster);
				if !throw_at_monster(monster, obj_id, player, level) {
					flop_weapon(obj_id, spot.row, spot.col, player, level);
				}
			} else {
				flop_weapon(obj_id, spot.row, spot.col, player, level);
			}
			vanish(obj_id, true, player, level);
		}
	}
}

unsafe fn throw_at_monster(monster: &mut Monster, obj_id: ObjectId, player: &mut Player, level: &mut Level) -> bool {
	let hit_chance = {
		let player_exp = player.buffed_exp();
		let player_debuf = player.debuf_exp();
		let player_weapon_is_bow = rogue_weapon_is_bow(player);

		let obj = player.object(obj_id).expect("obj in pack");
		let mut hit_chance = get_hit_chance(Some(obj), player_exp, player_debuf);
		if obj.which_kind == ARROW && player_weapon_is_bow {
			hit_chance += hit_chance / 3;
		} else if obj.is_wielded_throwing_weapon() {
			hit_chance += hit_chance / 3;
		}
		hit_chance
	};
	HIT_MESSAGE = format!("the {}", player.to_object_name_with_quantity(obj_id, 1).trim());
	if !rand_percent(hit_chance) {
		HIT_MESSAGE += " misses  ";
		return false;
	}

	HIT_MESSAGE += " hit  ";
	if player.object_what(obj_id) == Wand && rand_percent(75) {
		zap_monster(monster, player.object_kind(obj_id), player, level);
	} else {
		let player_str = player.buffed_strength();
		let player_exp = player.buffed_exp();
		let player_debuf = player.debuf_exp();
		let damage = {
			let mut damage = get_weapon_damage(player.object(obj_id), player_str, player_exp, player_debuf);
			if player.object_kind(obj_id) == ARROW && rogue_weapon_is_bow(player) {
				damage += get_weapon_damage(player.weapon(), player_str, player_exp, player_debuf);
				damage = (damage * 2) / 3;
			} else if player.check_object(obj_id, obj::is_wielded_throwing_weapon) {
				damage = (damage * 3) / 2;
			}
			damage
		};
		mon_damage(monster, damage, player, level);
	}
	return true;
}

fn rogue_weapon_is_bow(player: &Player) -> bool {
	player.weapon_kind() == Some(WeaponKind::Bow)
}


pub unsafe fn get_thrown_at_monster(obj_what: ObjectWhat, dir: char, row: &mut i64, col: &mut i64, player: &Player, level: &Level) -> Option<u64> {
	let mut orow = *row;
	let mut ocol = *col;
	let obj_char = get_mask_char(obj_what);
	let mut i = 0;
	while i < 24 {
		get_dir_rc(dir, row, col, false);
		const WALL_OR_HIDDEN: [CellKind; 3] = [CellKind::HorizontalWall, CellKind::VerticalWall, CellKind::Hidden];
		let cell = &level.dungeon[*row as usize][*col as usize];
		if cell.is_nothing()
			|| (cell.is_any_kind(&WALL_OR_HIDDEN) && !cell.is_trap()) {
			*row = orow;
			*col = ocol;
			return None;
		}

		if i != 0 && player.can_see(orow, ocol, level) {
			mvaddch(orow as i32, ocol as i32, get_dungeon_char(orow, ocol, player, level));
		}
		if player.can_see(*row, *col, level) {
			if !cell.is_monster() {
				mvaddch(*row as i32, *col as i32, chtype::from(obj_char));
			}
			refresh();
		}
		if cell.is_monster() {
			if !imitating(*row, *col, level) {
				return MASH.monster_at_spot(*row, *col).map(|m| m.id());
			}
		}
		if cell.is_tunnel() {
			i += 2;
		}
		orow = *row;
		ocol = *col;
		i += 1;
	}
	return None;
}

unsafe fn flop_weapon(obj_id: ObjectId, row: i64, col: i64, player: &Player, level: &mut Level) {
	let mut found = false;
	let mut walk = RandomWalk::new(row, col);
	for _ in 0..9 {
		const GOOD_CELL_KINDS: [CellKind; 4] = [CellKind::Floor, CellKind::Tunnel, CellKind::Door, CellKind::Monster];
		if level.dungeon[walk.spot().row as usize][walk.spot().col as usize].is_other_kind(&GOOD_CELL_KINDS) {
			break;
		}
		walk.step();
		let spot = walk.spot();
		if spot.is_out_of_bounds()
			|| level.dungeon[spot.row as usize][spot.col as usize].is_nothing()
			|| level.dungeon[spot.row as usize][spot.col as usize].is_other_kind(&GOOD_CELL_KINDS) {
			continue;
		}
		found = true;
		break;
	}
	let DungeonSpot { row, col } = walk.spot().clone();
	if found || walk.steps_taken == 0 {
		let obj = player.object(obj_id).expect("obj in pack");
		let mut new_obj = obj.clone_with_new_id();
		new_obj.in_use_flags = NOT_USED;
		new_obj.quantity = 1;
		new_obj.ichar = 'L';
		place_at(new_obj, row, col, level);
		if player.can_see(row, col, level) && !player.is_at(row, col) {
			let was_monster = level.dungeon[row as usize][col as usize].is_monster();
			level.dungeon[row as usize][col as usize].remove_kind(CellKind::Monster);
			let dungeon_char = get_dungeon_char(row, col, player, level);
			if was_monster {
				let monster_char = mvinch(row as i32, col as i32) as u8 as char;
				if let Some(monster) = MASH.monster_at_spot_mut(row, col) {
					monster.trail_char = dungeon_char;
				}
				if (monster_char < 'A') || (monster_char > 'Z') {
					mvaddch(row as i32, col as i32, dungeon_char);
				}
				level.dungeon[row as usize][col as usize].add_kind(CellKind::Monster)
			} else {
				mvaddch(row as i32, col as i32, dungeon_char);
			}
		}
	} else {
		let obj_name = player.to_object_name_with_quantity(obj_id, 1);
		let msg = format!("the {}vanishes as it hits the ground", obj_name);
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
	moves: [Move; 9],
	spot: DungeonSpot,
	pub steps_taken: usize,
}

impl RandomWalk {
	pub fn new(row: i64, col: i64) -> Self {
		let mut moves: [Move; 9] = [Left, Up, DownLeft, UpLeft, Right, Down, UpRight, Same, DownRight];
		moves.shuffle(&mut thread_rng());
		RandomWalk { spot: DungeonSpot { row, col }, moves, steps_taken: 0 }
	}
	pub fn step(&mut self) {
		if self.steps_taken < self.moves.len() {
			let (row, col) = self.moves[self.steps_taken].apply(self.spot.row, self.spot.col);
			self.spot.row = row;
			self.spot.col = col;
			self.steps_taken += 1;
		}
	}
	pub fn spot(&self) -> &DungeonSpot { &self.spot }
}
