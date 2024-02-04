#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{chtype, mvaddch, refresh, standend, standout};
use crate::prelude::*;
use crate::prelude::armor_kind::LEATHER;
use crate::prelude::ending::Ending;
use crate::prelude::item_usage::BEING_USED;
use crate::prelude::object_what::ObjectWhat::{Gold, Weapon};
use crate::prelude::SpotFlag::{Door, Floor, Monster, Object, Stairs, Trap, Tunnel};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_GOLD, STAT_HP, STAT_STRENGTH};

pub static mut less_hp: isize = 0;
pub static FLAME_NAME: &'static str = "flame";
pub static mut being_held: bool = false;

pub unsafe fn special_hit(monster: &mut object, depth: &RogueDepth) {
	if monster.m_flags.confused && rand_percent(66) {
		return;
	}
	if monster.m_flags.rusts {
		rust(Some(monster), depth.cur);
	}
	if monster.m_flags.holds && levitate == 0 {
		being_held = true;
	}
	if monster.m_flags.freezes {
		freeze(monster, depth);
	}
	if monster.m_flags.stings {
		sting(monster, depth.cur);
	}
	if monster.m_flags.drains_life {
		drain_life(depth.cur);
	}
	if monster.m_flags.drops_level {
		drop_level(depth.cur);
	}
	if monster.m_flags.steals_gold {
		steal_gold(monster, depth.cur);
	} else if monster.m_flags.steals_item {
		steal_item(monster, depth);
	}
}

pub unsafe fn rust(monster: Option<&mut obj>, level_depth: usize) {
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

unsafe fn freeze(monster: &mut obj, depth: &RogueDepth) {
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
			mv_mons(depth);
		}
		if rand_percent(freeze_percent as usize) {
			for _ in 0..50 {
				mv_mons(depth);
			}
			killed_by(Ending::Hypothermia, depth.max);
		}
		message(YOU_CAN_MOVE_AGAIN, 1);
		monster.m_flags.freezing_rogue = false;
	}
}

unsafe fn steal_gold(monster: &mut obj, cur_level: usize) {
	if rogue.gold <= 0 || rand_percent(10) {
		return;
	}

	let amount = get_rand(cur_level * 10, cur_level * 30).min(rogue.gold);
	rogue.gold -= amount;
	message("your purse feels lighter", 0);
	print_stats(STAT_GOLD, cur_level);
	disappear(monster);
}

unsafe fn steal_item(monster: &mut obj, depth: &RogueDepth) {
	if rand_percent(15) {
		return;
	}
	let mut obj = rogue.pack.next_object;
	if obj.is_null() {
		disappear(monster);
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
		disappear(monster);
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

	vanish(&mut *obj, false, &mut rogue.pack, depth);
	disappear(monster);
}

unsafe fn disappear(monster: &mut obj) {
	let row = monster.row;
	let col = monster.col;

	Monster.clear(&mut dungeon[row as usize][col as usize]);
	if rogue_can_see(row, col) {
		mvaddch(row as i32, col as i32, get_dungeon_char(row, col));
	}
	take_from_pack(monster, &mut level_monsters);
	free_object(monster);
	mon_disappeared = true;
}


pub unsafe fn cough_up(monster: &mut obj, depth: &RogueDepth) {
	if depth.cur < depth.max {
		return;
	}
	let obj = if (*monster).m_flags.steals_gold {
		let obj = alloc_object();
		(*obj).what_is = Gold;
		(*obj).quantity = get_rand((depth.cur * 15) as i16, (depth.cur * 30) as i16);
		obj
	} else {
		if !rand_percent((*monster).drop_percent()) {
			return;
		}
		gr_object(depth.cur)
	};
	let row = (*monster).row;
	let col = (*monster).col;
	for n in 0..=5 {
		for i in -n..=n {
			let cough_col = col + i;
			if try_to_cough(row + n, cough_col, &mut *obj) {
				return;
			}
			if try_to_cough(row - n, cough_col, &mut *obj) {
				return;
			}
		}
		for i in -n..=n {
			let cough_row = row + i;
			if try_to_cough(cough_row, col - n, &mut *obj) {
				return;
			}
			if try_to_cough(cough_row, col + n, &mut *obj) {
				return;
			}
		}
	}
	free_object(obj);
}

unsafe fn try_to_cough(row: i64, col: i64, obj: &mut obj) -> bool {
	if row < MIN_ROW || row > (DROWS - 2) as i64 || col < 0 || col > (DCOLS - 1) as i64 {
		return false;
	}
	let dungeon_cell = dungeon[row as usize][col as usize];
	if !SpotFlag::is_any_set(&vec![Object, Stairs, Trap], dungeon_cell)
		&& SpotFlag::is_any_set(&vec![Tunnel, Floor, Door], dungeon_cell) {
		place_at(obj, row, col);
		let no_rogue = row != rogue.row || col != rogue.col;
		let no_monster = !Monster.is_set(dungeon_cell);
		if no_rogue && no_monster {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col));
		}
		return true;
	}
	return false;
}

pub unsafe fn seek_gold(monster: &mut obj, depth: &RogueDepth) -> bool {
	let rn = get_room_number(monster.row, monster.col);
	if rn < 0 {
		return false;
	}

	let rn = rn as usize;
	for i in (ROOMS[rn].top_row + 1)..ROOMS[rn].bottom_row {
		for j in (ROOMS[rn].left_col + 1)..ROOMS[rn].right_col {
			if gold_at(i, j) && !Monster.is_set(dungeon[i as usize][j as usize]) {
				monster.m_flags.can_flit = true;
				let can_go_if_while_can_flit = mon_can_go(monster, i, j);
				monster.m_flags.can_flit = false;
				if can_go_if_while_can_flit {
					move_mon_to(monster, i, j);
					monster.m_flags.asleep = true;
					monster.m_flags.wakens = false;
					monster.m_flags.seeks_gold = false;
					return true;
				}
				monster.m_flags.seeks_gold = false;
				monster.m_flags.can_flit = true;
				mv_monster(monster, i, j, depth);
				monster.m_flags.can_flit = false;
				monster.m_flags.seeks_gold = true;
				return true;
			}
		}
	}
	return false;
}

unsafe fn gold_at(row: i64, col: i64) -> bool {
	if Object.is_set(dungeon[row as usize][col as usize]) {
		let obj = object_at(&mut level_objects, row, col);
		if !obj.is_null() && (*obj).what_is == Gold {
			return true;
		}
	}
	return false;
}

pub fn check_gold_seeker(monster: &mut object) {
	monster.m_flags.seeks_gold = false;
}

pub unsafe fn check_imitator(monster: &mut object) -> bool {
	if monster.m_flags.imitates {
		wake_up(monster);
		if blind == 0 {
			mvaddch(monster.row as i32, monster.col as i32, get_dungeon_char((*monster).row, (*monster).col));
			check_message();
			let msg = format!("wait, that's a {}!", mon_name(monster));
			message(&msg, 1);
		}
		return true;
	}
	return false;
}

pub unsafe fn imitating(row: i64, col: i64) -> bool {
	if Monster.is_set(dungeon[row as usize][col as usize]) {
		let monster = object_at(&level_monsters, row, col);
		if !monster.is_null() {
			if (*monster).m_flags.imitates {
				return true;
			}
		}
	}
	return false;
}

unsafe fn sting(monster: &obj, level_depth: usize) {
	if rogue.str_current <= 3 || sustain_strength {
		return;
	}

	let mut sting_chance: isize = 35;
	sting_chance += 6 * (6 - get_armor_class(rogue.armor));

	if (rogue.exp + ring_exp) > 8 {
		sting_chance -= 6 * ((rogue.exp + ring_exp) - 8);
	}
	if rand_percent(sting_chance as usize) {
		message(&format!("the {}'s bite has weakened you", mon_name(monster)), 0);
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

pub unsafe fn m_confuse(monster: &mut object) -> bool {
	if !rogue_can_see((*monster).row, (*monster).col) {
		return false;
	}
	if rand_percent(45) {
		/* will not confuse the rogue */
		monster.m_flags.confuses = false;
		return false;
	}
	if rand_percent(55) {
		monster.m_flags.confuses = false;
		let msg = format!("the gaze of the {} has confused you", mon_name(monster));
		message(&msg, 1);
		confuse();
		return true;
	}
	return false;
}

pub unsafe fn flame_broil(monster: &mut object, depth: &RogueDepth) -> bool {
	if !mon_sees(monster, rogue.row, rogue.col) || coin_toss() {
		return false;
	}
	{
		let mut delta_row = rogue.row - monster.row;
		let mut delta_col = rogue.col - monster.col;
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
	if blind == 0 && !rogue_is_around(monster.row, monster.col) {
		let mut row = monster.row;
		let mut col = monster.col;
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
		row = monster.row;
		col = monster.col;
		get_closer(&mut row, &mut col, rogue.row, rogue.col);
		loop {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col));
			refresh();
			get_closer(&mut row, &mut col, rogue.row, rogue.col);
			let stay_looping = row != rogue.row || col != rogue.col;
			if !stay_looping {
				break;
			}
		}
	}
	mon_hit(monster, Some(FLAME_NAME), true, depth);
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