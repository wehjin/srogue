#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch, refresh, standend, standout};

use crate::armors::ArmorKind;
use crate::hit::mon_hit;
use crate::inventory::get_obj_desc;
use crate::level::{add_exp, hp_raise, Level, LEVEL_POINTS};
use crate::level::constants::{DCOLS, DROWS};
use crate::message::{check_message, message, print_stats};
use crate::monster::{mon_can_go, mon_disappeared, mon_name, Monster, MonsterMash, move_mon_to, mv_mons, mv_monster};
use crate::objects::{alloc_object, get_armor_class, gr_object, Object, ObjectPack, place_at};
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::{Gold, Weapon};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_GOLD, STAT_HP, STAT_STRENGTH};
use crate::r#move::YOU_CAN_MOVE_AGAIN;
use crate::r#use::{confuse, vanish};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::{get_dungeon_char, get_room_number};
use crate::score::killed_by;

pub const FLAME_NAME: &'static str = "flame";

pub unsafe fn special_hit(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	if mash.monster_flags(mon_id).confused && rand_percent(66) {
		return;
	}
	if mash.monster_flags(mon_id).rusts {
		rust(Some(mash.monster_mut(mon_id)), player);
	}
	if mash.monster_flags(mon_id).holds && player.levitate.is_inactive() {
		level.being_held = true;
	}
	if mash.monster_flags(mon_id).freezes {
		freeze(mon_id, mash, player, level, ground);
	}
	if mash.monster_flags(mon_id).stings {
		sting(mash.monster(mon_id), player, level);
	}
	if mash.monster_flags(mon_id).drains_life {
		drain_life(player);
	}
	if mash.monster_flags(mon_id).drops_level {
		drop_level(player);
	}
	if mash.monster_flags(mon_id).steals_gold {
		steal_gold(mon_id, mash, player, level);
	} else if mash.monster_flags(mon_id).steals_item {
		steal_item(mon_id, mash, player, level, ground);
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

unsafe fn freeze(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	if rand_percent(12) {
		return;
	}
	let mut freeze_percent: isize = 99;
	freeze_percent -= player.rogue.str_current + (player.rogue.str_current / 2);
	freeze_percent -= player.buffed_exp() * 4;
	freeze_percent -= get_armor_class(player.armor()) * 5;
	freeze_percent -= player.rogue.hp_max / 3;
	if freeze_percent > 10 {
		mash.monster_flags_mut(mon_id).freezing_rogue = true;
		message("you are frozen", 1);

		let n = get_rand(4, 8);
		for _ in 0..n {
			mv_mons(mash, player, level, ground);
		}
		if rand_percent(freeze_percent as usize) {
			for _ in 0..50 {
				mv_mons(mash, player, level, ground);
			}
			killed_by(Ending::Hypothermia, player);
		}
		message(YOU_CAN_MOVE_AGAIN, 1);
		mash.monster_flags_mut(mon_id).freezing_rogue = false;
	}
}

unsafe fn steal_gold(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	if player.rogue.gold <= 0 || rand_percent(10) {
		return;
	}

	let cur_depth = player.cur_depth as usize;
	let amount = get_rand(cur_depth * 10, cur_depth * 30).min(player.rogue.gold);
	player.rogue.gold -= amount;
	message("your purse feels lighter", 0);
	print_stats(STAT_GOLD, player);
	disappear(mon_id, mash, player, level);
}

unsafe fn steal_item(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	if rand_percent(15) {
		return;
	}
	if player.pack().len() == 0 {
		disappear(mon_id, mash, player, level);
		return;
	}
	match player.random_unused_object_id() {
		None => {
			disappear(mon_id, mash, player, level);
			return;
		}
		Some(obj_id) => {
			let msg = {
				let obj_desc = {
					let mut temp_obj = player.expect_object(obj_id).clone();
					if temp_obj.what_is != Weapon {
						temp_obj.quantity = 1;
					}
					get_obj_desc(&temp_obj, player.settings.fruit.to_string(), player)
				};
				format!("she stole {}", obj_desc)
			};
			message(&msg, 0);
			vanish(obj_id, false, mash, player, level, ground);
			disappear(mon_id, mash, player, level);
		}
	}
}

unsafe fn disappear(mon_id: u64, mash: &mut MonsterMash, player: &Player, level: &mut Level) {
	let monster_spot = {
		let monster = mash.monster(mon_id);
		level.dungeon[monster.spot.row as usize][monster.spot.col as usize].set_monster(false);
		monster.spot
	};
	let DungeonSpot { row, col } = monster_spot;
	if player.can_see(row, col, level) {
		let dungeon_char = get_dungeon_char(row, col, mash, player, level);
		mvaddch(row as i32, col as i32, dungeon_char);
	}
	mash.remove_monster(mon_id);
	mon_disappeared = true;
}


pub unsafe fn cough_up(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &mut ObjectPack) {
	if player.cur_depth < player.max_depth {
		return;
	}
	let obj = if mash.monster_flags(mon_id).steals_gold {
		let mut obj = alloc_object();
		obj.what_is = Gold;
		obj.quantity = get_rand((player.cur_depth * 15) as i16, (player.cur_depth * 30) as i16);
		obj
	} else {
		if !rand_percent(mash.monster(mon_id).drop_percent) {
			return;
		}
		gr_object(player)
	};
	let monster = mash.monster(mon_id);
	let row = monster.spot.row;
	let col = monster.spot.col;
	for n in 0..=5 {
		for i in -n..=n {
			let cough_col = col + i;
			if try_to_cough(row + n, cough_col, &obj, mash, player, level, ground) {
				return;
			}
			if try_to_cough(row - n, cough_col, &obj, mash, player, level, ground) {
				return;
			}
		}
		for i in -n..=n {
			let cough_row = row + i;
			if try_to_cough(cough_row, col - n, &obj, mash, player, level, ground) {
				return;
			}
			if try_to_cough(cough_row, col + n, &obj, mash, player, level, ground) {
				return;
			}
		}
	}
}

unsafe fn try_to_cough(row: i64, col: i64, obj: &Object, mash: &mut MonsterMash, player: &Player, level: &mut Level, ground: &mut ObjectPack) -> bool {
	if row < MIN_ROW || row > (DROWS - 2) as i64 || col < 0 || col > (DCOLS - 1) as i64 {
		return false;
	}
	let dungeon_cell = level.dungeon[row as usize][col as usize];
	if !(dungeon_cell.has_object() || dungeon_cell.is_stairs() || dungeon_cell.is_trap())
		&& (dungeon_cell.is_tunnel() || dungeon_cell.is_floor() || dungeon_cell.is_door()) {
		place_at(obj.clone(), row, col, level, ground);
		if (row != player.rogue.row || col != player.rogue.col)
			&& !dungeon_cell.has_monster() {
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col, mash, player, level));
		}
		return true;
	}
	return false;
}

pub unsafe fn seek_gold(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) -> bool {
	let rn = {
		let monster = mash.monster(mon_id);
		get_room_number(monster.spot.row, monster.spot.col, level)
	};
	if rn < 0 {
		return false;
	}

	let rn = rn as usize;
	for i in (level.rooms[rn].top_row + 1)..level.rooms[rn].bottom_row {
		for j in (level.rooms[rn].left_col + 1)..level.rooms[rn].right_col {
			if gold_at(i, j, level, ground) && !level.dungeon[i as usize][j as usize].has_monster() {
				mash.monster_flags_mut(mon_id).can_flit = true;
				let can_go_if_while_can_flit = mon_can_go(mash.monster(mon_id), i, j, player, level, ground);
				mash.monster_flags_mut(mon_id).can_flit = false;
				if can_go_if_while_can_flit {
					let monster = mash.monster_mut(mon_id);
					move_mon_to(monster, i, j, player, level);
					monster.m_flags.asleep = true;
					monster.m_flags.wakens = false;
					monster.m_flags.seeks_gold = false;
					return true;
				}
				mash.monster_flags_mut(mon_id).seeks_gold = false;
				mash.monster_flags_mut(mon_id).can_flit = true;
				mv_monster(mon_id, i, j, mash, player, level, ground);
				mash.monster_flags_mut(mon_id).can_flit = false;
				mash.monster_flags_mut(mon_id).seeks_gold = true;
				return true;
			}
		}
	}
	return false;
}

unsafe fn gold_at(row: i64, col: i64, level: &Level, ground: &ObjectPack) -> bool {
	if level.dungeon[row as usize][col as usize].has_object() {
		if let Some(obj) = ground.find_object_at(row, col) {
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

pub unsafe fn check_imitator(mon_id: u64, mash: &mut MonsterMash, player: &Player, level: &Level) -> bool {
	if mash.monster_flags(mon_id).imitates {
		mash.monster_mut(mon_id).wake_up();
		if player.blind.is_inactive() {
			let monster = mash.monster(mon_id);
			let dungeon_char = get_dungeon_char(monster.spot.row, monster.spot.col, mash, player, level);
			{
				let monster = mash.monster(mon_id);
				mvaddch(monster.spot.row as i32, monster.spot.col as i32, dungeon_char);
				check_message();
				let msg = format!("wait, that's a {}!", mon_name(monster, player, level));
				message(&msg, 1);
			}
		}
		return true;
	}
	return false;
}

pub unsafe fn imitating(row: i64, col: i64, mash: &mut MonsterMash, level: &Level) -> bool {
	if level.dungeon[row as usize][col as usize].has_monster() {
		if let Some(monster) = mash.monster_at_spot(row, col) {
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

	let hp = hp_raise(player);
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
		let drain = 1;
		player.rogue.hp_max -= drain;
		player.rogue.hp_current -= drain;
		player.less_hp += drain;
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

pub unsafe fn m_confuse(monster: &mut Monster, player: &mut Player, level: &Level) -> bool {
	let row = monster.spot.row;
	let col = monster.spot.col;
	if !player.can_see(row, col, level) {
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
		confuse(player);
		return true;
	}
	return false;
}

pub unsafe fn flame_broil(mon_id: u64, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) -> bool {
	if !mash.monster(mon_id).sees(player.rogue.row, player.rogue.col, level) || coin_toss() {
		return false;
	}
	{
		let monster = mash.monster(mon_id);
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
	let monster = mash.monster(mon_id);
	let row = monster.spot.row;
	let col = monster.spot.col;
	if player.blind.is_inactive() && !player.is_near(row, col) {
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
			mvaddch(row as i32, col as i32, get_dungeon_char(row, col, mash, player, level));
			refresh();
			get_closer(&mut row, &mut col, player.rogue.row, player.rogue.col);
			let stay_looping = row != player.rogue.row || col != player.rogue.col;
			if !stay_looping {
				break;
			}
		}
	}
	mon_hit(mon_id, Some(FLAME_NAME), true, mash, player, level, ground);
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