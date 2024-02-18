use ncurses::chtype;
use serde::{Deserialize, Serialize};

pub use flags::MonsterFlags;
pub use kind::*;
pub use mash::*;

use crate::hit::mon_hit;
use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::motion::is_passable;
use crate::objects::{ObjectId, ObjectPack};
use crate::odds;
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::object_what::ObjectWhat::Scroll;
use crate::random::{coin_toss, get_rand, get_rand_indices, rand_percent};
use crate::room::{dr_course, get_room_number, gr_spot};
use crate::scrolls::ScrollKind;
use crate::scrolls::ScrollKind::ScareMonster;
use crate::spec_hit::{flame_broil, m_confuse, seek_gold};
use crate::throw::RandomWalk;

pub mod flags;
mod kind;
mod mash;

#[derive(Clone, Serialize, Deserialize)]
pub struct Fighter {
	pub armor: Option<ObjectId>,
	pub weapon: Option<ObjectId>,
	pub left_ring: Option<ObjectId>,
	pub right_ring: Option<ObjectId>,
	pub hp_current: isize,
	pub hp_max: isize,
	pub str_current: isize,
	pub str_max: isize,
	pub pack: ObjectPack,
	pub gold: usize,
	pub exp: isize,
	pub exp_points: isize,
	pub row: i64,
	pub col: i64,
	pub fchar: char,
	pub moves_left: isize,
}


pub fn put_mons(game: &mut GameState) {
	for _ in 0..get_rand(4, 6) {
		let mut monster = gr_monster(game.player.cur_depth, 0, None);
		if monster.m_flags.wanders && coin_toss() {
			monster.wake_up();
		}
		let spot = gr_spot(
			|cell| cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.has_object(),
			&game.player,
			&game.level,
		);
		put_m_at(spot.row, spot.col, monster, &mut game.mash, &mut game.level);
	}
}

pub fn gr_monster(level_depth: isize, first_level_boost: isize, kind: Option<MonsterKind>) -> Monster {
	let kind = kind.unwrap_or_else(|| MonsterKind::random(level_depth, first_level_boost));
	let mut monster = Monster::create(kind);
	if monster.m_flags.imitates {
		monster.disguise_char = gr_obj_char();
	}
	if level_depth > AMULET_LEVEL + 2 {
		monster.m_flags.hasted = true;
	}
	monster.target_spot = None;
	return monster;
}

pub fn mv_mons(game: &mut GameState) {
	if game.player.haste_self.is_half_active() {
		return;
	}

	for mon_id in game.mash.monster_ids() {
		if game.player.cleaned_up.is_some() {
			break;
		}
		let mut done_with_monster = false;
		if game.mash.test_monster(mon_id, Monster::is_hasted) {
			game.mash.mon_disappeared = false;
			mv_monster(mon_id, game.player.rogue.row, game.player.rogue.col, game);
			if game.mash.mon_disappeared {
				done_with_monster = true;
			}
		} else if game.mash.test_monster(mon_id, Monster::is_slowed) {
			game.mash.monster_mut(mon_id).flip_slowed_toggle();
			if game.mash.test_monster(mon_id, Monster::slowed_toggle) {
				done_with_monster = true;
			}
		}
		if !done_with_monster && game.mash.test_monster(mon_id, Monster::is_confused) {
			if move_confused(game.mash.monster_mut(mon_id), &game.player, &mut game.level, &game.ground) {
				done_with_monster = true;
			}
		}
		if !done_with_monster {
			let mut flew = false;
			let monster = game.mash.monster(mon_id);
			if monster.flies()
				&& !monster.is_napping()
				&& !mon_can_go(monster, game.player.rogue.row, game.player.rogue.col, &game.player, &game.level, &game.ground) {
				flew = true;
				mv_monster(mon_id, game.player.rogue.row, game.player.rogue.col, game);
			}
			let monster = game.mash.monster(mon_id);
			if !(flew && mon_can_go(monster, game.player.rogue.row, game.player.rogue.col, &game.player, &game.level, &game.ground)) {
				mv_monster(mon_id, game.player.rogue.row, game.player.rogue.col, game);
			}
		}
	}
}

pub fn party_monsters(rn: usize, n: usize, level_depth: isize, mash: &mut MonsterMash, level: &mut Level) {
	let first_level_shift = level_depth % 3;
	let n = n + n;
	for _i in 0..n {
		if no_spot_for_monster(rn, level) {
			break;
		}
		let mut found: Option<(i64, i64)> = None;
		for _j in 0..250 {
			let row = get_rand(level.rooms[rn].top_row + 1, level.rooms[rn].bottom_row - 1);
			let col = get_rand(level.rooms[rn].left_col + 1, level.rooms[rn].right_col - 1);
			let dungeon_spot = level.dungeon[row as usize][col as usize];
			if !dungeon_spot.has_monster() && (dungeon_spot.is_any_floor() || dungeon_spot.is_any_tunnel()) {
				found = Some((row, col));
				break;
			}
		}
		if let Some((row, col)) = found {
			let mut monster = gr_monster(level_depth, first_level_shift, None);
			if !monster.m_flags.imitates {
				monster.m_flags.wakens = true;
			}
			put_m_at(row, col, monster, mash, level);
		}
	}
}

pub fn gmc_row_col(row: i64, col: i64, game: &GameState) -> chtype {
	if let Some(monster) = game.mash.monster_at_spot(row, col) {
		gmc(monster, &game.player, &game.level)
	} else {
		ncurses::chtype::from('&')
	}
}

pub fn gmc(monster: &Monster, player: &Player, level: &Level) -> chtype {
	if (monster.is_invisible() && !player_defeats_invisibility(player, level))
		|| player.blind.is_active() {
		monster.trail_char
	} else if monster.m_flags.imitates {
		monster.disguise_char
	} else {
		monster.m_char()
	}
}

pub fn mv_monster(mon_id: u64, row: i64, col: i64, game: &mut GameState) {
	if game.mash.monster_flags(mon_id).asleep {
		if game.mash.monster_flags(mon_id).napping {
			game.mash.monster_mut(mon_id).do_nap();
			return;
		}
		let chance = odds::WAKE_PERCENT;
		let monster = game.mash.monster(mon_id);
		let row1 = monster.spot.row;
		let col1 = monster.spot.col;
		if (monster.m_flags.wakens)
			&& game.player.is_near(row1, col1)
			&& rand_percent(game.player.ring_effects.apply_stealthy(chance)) {
			game.mash.monster_mut(mon_id).wake_up();
		}
		return;
	} else if game.mash.monster_flags(mon_id).already_moved {
		game.mash.monster_flags_mut(mon_id).already_moved = false;
		return;
	}
	if game.mash.monster_flags(mon_id).flits && flit(game.mash.monster_mut(mon_id), &game.player, &mut game.level, &game.ground) {
		return;
	}
	if game.mash.monster_flags(mon_id).stationary
		&& !mon_can_go(game.mash.monster(mon_id), game.player.rogue.row, game.player.rogue.col, &game.player, &game.level, &game.ground) {
		return;
	}
	if game.mash.monster_flags(mon_id).freezing_rogue {
		return;
	}
	if game.mash.monster_flags(mon_id).confuses && m_confuse(mon_id, game) {
		return;
	}
	if mon_can_go(game.mash.monster(mon_id), game.player.rogue.row, game.player.rogue.col, &game.player, &game.level, &game.ground) {
		mon_hit(mon_id, None, false, game);
		return;
	}
	if game.mash.monster_flags(mon_id).flames && flame_broil(mon_id, game) {
		return;
	}
	if game.mash.monster_flags(mon_id).seeks_gold && seek_gold(mon_id, game) {
		return;
	}

	game.mash.monster_mut(mon_id).clear_target_spot_if_reached();
	let monster = game.mash.monster_mut(mon_id);
	let target_spot = monster.target_spot_or(DungeonSpot { row, col });
	let row = monster.spot.next_closest_row(target_spot.row);
	if game.level.dungeon[row as usize][monster.spot.col as usize].is_any_door() && mtry(monster, row, monster.spot.col, &game.player, &mut game.level, &game.ground) {
		return;
	}
	let col = monster.spot.next_closest_col(target_spot.col);
	if game.level.dungeon[monster.spot.row as usize][col as usize].is_any_door() && mtry(monster, monster.spot.row, col, &game.player, &mut game.level, &game.ground) {
		return;
	}
	if mtry(monster, row, col, &game.player, &mut game.level, &game.ground) {
		return;
	}
	for kind in get_rand_indices(6) {
		match kind {
			0 => if mtry(monster, row, monster.spot.col - 1, &game.player, &mut game.level, &game.ground) { break; }
			1 => if mtry(monster, row, monster.spot.col, &game.player, &mut game.level, &game.ground) { break; }
			2 => if mtry(monster, row, monster.spot.col + 1, &game.player, &mut game.level, &game.ground) { break; }
			3 => if mtry(monster, monster.spot.row - 1, col, &game.player, &mut game.level, &game.ground) { break; }
			4 => if mtry(monster, monster.spot.row, col, &game.player, &mut game.level, &game.ground) { break; }
			5 => if mtry(monster, monster.spot.row + 1, col, &game.player, &mut game.level, &game.ground) { break; }
			_ => unreachable!("0 <= n  <= 5")
		}
	}

	// No possible moves
	monster.stuck_counter.log_row_col(monster.spot.row, monster.spot.col);
	if monster.stuck_counter.count > 4 {
		let row1 = game.player.rogue.row;
		let col1 = game.player.rogue.col;
		if monster.target_spot.is_none() && !monster.sees(row1, col1, &game.level) {
			monster.set_target_spot(
				get_rand(1, (DROWS - 2) as i64),
				get_rand(0, (DCOLS - 1) as i64),
			);
		} else {
			monster.clear_target_spot();
		}
	}
}

pub fn mtry(monster: &mut Monster, row: i64, col: i64, player: &Player, level: &mut Level, ground: &ObjectPack) -> bool {
	if mon_can_go(monster, row, col, player, level, ground) {
		move_mon_to(monster, row, col, player, level);
		return true;
	}
	return false;
}

pub fn move_mon_to(monster: &mut Monster, row: i64, col: i64, player: &Player, level: &mut Level) {
	let (mrow, mcol) = (monster.spot.row, monster.spot.col);
	level.dungeon[mrow as usize][mcol as usize].set_monster(false);
	level.dungeon[row as usize][col as usize].set_monster(true);

	let c = ncurses::mvinch(mrow as i32, mcol as i32);
	if (c >= chtype::from('A')) && (c <= chtype::from('Z')) {
		// Restore the screen appearance at the newly vacated spot
		let exit_trail_char = if !level.detect_monster {
			monster.trail_char
		} else {
			if player.can_see(mrow, mcol, level) {
				monster.trail_char
			} else {
				if monster.trail_char == chtype::from('.') {
					monster.trail_char = chtype::from(' ');
				}
				monster.trail_char
			}
		};
		ncurses::mvaddch(mrow as i32, mcol as i32, exit_trail_char);
	}
	// Set the screen appearance at the newly occupied spot
	monster.trail_char = ncurses::mvinch(row as i32, col as i32);
	if player.blind.is_inactive()
		&& (level.detect_monster || player.can_see(row, col, level)) {
		if !monster.m_flags.invisible || player_defeats_invisibility(player, level) {
			ncurses::mvaddch(row as i32, col as i32, gmc(monster, player, level));
		}
	}
	if level.dungeon[row as usize][col as usize].is_any_door()
		&& !player.in_room(row, col, level)
		&& level.dungeon[mrow as usize][mcol as usize].is_any_floor()
		&& player.blind.is_inactive() {
		ncurses::mvaddch(mrow as i32, mcol as i32, chtype::from(' '));
	}
	if level.dungeon[row as usize][col as usize].is_any_door() {
		let entering = level.dungeon[monster.spot.row as usize][monster.spot.col as usize].is_any_tunnel();
		dr_course(monster, entering, row, col, player, level);
	} else {
		monster.spot.set(row, col);
	}
}

pub fn mon_can_go(monster: &Monster, row: i64, col: i64, player: &Player, level: &Level, ground: &ObjectPack) -> bool {
	let dr = monster.spot.row as isize - row as isize;        /* check if move distance > 1 */
	if (dr >= 2) || (dr <= -2) {
		return false;
	}
	let dc = monster.spot.col as isize - col as isize;
	if (dc >= 2) || (dc <= -2) {
		return false;
	}
	if level.dungeon[monster.spot.row as usize][col as usize].is_nothing()
		|| level.dungeon[row as usize][monster.spot.col as usize].is_nothing() {
		return false;
	}
	if !is_passable(row, col, level) || level.dungeon[row as usize][col as usize].has_monster() {
		return false;
	}
	if (monster.spot.row != row) && (monster.spot.col != col)
		&& (level.dungeon[row as usize][col as usize].is_any_door() || level.dungeon[monster.spot.row as usize][monster.spot.col as usize].is_any_door()) {
		return false;
	}
	if monster.target_spot.is_none()
		&& !monster.m_flags.flits
		&& !monster.m_flags.confused
		&& !monster.m_flags.can_flit {
		if (monster.spot.row < player.rogue.row) && (row < monster.spot.row) { return false; }
		if (monster.spot.row > player.rogue.row) && (row > monster.spot.row) { return false; }
		if (monster.spot.col < player.rogue.col) && (col < monster.spot.col) { return false; }
		if (monster.spot.col > player.rogue.col) && (col > monster.spot.col) { return false; }
	}
	if level.dungeon[row as usize][col as usize].has_object() {
		if let Some(obj_id) = ground.find_id_at(row, col) {
			let obj = ground.object(obj_id).expect("object in level_object");
			if obj.what_is == Scroll
				&& ScrollKind::from_index(obj.which_kind as usize) == ScareMonster {
				return false;
			}
		}
	}
	return true;
}

pub fn wake_room(rn: usize, entering: bool, row: i64, col: i64, game: &mut GameState) {
	let normal_chance = game.level.room_wake_percent(rn);
	let buffed_chance = game.player.ring_effects.apply_stealthy(normal_chance);
	for monster in &mut game.mash.monsters {
		if monster.in_room(rn, &game.level) {
			if entering {
				monster.clear_target_spot();
			} else {
				monster.set_target_spot(row, col);
			}
			if monster.m_flags.wakens {
				if rand_percent(buffed_chance) {
					monster.wake_up();
				}
			}
		}
	}
}

pub fn mon_name(monster: &Monster, player: &Player, level: &Level) -> &'static str {
	if player.blind.is_active()
		|| (monster.m_flags.invisible && !player_defeats_invisibility(player, level)) {
		"something"
	} else if player.halluc.is_active() {
		MonsterKind::random_name()
	} else {
		monster.name()
	}
}

pub fn player_defeats_invisibility(player: &Player, level: &Level) -> bool {
	level.detect_monster || level.see_invisible || player.ring_effects.has_see_invisible()
}

fn random_wanderer(level_depth: isize) -> Option<Monster> {
	for _i in 0..15 {
		let monster = gr_monster(level_depth, 0, None);
		if monster.wanders_or_wakens() {
			return Some(monster);
		}
	}
	return None;
}

fn random_spot_for_wanderer(player: &Player, level: &Level) -> Option<DungeonSpot> {
	for _ in 0..25 {
		let spot = gr_spot(
			|cell| cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.has_object(),
			player,
			level,
		);
		if !player.can_see(spot.row, spot.col, level) {
			return Some(spot);
		}
	}
	None
}

pub fn put_wanderer(game: &mut GameState) {
	if let Some(mut monster) = random_wanderer(game.player.cur_depth) {
		monster.wake_up();
		if let Some(spot) = random_spot_for_wanderer(&game.player, &game.level) {
			put_m_at(spot.row, spot.col, monster, &mut game.mash, &mut game.level);
		}
	}
}

pub fn show_monsters(game: &mut GameState) {
	game.level.detect_monster = true;
	if game.player.blind.is_active() {
		return;
	}
	for monster in &mut game.mash.monsters {
		ncurses::mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.m_char());
		if monster.m_flags.imitates {
			monster.m_flags.imitates = false;
			monster.m_flags.wakens = true;
		}
	}
}

fn random_spot_for_monster(start_row: i64, start_col: i64, level: &Level) -> Option<DungeonSpot> {
	let mut walk = RandomWalk::new(start_row, start_col);
	for _ in 0..9 {
		walk.step();
		let spot = walk.spot();
		if spot.is_at(start_row, start_col) || spot.is_out_of_bounds() {
			continue;
		}
		let cell = level.dungeon[spot.row as usize][spot.col as usize];
		if !cell.has_monster() && (cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.is_any_door()) {
			return Some(spot.clone());
		}
	}
	None
}

pub fn create_monster(game: &mut GameState) {
	let player = &game.player;
	if let Some(found) = random_spot_for_monster(player.rogue.row, player.rogue.col, &game.level) {
		let monster = gr_monster(player.cur_depth, 0, None);
		let level = &mut game.level;
		put_m_at(found.row, found.col, monster, &mut game.mash, level);

		let monster = game.mash.monster_at_spot_mut(found.row, found.col).expect("created is in monster in mash");
		ncurses::mvaddch(found.row as i32, found.col as i32, gmc(monster, player, level));
		if monster.wanders_or_wakens() {
			monster.wake_up();
		}
	} else {
		game.dialog.message("you hear a faint cry of anguish in the distance", 0);
	}
}

pub fn put_m_at(row: i64, col: i64, mut monster: Monster, mash: &mut MonsterMash, level: &mut Level) {
	monster.set_spot(row, col);
	level.dungeon[row as usize][col as usize].set_monster(true);
	monster.trail_char = ncurses::mvinch(row as i32, col as i32);
	mash.add_monster(monster);
	if let Some(monster) = mash.monster_at_spot_mut(row, col) {
		aim_monster(monster, level);
	}
}

pub fn move_confused(monster: &mut Monster, player: &Player, level: &mut Level, ground: &ObjectPack) -> bool {
	if !monster.m_flags.asleep {
		monster.decrement_moves_confused();
		if monster.m_flags.stationary {
			return if coin_toss() { true } else { false };
		} else if rand_percent(15) {
			return true;
		} else {
			let mut walk = RandomWalk::new(monster.spot.row, monster.spot.col);
			for _ in 0..9 {
				walk.step();
				let spot = walk.spot();
				if spot.is_at(player.rogue.row, player.rogue.col) {
					return false;
				}
				if mtry(monster, spot.row, spot.col, player, level, ground) {
					return true;
				}
			}
		}
	}
	false
}

pub fn flit(monster: &mut Monster, player: &Player, level: &mut Level, ground: &ObjectPack) -> bool {
	if !rand_percent(odds::FLIT_PERCENT) {
		return false;
	}
	if rand_percent(10) {
		return true;
	}
	let mut walk = RandomWalk::new(monster.spot.row, monster.spot.col);
	for _ in 0..9 {
		walk.step();
		let spot = walk.spot();
		if spot.is_at(player.rogue.row, player.rogue.col) {
			continue;
		}
		if mtry(monster, spot.row, spot.col, player, level, ground) {
			return true;
		}
	}
	true
}

pub fn gr_obj_char() -> chtype {
	const OPTIONS: [char; 9] = ['%', '!', '?', ']', '=', '/', ')', ':', '*'];
	let index = get_rand(0, OPTIONS.len() - 1);
	chtype::from(OPTIONS[index])
}

pub fn aim_monster(monster: &mut Monster, level: &Level) {
	let rn = get_room_number(monster.spot.row, monster.spot.col, level) as usize;
	let r = get_rand(0, 12);
	for i in 0..4 {
		let d = ((r + i) % 4) as usize;
		if level.rooms[rn].doors[d].oth_room.is_some() {
			monster.set_target_spot(
				level.rooms[rn].doors[d].door_row,
				level.rooms[rn].doors[d].door_col,
			);
			break;
		}
	}
}

pub fn no_spot_for_monster(rn: usize, level: &Level) -> bool {
	let room = &level.rooms[rn];
	let floor_bounds = room.to_floor_bounds();
	for row in floor_bounds.rows() {
		for col in floor_bounds.cols() {
			if !level.dungeon[row as usize][col as usize].has_monster() {
				// Found a spot for the monster
				return false;
			}
		}
	}
	return true;
}

pub fn aggravate(game: &mut GameState) {
	game.dialog.message("you hear a high pitched humming noise", 0);
	for monster in &mut game.mash.monsters {
		monster.wake_up();
		monster.m_flags.imitates = false;
		if game.player.can_see(monster.spot.row, monster.spot.col, &game.level) {
			ncurses::mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.m_char());
		}
	}
}

pub fn mv_aquatars(game: &mut GameState) {
	for mon_id in game.mash.monster_ids() {
		let monster = game.mash.monster(mon_id);
		if monster.kind == MonsterKind::Aquator
			&& mon_can_go(monster, game.player.rogue.row, game.player.rogue.col, &game.player, &game.level, &game.ground) {
			mv_monster(mon_id, game.player.rogue.row, game.player.rogue.col, game);
			game.mash.monster_flags_mut(mon_id).already_moved = true;
		}
	}
}
