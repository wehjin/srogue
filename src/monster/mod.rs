#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::chtype;
use crate::message::message;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::room::gr_row_col;


pub mod flags;
mod kind;
mod mash;

use crate::prelude::*;
pub use flags::MonsterFlags;
pub use kind::*;
pub use mash::*;
use crate::{odds};
use crate::level::constants::{DCOLS, DROWS};
use crate::prelude::object_what::ObjectWhat::Scroll;
use crate::prelude::scroll_kind::ScrollKind;
use crate::prelude::scroll_kind::ScrollKind::ScareMonster;
use crate::room::RoomType::Maze;

#[derive(Clone)]
pub struct Fighter {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: isize,
	pub hp_max: isize,
	pub str_current: isize,
	pub str_max: isize,
	pub pack: object,
	pub gold: usize,
	pub exp: isize,
	pub exp_points: isize,
	pub row: i64,
	pub col: i64,
	pub fchar: char,
	pub moves_left: usize,
}

pub static mut mon_disappeared: bool = false;

pub unsafe fn put_mons(level_depth: usize, level: &mut Level) {
	for _ in 0..get_rand(4, 6) {
		let mut monster = gr_monster(level_depth, 0, None);
		if monster.m_flags.wanders && coin_toss() {
			monster.wake_up();
		}
		let DungeonSpot { row, col } = random_spot_with_flag(&[CellKind::Floor, CellKind::Tunnel, CellKind::Stairs, CellKind::Object], level);
		put_m_at(row, col, monster, level);
	}
}

pub fn gr_monster(level_depth: usize, first_level_boost: usize, kind: Option<MonsterKind>) -> Monster {
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

pub unsafe fn mv_mons(player: &Player, level: &mut Level) {
	if haste_self % 2 != 0 {
		return;
	}

	for mut monster in &mut MASH.monsters {
		let mut done_with_monster = false;
		if monster.m_flags.hasted {
			mon_disappeared = false;
			mv_monster(&mut monster, rogue.row, rogue.col, player, level);
			if mon_disappeared {
				done_with_monster = true;
			}
		} else if monster.m_flags.slowed {
			monster.flip_slowed_toggle();
			if monster.slowed_toggle() {
				done_with_monster = true;
			}
		}
		if !done_with_monster && monster.m_flags.confused {
			if move_confused(monster, level) {
				done_with_monster = true;
			}
		}
		if !done_with_monster {
			let mut flew = false;
			if monster.m_flags.flies
				&& !monster.m_flags.napping
				&& !mon_can_go(monster, rogue.row, rogue.col, level) {
				flew = true;
				mv_monster(monster, rogue.row, rogue.col, player, level);
			}
			if !(flew && mon_can_go(&*monster, rogue.row, rogue.col, level)) {
				mv_monster(monster, rogue.row, rogue.col, player, level);
			}
		}
	}
}

pub unsafe fn party_monsters(rn: usize, n: usize, level_depth: usize, level: &mut Level) {
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
			if !dungeon_spot.is_kind(CellKind::Monster) && dungeon_spot.is_any_kind(&[CellKind::Floor, CellKind::Tunnel]) {
				found = Some((row, col));
				break;
			}
		}
		if let Some((row, col)) = found {
			let mut monster = gr_monster(level_depth, first_level_shift, None);
			if !monster.m_flags.imitates {
				monster.m_flags.wakens = true;
			}
			put_m_at(row, col, monster, level);
		}
	}
}

pub unsafe fn gmc_row_col(row: i64, col: i64, level: &Level) -> chtype {
	let monster = MASH.monster_at_spot(row, col);
	if let Some(monster) = monster {
		gmc(monster, level)
	} else {
		ncurses::chtype::from('&')
	}
}

pub unsafe fn gmc(monster: &Monster, level: &Level) -> chtype {
	if (monster.is_invisible() && !player_defeats_invisibility(level))
		|| (blind != 0) {
		monster.trail_char
	} else if monster.m_flags.imitates {
		monster.disguise_char
	} else {
		monster.m_char()
	}
}

pub unsafe fn mv_monster(monster: &mut Monster, row: i64, col: i64, player: &Player, level: &mut Level) {
	if monster.m_flags.asleep {
		if monster.m_flags.napping {
			monster.do_nap();
			return;
		}
		if (monster.m_flags.wakens)
			&& rogue_is_around(monster.spot.row, monster.spot.col)
			&& rand_percent(if stealthy > 0 { odds::WAKE_PERCENT / (odds::STEALTH_FACTOR + (stealthy as usize)) } else { odds::WAKE_PERCENT }) {
			monster.wake_up();
		}
		return;
	} else if monster.m_flags.already_moved {
		monster.m_flags.already_moved = false;
		return;
	}
	if monster.m_flags.flits && flit(monster, level) {
		return;
	}
	if monster.m_flags.stationary && !mon_can_go(monster, rogue.row, rogue.col, level) {
		return;
	}
	if monster.m_flags.freezing_rogue {
		return;
	}
	if monster.m_flags.confuses && m_confuse(monster, level) {
		return;
	}
	if mon_can_go(monster, rogue.row, rogue.col, level) {
		mon_hit(monster, None, false, player, level);
		return;
	}
	if monster.m_flags.flames && flame_broil(monster, player, level) {
		return;
	}
	if monster.m_flags.seeks_gold && seek_gold(monster, player, level) {
		return;
	}

	monster.clear_target_spot_if_reached();
	let target_spot = monster.target_spot_or(DungeonSpot { row, col });
	let row = monster.spot.next_closest_row(target_spot.row);
	if level.dungeon[row as usize][monster.spot.col as usize].is_door() && mtry(monster, row, monster.spot.col, level) {
		return;
	}
	let col = monster.spot.next_closest_col(target_spot.col);
	if level.dungeon[monster.spot.row as usize][col as usize].is_door() && mtry(monster, monster.spot.row, col, level) {
		return;
	}
	if mtry(monster, row, col, level) {
		return;
	}
	for kind in get_rand_indices(6) {
		match kind {
			0 => if mtry(monster, row, monster.spot.col - 1, level) { break; }
			1 => if mtry(monster, row, monster.spot.col, level) { break; }
			2 => if mtry(monster, row, monster.spot.col + 1, level) { break; }
			3 => if mtry(monster, monster.spot.row - 1, col, level) { break; }
			4 => if mtry(monster, monster.spot.row, col, level) { break; }
			5 => if mtry(monster, monster.spot.row + 1, col, level) { break; }
			_ => unreachable!("0 <= n  <= 5")
		}
	}

	// No possible moves
	monster.stuck_counter.log_row_col(monster.spot.row, monster.spot.col);
	if monster.stuck_counter.count > 4 {
		if monster.target_spot.is_none() && !mon_sees(monster, rogue.row, rogue.col, level) {
			monster.set_target_spot(
				get_rand(1, (DROWS - 2) as i64),
				get_rand(0, (DCOLS - 1) as i64),
			);
		} else {
			monster.clear_target_spot();
		}
	}
}

pub unsafe fn mtry(monster: &mut Monster, row: i64, col: i64, level: &mut Level) -> bool {
	if mon_can_go(monster, row, col, level) {
		move_mon_to(monster, row, col, level);
		return true;
	}
	return false;
}

pub unsafe fn move_mon_to(monster: &mut Monster, row: i64, col: i64, level: &mut Level) {
	let (mrow, mcol) = (monster.spot.row, monster.spot.col);
	level.dungeon[mrow as usize][mcol as usize].remove_kind(CellKind::Monster);
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Monster);

	let c = ncurses::mvinch(mrow as i32, mcol as i32);
	if (c >= chtype::from('A')) && (c <= chtype::from('Z')) {
		// Restore the screen appearance at the newly vacated spot
		let exit_trail_char = if !level.detect_monster {
			monster.trail_char
		} else {
			if rogue_can_see(mrow, mcol, level) {
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
	if blind == 0 && (level.detect_monster || rogue_can_see(row, col, level)) {
		if !monster.m_flags.invisible || player_defeats_invisibility(level) {
			ncurses::mvaddch(row as i32, col as i32, gmc(monster, level));
		}
	}
	if level.dungeon[row as usize][col as usize].is_door()
		&& !in_current_room(row, col, level)
		&& level.dungeon[mrow as usize][mcol as usize].is_floor()
		&& blind == 0 {
		ncurses::mvaddch(mrow as i32, mcol as i32, chtype::from(' '));
	}
	if level.dungeon[row as usize][col as usize].is_door() {
		let entering = level.dungeon[monster.spot.row as usize][monster.spot.col as usize].is_tunnel();
		dr_course(monster, entering, row, col, level);
	} else {
		monster.spot.set(row, col);
	}
}

pub unsafe fn mon_can_go(monster: &Monster, row: i64, col: i64, level: &Level) -> bool {
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
	if !is_passable(row, col, level) || level.dungeon[row as usize][col as usize].is_monster() {
		return false;
	}
	if (monster.spot.row != row) && (monster.spot.col != col)
		&& (level.dungeon[row as usize][col as usize].is_door() || level.dungeon[monster.spot.row as usize][monster.spot.col as usize].is_door()) {
		return false;
	}
	if monster.target_spot.is_none()
		&& !monster.m_flags.flits
		&& !monster.m_flags.confused
		&& !monster.m_flags.can_flit {
		if (monster.spot.row < rogue.row) && (row < monster.spot.row) { return false; }
		if (monster.spot.row > rogue.row) && (row > monster.spot.row) { return false; }
		if (monster.spot.col < rogue.col) && (col < monster.spot.col) { return false; }
		if (monster.spot.col > rogue.col) && (col > monster.spot.col) { return false; }
	}
	if level.dungeon[row as usize][col as usize].is_object() {
		let obj = object_at(&level_objects, row, col);
		if (*obj).what_is == Scroll && ScrollKind::from_index((*obj).which_kind as usize) == ScareMonster {
			return false;
		}
	}
	return true;
}

pub unsafe fn wake_room(rn: i64, entering: bool, row: i64, col: i64, level: &Level) {
	let wake_percent = {
		let wake_percent = if Some(rn as usize) == level.party_room {
			odds::PARTY_WAKE_PERCENT
		} else {
			odds::WAKE_PERCENT
		};
		if stealthy > 0 {
			wake_percent / (odds::STEALTH_FACTOR + stealthy as usize)
		} else {
			wake_percent
		}
	};
	for monster in &mut MASH.monsters {
		if monster.in_room(rn, level) {
			if entering {
				monster.clear_target_spot();
			} else {
				monster.set_target_spot(row, col);
			}
		}
		if monster.m_flags.wakens && monster.in_room(rn, level) {
			if rand_percent(wake_percent) {
				monster.wake_up();
			}
		}
	}
}

pub unsafe fn mon_name(monster: &Monster, level: &Level) -> &'static str {
	if player_is_blind() || (monster.m_flags.invisible && !player_defeats_invisibility(level)) {
		"something"
	} else if player_hallucinating() {
		MonsterKind::random_name()
	} else {
		monster.name()
	}
}

pub unsafe fn player_hallucinating() -> bool { halluc != 0 }

pub unsafe fn player_is_blind() -> bool { blind != 0 }

pub unsafe fn player_defeats_invisibility(level: &Level) -> bool { level.detect_monster || level.see_invisible || r_see_invisible }

pub unsafe fn rogue_is_around(row: i64, col: i64) -> bool {
	let row_diff = row - rogue.row;
	let col_diff = col - rogue.col;
	(row_diff >= -1) && (row_diff <= 1) && (col_diff >= -1) && (col_diff <= 1)
}

fn random_wanderer(level_depth: usize) -> Option<Monster> {
	for _i in 0..15 {
		let monster = gr_monster(level_depth, 0, None);
		if monster.wanders_or_wakens() {
			return Some(monster);
		}
	}
	return None;
}

unsafe fn random_spot_for_wanderer(level: &Level) -> Option<DungeonSpot> {
	let mut row = 0;
	let mut col = 0;
	for _ in 0..25 {
		gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Tunnel, CellKind::Stairs, CellKind::Object], level);
		if !rogue_can_see(row, col, level) {
			return Some(DungeonSpot { row, col });
		}
	}
	None
}

pub unsafe fn put_wanderer(level_depth: usize, level: &mut Level) {
	if let Some(mut monster) = random_wanderer(level_depth) {
		monster.wake_up();
		if let Some(spot) = random_spot_for_wanderer(level) {
			put_m_at(spot.row, spot.col, monster, level);
		}
	}
}

pub unsafe fn show_monsters(level: &mut Level) {
	level.detect_monster = true;
	if blind != 0 {
		return;
	}
	for monster in &mut MASH.monsters {
		ncurses::mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.m_char());
		if monster.m_flags.imitates {
			monster.m_flags.imitates = false;
			monster.m_flags.wakens = true;
		}
	}
}

unsafe fn random_spot_for_monster(start_row: i64, start_col: i64, level: &Level) -> Option<DungeonSpot> {
	let mut walk = RandomWalk::new(start_row, start_col);
	for _ in 0..9 {
		walk.step();
		let spot = walk.spot();
		if spot.is_at(start_row, start_col) || spot.is_out_of_bounds() {
			continue;
		}
		const GOOD_CELL_KINDS_FOR_MONSTER: [CellKind; 4] = [CellKind::Floor, CellKind::Tunnel, CellKind::Stairs, CellKind::Door];
		if !level.dungeon[spot.row as usize][spot.col as usize].is_monster()
			&& level.dungeon[spot.row as usize][spot.col as usize].is_any_kind(&GOOD_CELL_KINDS_FOR_MONSTER) {
			return Some(spot.clone());
		}
	}
	None
}

pub unsafe fn create_monster(level_depth: usize, level: &mut Level) {
	if let Some(found) = random_spot_for_monster(rogue.row, rogue.col, level) {
		let mut monster = gr_monster(level_depth, 0, None);
		put_m_at(found.row, found.col, monster, level);

		let monster = MASH.monster_at_spot_mut(found.row, found.col).expect("created is in monster in mash");
		ncurses::mvaddch(found.row as i32, found.col as i32, gmc(monster, level));
		if monster.wanders_or_wakens() {
			monster.wake_up();
		}
	} else {
		message("you hear a faint cry of anguish in the distance", 0);
	}
}

pub unsafe fn put_m_at(row: i64, col: i64, mut monster: Monster, level: &mut Level) {
	monster.set_spot(row, col);
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Monster);
	monster.trail_char = ncurses::mvinch(row as i32, col as i32);
	MASH.add_monster(monster);
	if let Some(monster) = MASH.monster_at_spot_mut(row, col) {
		aim_monster(monster, level);
	}
}

pub unsafe fn rogue_can_see(row: i64, col: i64, level: &Level) -> bool {
	blind == 0
		&& ((in_current_room(row, col, level) && not_in_maze(level)) || is_very_close(row, col))
}

unsafe fn is_very_close(row: i64, col: i64) -> bool {
	rogue_is_around(row, col)
}

unsafe fn not_in_maze(level: &Level) -> bool {
	level.rooms[cur_room as usize].room_type != Maze
}

unsafe fn in_current_room(row: i64, col: i64, level: &Level) -> bool {
	get_room_number(row, col, level) == cur_room
}

pub unsafe fn move_confused(monster: &mut Monster, level: &mut Level) -> bool {
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
				if spot.is_at(rogue.row, rogue.col) {
					return false;
				}
				if mtry(monster, spot.row, spot.col, level) {
					return true;
				}
			}
		}
	}
	false
}

pub unsafe fn flit(monster: &mut Monster, level: &mut Level) -> bool {
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
		if spot.is_at(rogue.row, rogue.col) {
			continue;
		}
		if mtry(monster, spot.row, spot.col, level) {
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

pub unsafe fn aim_monster(monster: &mut Monster, level: &Level) {
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
			if !level.dungeon[row as usize][col as usize].is_monster() {
				// Found a spot for the monster
				return false;
			}
		}
	}
	return true;
}

pub unsafe fn aggravate(level: &Level) {
	message("you hear a high pitched humming noise", 0);
	for monster in &mut MASH.monsters {
		monster.wake_up();
		monster.m_flags.imitates = false;
		if rogue_can_see(monster.spot.row, monster.spot.col, level) {
			ncurses::mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.m_char());
		}
	}
}

pub unsafe fn mon_sees(monster: &Monster, row: i64, col: i64, level: &Level) -> bool {
	if let Some(rn) = monster.in_same_room_as_spot(row, col, level) {
		if level.rooms[rn].room_type != Maze {
			return true;
		}
	}
	let row_diff = row - monster.spot.row;
	let ool_diff = col - monster.spot.col;
	row_diff >= -1 && row_diff <= 1 && ool_diff >= -1 && ool_diff <= 1
}

pub unsafe fn mv_aquatars(player: &Player, level: &mut Level) {
	for monster in &mut MASH.monsters {
		if monster.kind == MonsterKind::Aquator
			&& mon_can_go(monster, rogue.row, rogue.col, level) {
			mv_monster(monster, rogue.row, rogue.col, player, level);
			monster.m_flags.already_moved = true;
		}
	}
}
