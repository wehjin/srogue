#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch, refresh, standend, standout};
use crate::armors::ArmorKind;
use crate::hit::mon_hit;
use crate::inventory::get_obj_desc;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::{add_exp, CellKind, hp_raise, Level, LEVEL_POINTS};
use crate::message::{check_message, message, print_stats};
use crate::monster::{MASH, mon_can_go, mon_disappeared, mon_name, mon_sees, Monster, move_mon_to, mv_mons, mv_monster, rogue_can_see, rogue_is_around};
use crate::objects::{alloc_object, get_armor_class, gr_object, level_objects, obj, place_at};
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::{Gold, Weapon};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_GOLD, STAT_HP, STAT_STRENGTH};
use crate::r#move::YOU_CAN_MOVE_AGAIN;
use crate::r#use::{confuse, levitate, vanish};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::{get_dungeon_char, get_room_number};
use crate::score::killed_by;


pub static mut less_hp: isize = 0;
pub const FLAME_NAME: &'static str = "flame";

pub unsafe fn special_hit(monster: &mut Monster, player: &mut Player, level: &mut Level) {
	if monster.m_flags.confused && rand_percent(66) {
		return;
	}
	if monster.m_flags.rusts {
		rust(Some(monster), player);
	}
	if monster.m_flags.holds && levitate == 0 {
		level.being_held = true;
	}
	if monster.m_flags.freezes {
		freeze(monster, player, level);
	}
	if monster.m_flags.stings {
		sting(monster, player, level);
	}
	if monster.m_flags.drains_life {
		drain_life(player);
	}
	if monster.m_flags.drops_level {
		drop_level(player);
	}
	if monster.m_flags.steals_gold {
		steal_gold(monster, player, level);
	} else if monster.m_flags.steals_item {
		steal_item(monster, player, level);
	}
}

pub unsafe fn rust(monster: Option<&mut Monster>, player: &mut Player) {
	if player.armor().is_none()
		|| get_armor_class(player.armor()) <= 1
		|| player.armor_kind() == Some(ArmorKind::Leather) {
		return;
	}

	let player_has_maintain_armor = player.ring_effects.has_maintain_armor();

	let armor = player.armor_mut().expect("armor exists");
	if armor.is_protected != 0 || player_has_maintain_armor {
		if let Some(monster) = monster {
			if !monster.m_flags.rust_vanished {
				message("the rust vanishes instantly", 0);
				monster.m_flags.rust_vanished = true;
			}
		}
	} else {
		armor.d_enchant -= 1;
		message("your armor weakens", 0);
		print_stats(STAT_ARMOR, player);
	}
}

unsafe fn freeze(monster: &mut Monster, player: &mut Player, level: &mut Level) {
	if rand_percent(12) {
		return;
	}
	let mut freeze_percent: isize = 99;
	freeze_percent -= player.rogue.str_current + (player.rogue.str_current / 2);
	freeze_percent -= player.buffed_exp() * 4;
	freeze_percent -= get_armor_class(player.armor()) * 5;
	freeze_percent -= player.rogue.hp_max / 3;
	if freeze_percent > 10 {
		monster.m_flags.freezing_rogue = true;
		message("you are frozen", 1);

		let n = get_rand(4, 8);
		for _ in 0..n {
			mv_mons(player, level);
		}
		if rand_percent(freeze_percent as usize) {
			for _ in 0..50 {
				mv_mons(player, level);
			}
			killed_by(Ending::Hypothermia, player);
		}
		message(YOU_CAN_MOVE_AGAIN, 1);
		monster.m_flags.freezing_rogue = false;
	}
}

unsafe fn steal_gold(monster: &mut Monster, player: &mut Player, level: &mut Level) {
	if player.rogue.gold <= 0 || rand_percent(10) {
		return;
	}

	let amount = get_rand(player.cur_depth * 10, player.cur_depth * 30).min(player.rogue.gold);
	player.rogue.gold -= amount;
	message("your purse feels lighter", 0);
	print_stats(STAT_GOLD, player);
	disappear(monster, player, level);
}

unsafe fn steal_item(monster: &mut Monster, player: &mut Player, level: &mut Level) {
	if rand_percent(15) {
		return;
	}
	if player.pack().len() == 0 {
		disappear(monster, player, level);
		return;
	}
	match player.random_unused_object_id() {
		None => {
			disappear(monster, player, level);
			return;
		}
		Some(obj_id) => {
			let msg = {
				let obj_desc = {
					let obj = player.object(obj_id).expect("unused obj is in player pack");
					let mut temp_obj = obj.clone();
					if temp_obj.what_is != Weapon {
						temp_obj.quantity = 1;
					}
					get_obj_desc(&temp_obj)
				};
				format!("she stole {}", obj_desc)
			};
			message(&msg, 0);
			vanish(obj_id, false, player, level);
			disappear(monster, player, level);
		}
	}
}

unsafe fn disappear(monster: &mut Monster, player: &Player, level: &mut Level) {
	level.dungeon[monster.spot.row as usize][monster.spot.col as usize].remove_kind(CellKind::Monster);
	if rogue_can_see(monster.spot.row, monster.spot.col, player, level) {
		let dungeon_char = get_dungeon_char(monster.spot.row, monster.spot.col, player, level);
		mvaddch(monster.spot.row as i32, monster.spot.col as i32, dungeon_char);
	}
	MASH.remove_monster(monster.id());
	mon_disappeared = true;
}


pub unsafe fn cough_up(monster: &mut Monster, player: &Player, level: &mut Level) {
	if player.cur_depth < player.max_depth {
		return;
	}
	let obj = if monster.m_flags.steals_gold {
		let mut obj = alloc_object();
		obj.what_is = Gold;
		obj.quantity = get_rand((player.cur_depth * 15) as i16, (player.cur_depth * 30) as i16);
		obj
	} else {
		if !rand_percent(monster.drop_percent) {
			return;
		}
		gr_object(player.cur_depth)
	};
	let row = monster.spot.row;
	let col = monster.spot.col;
	for n in 0..=5 {
		for i in -n..=n {
			let cough_col = col + i;
			if try_to_cough(row + n, cough_col, &obj, player, level) {
				return;
			}
			if try_to_cough(row - n, cough_col, &obj, player, level) {
				return;
			}
		}
		for i in -n..=n {
			let cough_row = row + i;
			if try_to_cough(cough_row, col - n, &obj, player, level) {
				return;
			}
			if try_to_cough(cough_row, col + n, &obj, player, level) {
				return;
			}
		}
	}
}

unsafe fn try_to_cough(row: i64, col: i64, obj: &obj, player: &Player, level: &mut Level) -> bool {
	if row < MIN_ROW || row > (DROWS - 2) as i64 || col < 0 || col > (DCOLS - 1) as i64 {
		return false;
	}
	let dungeon_cell = level.dungeon[row as usize][col as usize];
	if !dungeon_cell.is_any_kind(&[CellKind::Object, CellKind::Stairs, CellKind::Trap])
		&& dungeon_cell.is_any_kind(&[CellKind::Tunnel, CellKind::Floor, CellKind::Door]) {
		place_at(obj.clone(), row, col, level);
		if (row != player.rogue.row || col != player.rogue.col)
			&& !dungeon_cell.is_monster() {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col, player, level));
		}
		return true;
	}
	return false;
}

pub unsafe fn seek_gold(monster: &mut Monster, player: &mut Player, level: &mut Level) -> bool {
	let rn = get_room_number(monster.spot.row, monster.spot.col, level);
	if rn < 0 {
		return false;
	}

	let rn = rn as usize;
	for i in (level.rooms[rn].top_row + 1)..level.rooms[rn].bottom_row {
		for j in (level.rooms[rn].left_col + 1)..level.rooms[rn].right_col {
			if gold_at(i, j, level) && !level.dungeon[i as usize][j as usize].is_monster() {
				monster.m_flags.can_flit = true;
				let can_go_if_while_can_flit = mon_can_go(monster, i, j, player, level);
				monster.m_flags.can_flit = false;
				if can_go_if_while_can_flit {
					move_mon_to(monster, i, j, player, level);
					monster.m_flags.asleep = true;
					monster.m_flags.wakens = false;
					monster.m_flags.seeks_gold = false;
					return true;
				}
				monster.m_flags.seeks_gold = false;
				monster.m_flags.can_flit = true;
				mv_monster(monster, i, j, player, level);
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
		if let Some(obj) = level_objects.find_object_at(row, col) {
			if obj.what_is == Gold {
				return true;
			}
		}
	}
	return false;
}

pub fn clear_gold_seeker(monster: &mut Monster) {
	monster.m_flags.seeks_gold = false;
}

pub unsafe fn check_imitator(monster: &mut Monster, player: &Player, level: &Level) -> bool {
	if monster.m_flags.imitates {
		monster.wake_up();
		if player.blind.is_inactive() {
			mvaddch(monster.spot.row as i32, monster.spot.col as i32, get_dungeon_char(monster.spot.row, monster.spot.col, player, level));
			check_message();
			let msg = format!("wait, that's a {}!", mon_name(monster, player, level));
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

unsafe fn sting(monster: &Monster, player: &mut Player, level: &Level) {
	if player.rogue.str_current <= 3 || player.ring_effects.has_sustain_strength() {
		return;
	}

	let mut sting_chance: isize = 35;
	sting_chance += 6 * (6 - get_armor_class(player.armor()));

	let buffed_exp = player.buffed_exp();
	if buffed_exp > 8 {
		sting_chance -= 6 * (buffed_exp - 8);
	}
	if rand_percent(sting_chance as usize) {
		message(&format!("the {}'s bite has weakened you", mon_name(monster, player, level)), 0);
		player.rogue.str_current -= 1;
		print_stats(STAT_STRENGTH, player);
	}
}

unsafe fn drop_level(player: &mut Player) {
	if rand_percent(80) || player.rogue.exp <= 5 {
		return;
	}

	player.rogue.exp_points = LEVEL_POINTS[player.rogue.exp as usize - 2] - get_rand(9, 29);
	player.rogue.exp -= 2;

	let hp = hp_raise();
	player.rogue.hp_current -= hp;
	if player.rogue.hp_current <= 0 {
		player.rogue.hp_current = 1;
	}
	player.rogue.hp_max -= hp;
	if player.rogue.hp_max <= 0 {
		player.rogue.hp_max = 1;
	}
	add_exp(1, false, player);
}

unsafe fn drain_life(player: &mut Player) {
	if rand_percent(60) || player.rogue.hp_max <= 30 || player.rogue.hp_current < 10 {
		return;
	}

	let n = get_rand(1, 3);             /* 1 Hp, 2 Str, 3 both */
	if n != 2 || !player.ring_effects.has_sustain_strength() {
		message("you feel weaker", 0);
	}
	if n != 2 {
		player.rogue.hp_max -= 1;
		player.rogue.hp_current -= 1;
		less_hp += 1;
	}
	if n != 1 {
		if player.rogue.str_current > 3 && !player.ring_effects.has_sustain_strength() {
			player.rogue.str_current -= 1;
			if coin_toss() {
				player.rogue.str_max -= 1;
			}
		}
	}
	print_stats(STAT_STRENGTH | STAT_HP, player);
}

pub unsafe fn m_confuse(monster: &mut Monster, player: &Player, level: &Level) -> bool {
	if !rogue_can_see(monster.spot.row, monster.spot.col, player, level) {
		return false;
	}
	if rand_percent(45) {
		/* will not confuse the rogue */
		monster.m_flags.confuses = false;
		return false;
	}
	if rand_percent(55) {
		monster.m_flags.confuses = false;
		let msg = format!("the gaze of the {} has confused you", mon_name(monster, player, level));
		message(&msg, 1);
		confuse();
		return true;
	}
	return false;
}

pub unsafe fn flame_broil(monster: &mut Monster, player: &mut Player, level: &mut Level) -> bool {
	if !mon_sees(monster, player.rogue.row, player.rogue.col, level) || coin_toss() {
		return false;
	}
	{
		let mut delta_row = player.rogue.row - monster.spot.row;
		let mut delta_col = player.rogue.col - monster.spot.col;
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
	if player.blind.is_inactive() && !rogue_is_around(monster.spot.row, monster.spot.col, player) {
		let mut row = monster.spot.row;
		let mut col = monster.spot.col;
		get_closer(&mut row, &mut col, player.rogue.row, player.rogue.col);
		standout();
		loop {
			mvaddch(row as i32, col as i32, chtype::from('~'));
			refresh();
			get_closer(&mut row, &mut col, player.rogue.row, player.rogue.col);
			let stay_looping = row != player.rogue.row || col != player.rogue.col;
			if !stay_looping {
				break;
			}
		}
		standend();
		row = monster.spot.row;
		col = monster.spot.col;
		get_closer(&mut row, &mut col, player.rogue.row, player.rogue.col);
		loop {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col, player, level));
			refresh();
			get_closer(&mut row, &mut col, player.rogue.row, player.rogue.col);
			let stay_looping = row != player.rogue.row || col != player.rogue.col;
			if !stay_looping {
				break;
			}
		}
	}
	mon_hit(monster, Some(FLAME_NAME), true, player, level);
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