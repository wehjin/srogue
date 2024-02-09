#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{chtype, mvaddch, refresh, standend, standout};
use crate::level::constants::{DCOLS, DROWS};
use crate::monster::Monster;
use crate::prelude::*;
use crate::prelude::armor_kind::LEATHER;
use crate::prelude::ending::Ending;
use crate::prelude::item_usage::BEING_USED;
use crate::prelude::object_what::ObjectWhat::{Gold, Weapon};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_GOLD, STAT_HP, STAT_STRENGTH};

pub static mut less_hp: isize = 0;
pub static FLAME_NAME: &'static str = "flame";

pub unsafe fn special_hit(monster: &mut Monster, depth: &RogueDepth, level: &mut Level) {
	if monster.m_flags.confused && rand_percent(66) {
		return;
	}
	if monster.m_flags.rusts {
		rust(Some(monster), depth.cur);
	}
	if monster.m_flags.holds && levitate == 0 {
		level.being_held = true;
	}
	if monster.m_flags.freezes {
		freeze(monster, depth, level);
	}
	if monster.m_flags.stings {
		sting(monster, depth.cur, level);
	}
	if monster.m_flags.drains_life {
		drain_life(depth.cur);
	}
	if monster.m_flags.drops_level {
		drop_level(depth.cur);
	}
	if monster.m_flags.steals_gold {
		steal_gold(monster, depth.cur, level);
	} else if monster.m_flags.steals_item {
		steal_item(monster, depth, level);
	}
}

pub unsafe fn rust(monster: Option<&mut Monster>, level_depth: usize) {
	if rogue.armor.is_null() || (get_armor_class(&*rogue.armor) <= 1) || ((*rogue.armor).which_kind == LEATHER) {
		return;
	}
	if ((*rogue.armor).is_protected != 0) || maintain_armor {
		if let Some(monster) = monster {
			if !monster.m_flags.rust_vanished {
				message("the rust vanishes instantly", 0);
				monster.m_flags.rust_vanished = true;
			}
		}
	} else {
		(*rogue.armor).d_enchant -= 1;
		message("your armor weakens", 0);
		print_stats(STAT_ARMOR, level_depth);
	}
}

unsafe fn freeze(monster: &mut Monster, depth: &RogueDepth, level: &mut Level) {
	if rand_percent(12) {
		return;
	}
	let mut freeze_percent: isize = 99;
	freeze_percent -= rogue.str_current + (rogue.str_current / 2);
	freeze_percent -= (rogue.exp + ring_exp) * 4;
	freeze_percent -= get_armor_class(rogue.armor) * 5;
	freeze_percent -= rogue.hp_max / 3;
	if freeze_percent > 10 {
		monster.m_flags.freezing_rogue = true;
		message("you are frozen", 1);

		let n = get_rand(4, 8);
		for _ in 0..n {
			mv_mons(depth, level);
		}
		if rand_percent(freeze_percent as usize) {
			for _ in 0..50 {
				mv_mons(depth, level);
			}
			killed_by(Ending::Hypothermia, depth.max);
		}
		message(YOU_CAN_MOVE_AGAIN, 1);
		monster.m_flags.freezing_rogue = false;
	}
}

unsafe fn steal_gold(monster: &mut Monster, level_depth: usize, level: &mut Level) {
	if rogue.gold <= 0 || rand_percent(10) {
		return;
	}

	let amount = get_rand(level_depth * 10, level_depth * 30).min(rogue.gold);
	rogue.gold -= amount;
	message("your purse feels lighter", 0);
	print_stats(STAT_GOLD, level_depth);
	disappear(monster, level);
}

unsafe fn steal_item(monster: &mut Monster, depth: &RogueDepth, level: &mut Level) {
	if rand_percent(15) {
		return;
	}
	let mut obj = rogue.pack.next_object;
	if obj.is_null() {
		disappear(monster, level);
		return;
	}
	let mut has_something = false;
	while !obj.is_null() {
		if ((*obj).in_use_flags & BEING_USED) == 0 {
			has_something = true;
			break;
		}
		obj = (*obj).next_object;
	}
	if !has_something {
		disappear(monster, level);
		return;
	}

	obj = rogue.pack.next_object;
	{
		let n = get_rand(0, MAX_PACK_COUNT);
		for _ in 0..=n {
			obj = (*obj).next_object;
			while obj.is_null() || ((*obj).in_use_flags & BEING_USED) != 0 {
				if obj.is_null() {
					obj = rogue.pack.next_object;
				} else {
					obj = (*obj).next_object;
				}
			}
		}
	}

	let msg = {
		let obj_quantity = (*obj).quantity;
		if (*obj).what_is != Weapon {
			(*obj).quantity = 1;
		}
		let msg = format!("she stole {}", get_desc(&*obj));
		(*obj).quantity = obj_quantity;
		msg
	};
	message(&msg, 0);

	vanish(&mut *obj, false, &mut rogue.pack, depth, level);
	disappear(monster, level);
}

unsafe fn disappear(monster: &mut Monster, level: &mut Level) {
	level.dungeon[monster.spot.row as usize][monster.spot.col as usize].remove_kind(CellKind::Monster);
	if rogue_can_see(monster.spot.row, monster.spot.col, level) {
		let dungeon_char = get_dungeon_char(monster.spot.row, monster.spot.col, level);
		mvaddch(monster.spot.row as i32, monster.spot.col as i32, dungeon_char);
	}
	MASH.remove_monster(monster.id());
	mon_disappeared = true;
}


pub unsafe fn cough_up(monster: &mut Monster, depth: &RogueDepth, level: &mut Level) {
	if depth.cur < depth.max {
		return;
	}
	let obj = if monster.m_flags.steals_gold {
		let obj = alloc_object();
		(*obj).what_is = Gold;
		(*obj).quantity = get_rand((depth.cur * 15) as i16, (depth.cur * 30) as i16);
		obj
	} else {
		if !rand_percent(monster.drop_percent) {
			return;
		}
		gr_object(depth.cur)
	};
	let row = monster.spot.row;
	let col = monster.spot.col;
	for n in 0..=5 {
		for i in -n..=n {
			let cough_col = col + i;
			if try_to_cough(row + n, cough_col, &mut *obj, level) {
				return;
			}
			if try_to_cough(row - n, cough_col, &mut *obj, level) {
				return;
			}
		}
		for i in -n..=n {
			let cough_row = row + i;
			if try_to_cough(cough_row, col - n, &mut *obj, level) {
				return;
			}
			if try_to_cough(cough_row, col + n, &mut *obj, level) {
				return;
			}
		}
	}
	free_object(obj);
}

unsafe fn try_to_cough(row: i64, col: i64, obj: &mut obj, level: &mut Level) -> bool {
	if row < MIN_ROW || row > (DROWS - 2) as i64 || col < 0 || col > (DCOLS - 1) as i64 {
		return false;
	}
	let dungeon_cell = level.dungeon[row as usize][col as usize];
	if !dungeon_cell.is_any_kind(&[CellKind::Object, CellKind::Stairs, CellKind::Trap])
		&& dungeon_cell.is_any_kind(&[CellKind::Tunnel, CellKind::Floor, CellKind::Door]) {
		place_at(obj, row, col, level);
		if (row != rogue.row || col != rogue.col)
			&& !dungeon_cell.is_monster() {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col, level));
		}
		return true;
	}
	return false;
}

pub unsafe fn seek_gold(monster: &mut Monster, depth: &RogueDepth, level: &mut Level) -> bool {
	let rn = get_room_number(monster.spot.row, monster.spot.col, level);
	if rn < 0 {
		return false;
	}

	let rn = rn as usize;
	for i in (level.rooms[rn].top_row + 1)..level.rooms[rn].bottom_row {
		for j in (level.rooms[rn].left_col + 1)..level.rooms[rn].right_col {
			if gold_at(i, j, level) && !level.dungeon[i as usize][j as usize].is_monster() {
				monster.m_flags.can_flit = true;
				let can_go_if_while_can_flit = mon_can_go(monster, i, j, level);
				monster.m_flags.can_flit = false;
				if can_go_if_while_can_flit {
					move_mon_to(monster, i, j, level);
					monster.m_flags.asleep = true;
					monster.m_flags.wakens = false;
					monster.m_flags.seeks_gold = false;
					return true;
				}
				monster.m_flags.seeks_gold = false;
				monster.m_flags.can_flit = true;
				mv_monster(monster, i, j, depth, level);
				monster.m_flags.can_flit = false;
				monster.m_flags.seeks_gold = true;
				return true;
			}
		}
	}
	return false;
}

unsafe fn gold_at(row: i64, col: i64, level: &Level) -> bool {
	if level.dungeon[row as usize][col as usize].is_object() {
		let obj = object_at(&mut level_objects, row, col);
		if !obj.is_null() && (*obj).what_is == Gold {
			return true;
		}
	}
	return false;
}

pub fn clear_gold_seeker(monster: &mut Monster) {
	monster.m_flags.seeks_gold = false;
}

pub unsafe fn check_imitator(monster: &mut Monster, level: &Level) -> bool {
	if monster.m_flags.imitates {
		monster.wake_up();
		if blind == 0 {
			mvaddch(monster.spot.row as i32, monster.spot.col as i32, get_dungeon_char(monster.spot.row, monster.spot.col, level));
			check_message();
			let msg = format!("wait, that's a {}!", mon_name(monster, level));
			message(&msg, 1);
		}
		return true;
	}
	return false;
}

pub unsafe fn imitating(row: i64, col: i64, level: &Level) -> bool {
	if level.dungeon[row as usize][col as usize].is_monster() {
		if let Some(monster) = MASH.monster_at_spot(row, col) {
			if monster.m_flags.imitates {
				return true;
			}
		}
	}
	return false;
}

unsafe fn sting(monster: &Monster, level_depth: usize, level: &Level) {
	if rogue.str_current <= 3 || sustain_strength {
		return;
	}

	let mut sting_chance: isize = 35;
	sting_chance += 6 * (6 - get_armor_class(rogue.armor));

	if (rogue.exp + ring_exp) > 8 {
		sting_chance -= 6 * ((rogue.exp + ring_exp) - 8);
	}
	if rand_percent(sting_chance as usize) {
		message(&format!("the {}'s bite has weakened you", mon_name(monster, level)), 0);
		rogue.str_current -= 1;
		print_stats(STAT_STRENGTH, level_depth);
	}
}

unsafe fn drop_level(level_depth: usize) {
	if rand_percent(80) || rogue.exp <= 5 {
		return;
	}

	rogue.exp_points = LEVEL_POINTS[rogue.exp as usize - 2] - get_rand(9, 29);
	rogue.exp -= 2;

	let hp = hp_raise();
	rogue.hp_current -= hp;
	if rogue.hp_current <= 0 {
		rogue.hp_current = 1;
	}
	rogue.hp_max -= hp;
	if rogue.hp_max <= 0 {
		rogue.hp_max = 1;
	}
	add_exp(1, false, level_depth);
}

unsafe fn drain_life(level_depth: usize) {
	if rand_percent(60) || rogue.hp_max <= 30 || rogue.hp_current < 10 {
		return;
	}

	let n = get_rand(1, 3);             /* 1 Hp, 2 Str, 3 both */
	if n != 2 || !sustain_strength {
		message("you feel weaker", 0);
	}
	if n != 2 {
		rogue.hp_max -= 1;
		rogue.hp_current -= 1;
		less_hp += 1;
	}
	if n != 1 {
		if rogue.str_current > 3 && !sustain_strength {
			rogue.str_current -= 1;
			if coin_toss() {
				rogue.str_max -= 1;
			}
		}
	}
	print_stats(STAT_STRENGTH | STAT_HP, level_depth);
}

pub unsafe fn m_confuse(monster: &mut Monster, level: &Level) -> bool {
	if !rogue_can_see(monster.spot.row, monster.spot.col, level) {
		return false;
	}
	if rand_percent(45) {
		/* will not confuse the rogue */
		monster.m_flags.confuses = false;
		return false;
	}
	if rand_percent(55) {
		monster.m_flags.confuses = false;
		let msg = format!("the gaze of the {} has confused you", mon_name(monster, level));
		message(&msg, 1);
		confuse();
		return true;
	}
	return false;
}

pub unsafe fn flame_broil(monster: &mut Monster, depth: &RogueDepth, level: &mut Level) -> bool {
	if !mon_sees(monster, rogue.row, rogue.col, level) || coin_toss() {
		return false;
	}
	{
		let mut delta_row = rogue.row - monster.spot.row;
		let mut delta_col = rogue.col - monster.spot.col;
		if delta_row < 0 {
			delta_row = -delta_row;
		}
		if delta_col < 0 {
			delta_col = -delta_col;
		}
		if delta_row != 0 && delta_col != 0 && delta_row != delta_col || (delta_row > 7 || delta_col > 7) {
			return false;
		}
	}
	if blind == 0 && !rogue_is_around(monster.spot.row, monster.spot.col) {
		let mut row = monster.spot.row;
		let mut col = monster.spot.col;
		get_closer(&mut row, &mut col, rogue.row, rogue.col);
		standout();
		loop {
			mvaddch(row as i32, col as i32, chtype::from('~'));
			refresh();
			get_closer(&mut row, &mut col, rogue.row, rogue.col);
			let stay_looping = row != rogue.row || col != rogue.col;
			if !stay_looping {
				break;
			}
		}
		standend();
		row = monster.spot.row;
		col = monster.spot.col;
		get_closer(&mut row, &mut col, rogue.row, rogue.col);
		loop {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col, level));
			refresh();
			get_closer(&mut row, &mut col, rogue.row, rogue.col);
			let stay_looping = row != rogue.row || col != rogue.col;
			if !stay_looping {
				break;
			}
		}
	}
	mon_hit(monster, Some(FLAME_NAME), true, depth, level);
	return true;
}

fn get_closer(row: &mut i64, col: &mut i64, trow: i64, tcol: i64) {
	if *row < trow {
		*row += 1;
	} else if *row > trow {
		*row -= 1;
	}
	if *col < tcol {
		*col += 1;
	} else if *col > tcol {
		*col -= 1;
	}
}