use crate::armors::ArmorKind;
use crate::hit::mon_hit;
use crate::init::{Dungeon, GameState};
use crate::inventory::get_obj_desc;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::{add_exp, hp_raise, Level, LEVEL_POINTS};
use crate::monster::{mon_can_go_and_reach, mon_name, mv_mons, mv_monster, MonsterMash};
use crate::motion::YOU_CAN_MOVE_AGAIN;
use crate::objects::{alloc_object, get_armor_class, gr_object, place_at, Object};
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat::{Gold, Weapon};
use crate::prelude::*;
use crate::r#use::{confuse, vanish};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::render_system::animation::animate_flame_broil;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::dice::roll_chance;
use crate::score::killed_by;
use rand::{thread_rng, Rng};

pub const FLAME_NAME: &'static str = "flame";

pub fn special_hit(mon_id: u64, game: &mut impl Dungeon) {
	if game.as_monster_flags(mon_id).confused && rand_percent(66) {
		return;
	}
	if game.as_monster_flags(mon_id).rusts {
		// TODO rust(Some(mon_id), game);
	}
	if game.as_monster_flags(mon_id).holds && game.as_health().levitate.is_inactive() {
		let health = game.as_health_mut();
		health.being_held = true;
	}
	if game.as_monster_flags(mon_id).freezes {
		// TODO freeze(mon_id, game);
	}
	if game.as_monster_flags(mon_id).stings {
		// TODO sting(mon_id, game);
	}
	if game.as_monster_flags(mon_id).drains_life {
		// TODO drain_life(game);
	}
	if game.as_monster_flags(mon_id).drops_level {
		// TODO drop_level(game);
	}
	if game.as_monster_flags(mon_id).steals_gold {
		// TODO steal_gold(mon_id, game);
	} else if game.as_monster_flags(mon_id).steals_item {
		// TODO steal_item(mon_id, game);
	}
}

pub fn rust(mon_id: Option<u64>, game: &mut GameState) {
	if game.player.armor().is_none()
		|| get_armor_class(game.player.armor()) <= 1
		|| game.player.armor_kind() == Some(ArmorKind::Leather) {
		return;
	}

	let player_has_maintain_armor = game.player.ring_effects.has_maintain_armor();

	let armor = game.player.armor_mut().expect("armor exists");
	if armor.is_protected != 0 || player_has_maintain_armor {
		if let Some(mon_id) = mon_id {
			if !game.mash.monster_flags(mon_id).rust_vanished {
				game.diary.add_entry("the rust vanishes instantly");
				let flags = game.mash.monster_flags_mut(mon_id);
				flags.rust_vanished = true;
			}
		}
	} else {
		armor.d_enchant -= 1;
		game.diary.add_entry("your armor weakens");
		game.as_diary_mut().set_stats_changed(true);
	}
}

fn freeze(mon_id: u64, game: &mut GameState) {
	if rand_percent(12) {
		return;
	}
	let mut freeze_percent: isize = 99;
	freeze_percent -= game.player.rogue.str_current + (game.player.rogue.str_current / 2);
	freeze_percent -= game.player.buffed_exp() * 4;
	freeze_percent -= get_armor_class(game.player.armor()) * 5;
	freeze_percent -= game.player.rogue.hp_max / 3;
	if freeze_percent > 10 {
		let flags = game.mash.monster_flags_mut(mon_id);
		flags.freezing_rogue = true;
		game.player.interrupt_and_slurp();
		game.diary.add_entry("you are frozen");

		let n = get_rand(4, 8);
		for _ in 0..n {
			mv_mons(game);
		}
		if rand_percent(freeze_percent as usize) {
			for _ in 0..50 {
				mv_mons(game);
			}
			killed_by(Ending::Hypothermia, game);
		}
		game.player.interrupt_and_slurp();
		game.diary.add_entry(YOU_CAN_MOVE_AGAIN);
		let flags = game.mash.monster_flags_mut(mon_id);
		flags.freezing_rogue = false;
	}
}

fn steal_gold(mon_id: u64, game: &mut GameState) {
	if game.player.rogue.gold <= 0 || rand_percent(10) {
		return;
	}

	let cur_depth = game.player.cur_depth;
	let amount = get_rand(cur_depth * 10, cur_depth * 30).min(game.player.rogue.gold);
	game.player.rogue.gold -= amount;
	game.diary.add_entry("your purse feels lighter");
	game.as_diary_mut().set_stats_changed(true);
	disappear(mon_id, game);
}

fn steal_item(mon_id: u64, game: &mut GameState) {
	if rand_percent(15) {
		return;
	}
	if game.as_rogue_pack().len() == 0 {
		disappear(mon_id, game);
		return;
	}
	match game.player.random_unused_object_id() {
		None => {
			disappear(mon_id, game);
			return;
		}
		Some(obj_id) => {
			let msg = {
				let obj_desc = {
					let mut temp_obj = game.player.expect_object(obj_id).clone();
					if temp_obj.what_is != Weapon {
						temp_obj.quantity = 1;
					}
					get_obj_desc(&temp_obj, game.as_settings().fruit.to_string(), &game.player)
				};
				format!("she stole {}", obj_desc)
			};
			let diary = game.as_diary_mut();
			diary.add_entry(&msg);
			vanish(obj_id, false, game);
			disappear(mon_id, game);
		}
	}
}

fn disappear(mon_id: u64, game: &mut GameState) {
	let monster_spot = {
		let monster = game.mash.monster(mon_id);
		game.level.dungeon[monster.spot.row as usize][monster.spot.col as usize].set_monster(false);
		monster.spot
	};
	game.render_spot(monster_spot);
	game.mash.remove_monster(mon_id);
	game.mash.mon_disappeared = true;
}


pub fn cough_up(mon_id: u64, game: &mut GameState) {
	if game.player.cur_depth < game.player.max_depth {
		return;
	}
	let obj = if game.mash.monster_flags(mon_id).steals_gold {
		let mut obj = alloc_object(&mut thread_rng());
		obj.what_is = Gold;
		obj.quantity = get_rand((game.player.cur_depth * 15) as i16, (game.player.cur_depth * 30) as i16);
		obj
	} else {
		if !rand_percent(game.mash.monster(mon_id).drop_percent) {
			return;
		}
		gr_object(&mut game.player)
	};
	let monster = game.mash.monster(mon_id);
	let row = monster.spot.row;
	let col = monster.spot.col;
	for n in 0..=5 {
		for i in -n..=n {
			let cough_col = col + i;
			if try_to_cough(row + n, cough_col, &obj, game) {
				return;
			}
			if try_to_cough(row - n, cough_col, &obj, game) {
				return;
			}
		}
		for i in -n..=n {
			let cough_row = row + i;
			if try_to_cough(cough_row, col - n, &obj, game) {
				return;
			}
			if try_to_cough(cough_row, col + n, &obj, game) {
				return;
			}
		}
	}
}

fn try_to_cough(row: i64, col: i64, obj: &Object, game: &mut GameState) -> bool {
	if row < MIN_ROW || row > (DROWS - 2) as i64 || col < 0 || col > (DCOLS - 1) as i64 {
		return false;
	}
	let dungeon_cell = game.level.dungeon[row as usize][col as usize];
	if !(dungeon_cell.has_object() || dungeon_cell.is_stairs() || dungeon_cell.is_any_trap())
		&& (dungeon_cell.is_any_tunnel() || dungeon_cell.is_any_floor() || dungeon_cell.is_any_door()) {
		place_at(obj.clone(), row, col, &mut game.level, &mut game.ground);
		game.render_spot(DungeonSpot { row, col });
		return true;
	}
	false
}

pub fn seek_gold(mon_id: u64, game: &mut impl Dungeon) -> bool {
	let room = {
		let monster = game.as_monster(mon_id);
		let monster_row = monster.spot.row;
		let monster_col = monster.spot.col;
		game.get_room_at(monster_row, monster_col)
	};
	if room.is_none() {
		return false;
	}
	let room = room.unwrap();
	let bounds = game.room_bounds(room).inset(1, 1);
	for spot in bounds.to_spots() {
		let (row, col) = spot.i64();
		if gold_at(row, col, game) && !game.has_monster(row, col) {
			if mon_can_go_and_reach(mon_id, row, col, true, game) {
				game.move_mon_to(mon_id, row, col);
				let flags = game.as_monster_flags_mut(mon_id);
				flags.asleep = true;
				flags.wakens = false;
				flags.seeks_gold = false;
				return true;
			}
			{
				let flags = game.as_monster_flags_mut(mon_id);
				flags.seeks_gold = false;
			}
			mv_monster(mon_id, row, col, true, game, &mut thread_rng());
			{
				let flags = game.as_monster_flags_mut(mon_id);
				flags.seeks_gold = true;
			}
			return true;
		}
	}
	false
}

fn gold_at(row: i64, col: i64, game: &impl Dungeon) -> bool {
	if let Some(obj) = game.try_object_at(row, col) {
		if obj.what_is == Gold {
			return true;
		}
	}
	false
}

pub fn check_imitator(mon_id: u64, game: &mut GameState) -> bool {
	if game.as_monster(mon_id).imitates() {
		game.as_monster_mut(mon_id).wake_up();

		if game.player.health.blind.is_inactive() {
			let mon_name = mon_name(mon_id, game);
			let mon_spot = game.as_monster(mon_id).spot;
			game.render_spot(mon_spot);
			game.player.interrupt_and_slurp();
			game.diary.add_entry(&format!("wait, that's a {mon_name}!"));
		}
		return true;
	}
	false
}

pub fn imitating(row: usize, col: usize, mash: &MonsterMash, level: &Level) -> bool {
	if level.dungeon[row][col].has_monster() {
		if let Some(monster) = mash.monster_at_spot(row as i64, col as i64) {
			if monster.m_flags.imitates {
				return true;
			}
		}
	}
	false
}

fn sting(mon_id: u64, game: &mut GameState) {
	if game.player.rogue.str_current <= 3 || game.player.ring_effects.has_sustain_strength() {
		return;
	}

	let mut sting_chance: isize = 35;
	sting_chance += 6 * (6 - get_armor_class(game.player.armor()));

	let buffed_exp = game.player.buffed_exp();
	if buffed_exp > 8 {
		sting_chance -= 6 * (buffed_exp - 8);
	}
	if rand_percent(sting_chance as usize) {
		let name = mon_name(mon_id, game);
		game.diary.add_entry(&format!("the {}'s bite has weakened you", name));
		game.player.rogue.str_current -= 1;
		game.as_diary_mut().set_stats_changed(true);
	}
}

fn drop_level(game: &mut GameState) {
	let rng = &mut thread_rng();
	if roll_chance(80, rng) || game.player.rogue.exp.level <= 5 {
		return;
	}

	game.player.rogue.exp.points = LEVEL_POINTS[game.player.rogue.exp.level - 2] - rng.gen_range(9..=29);
	game.player.rogue.exp.level -= 2;

	let hp = hp_raise(game.player.wizard, rng);
	game.player.rogue.hp_current -= hp;
	if game.player.rogue.hp_current <= 0 {
		game.player.rogue.hp_current = 1;
	}
	game.player.rogue.hp_max -= hp;
	if game.player.rogue.hp_max <= 0 {
		game.player.rogue.hp_max = 1;
	}
	add_exp(1, false, game, rng);
}

fn drain_life(game: &mut GameState) {
	if rand_percent(60) || game.player.rogue.hp_max <= 30 || game.player.rogue.hp_current < 10 {
		return;
	}

	let n = get_rand(1, 3);             /* 1 Hp, 2 Str, 3 both */
	if n != 2 || !game.player.ring_effects.has_sustain_strength() {
		game.diary.add_entry("you feel weaker");
	}
	if n != 2 {
		let drain = 1;
		game.player.rogue.hp_max -= drain;
		game.player.rogue.hp_current -= drain;
		game.player.less_hp += drain;
	}
	if n != 1 {
		if game.player.rogue.str_current > 3 && !game.player.ring_effects.has_sustain_strength() {
			game.player.rogue.str_current -= 1;
			if coin_toss() {
				game.player.rogue.str_max -= 1;
			}
		}
	}
	game.as_diary_mut().set_stats_changed(true);
}

pub fn m_confuse(mon_id: u64, game: &mut impl Dungeon) -> bool {
	let monster = game.as_monster(mon_id);
	let row = monster.spot.row;
	let col = monster.spot.col;

	if !game.rogue_can_see(row, col) {
		return false;
	}
	if rand_percent(45) {
		/* will not confuse the rogue */
		let monster = game.as_monster_mut(mon_id);
		monster.m_flags.confuses = false;
		return false;
	}
	if rand_percent(55) {
		let monster = game.as_monster_mut(mon_id);
		monster.m_flags.confuses = false;
		let msg = format!("the gaze of the {} has confused you", mon_name(mon_id, game));
		game.interrupt_and_slurp();
		game.as_diary_mut().add_entry(&msg);
		confuse(game);
		return true;
	}
	false
}

pub fn flame_broil(mon_id: u64, game: &mut impl Dungeon) -> bool {
	let cant_see_rogue = !game.monster_sees_rogue(mon_id);
	if cant_see_rogue || coin_toss() {
		return false;
	}

	let mon_spot = game.as_monster(mon_id).spot;
	let player_spot = DungeonSpot { row: game.rogue_row(), col: game.rogue_col() };
	if !mon_spot.has_attack_vector_to(player_spot) || player_spot.distance_from(mon_spot) > 7 {
		return false;
	}
	if !game.as_health().is_blind() && !player_spot.is_near(mon_spot) {
		let path = mon_spot.path_to(player_spot);
		animate_flame_broil(&path);
	}
	mon_hit(mon_id, Some(FLAME_NAME), true, game);
	true
}
