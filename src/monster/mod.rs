use rand::{thread_rng, Rng};

use crate::init::{Dungeon, GameState};
use crate::level::Level;
use crate::odds;
use crate::prelude::object_what::ObjectWhat::Scroll;
use crate::prelude::*;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::arena::Arena;
use crate::resources::level::setup::npc;
use crate::resources::physics;
use crate::resources::play::state::RunState;
use crate::room::{get_room_number, gr_spot};
use crate::scrolls::ScrollKind;
use crate::scrolls::ScrollKind::ScareMonster;
use crate::throw::RandomWalk;
pub use flags::MonsterFlags;
pub use kind::*;
pub use mash::*;
use physics::a_moves_b_away_from_c;

pub mod flags;
mod kind;
mod mash;

pub fn put_mons(game: &mut GameState) {
	for _ in 0..get_rand(4, 6) {
		let mut monster = npc::roll_monster(game.player.cur_depth, 0, &mut thread_rng());
		if monster.m_flags.wanders && coin_toss() {
			monster.wake_up();
		}
		let spot = gr_spot(
			|cell| cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.has_object(),
			&game.player,
			&game.level,
		);
		game.airdrop_monster_at(spot.row, spot.col, monster);
	}
}

pub fn party_monsters(rn: usize, n: usize, level_depth: usize, game: &mut GameState, rng: &mut impl Rng) {
	let first_level_shift = level_depth % 3;
	let n = n + n;
	for _i in 0..n {
		if no_spot_for_monster(rn, &game.level) {
			break;
		}
		let mut found: Option<(i64, i64)> = None;
		for _j in 0..250 {
			let row = get_rand(game.level.rooms[rn].top_row + 1, game.level.rooms[rn].bottom_row - 1);
			let col = get_rand(game.level.rooms[rn].left_col + 1, game.level.rooms[rn].right_col - 1);
			let dungeon_spot = game.level.dungeon[row as usize][col as usize];
			if !dungeon_spot.has_monster() && (dungeon_spot.is_any_floor() || dungeon_spot.is_any_tunnel()) {
				found = Some((row, col));
				break;
			}
		}
		if let Some((row, col)) = found {
			let mut monster = npc::roll_monster(level_depth, first_level_shift, rng);
			if !monster.m_flags.imitates {
				monster.m_flags.wakens = true;
			}
			game.airdrop_monster_at(row, col, monster);
		}
	}
}

pub fn mtry(mon_id: MonsterIndex, row: i64, col: i64, allow_any_direction: bool, game: &mut impl Dungeon) -> bool {
	if mon_can_go_and_reach(mon_id, row, col, allow_any_direction, game) {
		game.move_mon_to(mon_id, row, col);
		return true;
	}
	false
}

pub fn mon_can_go_and_reach(mon_id: u64, row: i64, col: i64, allow_any_direction: bool, game: &impl Dungeon) -> bool {
	let monster_row = game.as_monster(mon_id).spot.row;
	let monster_col = game.as_monster(mon_id).spot.col;
	{
		let delta_rows = monster_row as isize - row as isize;        /* check if move distance > 1 */
		let delta_cols = monster_col as isize - col as isize;
		if (delta_rows >= 2) || (delta_rows <= -2) || (delta_cols >= 2) || (delta_cols <= -2) {
			return false;
		}
	}
	if game.is_no_feature_at(monster_row, col) || game.is_no_feature_at(row, monster_col) {
		return false;
	}
	if !game.is_passable_at(row, col) || game.has_monster_at(row, col) {
		return false;
	}
	if (monster_row != row) && (monster_col != col) && (game.is_any_door_at(row, col) || game.is_any_door_at(monster_row, monster_col)) {
		return false;
	}

	let monster = game.as_monster(mon_id);
	let can_move_any_direction = monster.m_flags.flits || monster.m_flags.confused || allow_any_direction;
	if monster.target_spot.is_none() && !can_move_any_direction {
		let rogue_row = game.rogue_row();
		let rogue_col = game.rogue_col();
		let spot_moves_monster_away_from_rogue = a_moves_b_away_from_c((row, col), (monster_row, monster_col), (rogue_row, rogue_col));
		if spot_moves_monster_away_from_rogue {
			return false;
		}
	}
	if let Some(obj) = game.try_object_at(row, col) {
		if Scroll == obj.what_is && ScareMonster == ScrollKind::from_index(obj.which_kind as usize) {
			return false;
		}
	}
	true
}

pub fn mon_name(mon_id: u64, game: &impl Dungeon) -> &'static str {
	if game.as_health().blind.is_active()
		|| (game.as_monster(mon_id).m_flags.invisible && !player_defeats_invisibility(game)) {
		"something"
	} else if game.as_health().halluc.is_active() {
		MonsterKind::random_name()
	} else {
		game.as_monster(mon_id).name()
	}
}

pub fn player_defeats_invisibility(game: &impl Dungeon) -> bool {
	game.detect_monster() || game.see_invisible() || game.as_ring_effects().has_see_invisible()
}

fn random_wanderer(level_depth: usize) -> Option<Monster> {
	for _i in 0..15 {
		let monster = npc::roll_monster(level_depth, 0, &mut thread_rng());
		if monster.wanders_or_wakens() {
			return Some(monster);
		}
	}
	None
}

pub fn put_wanderer(game: &mut impl Dungeon) {
	if let Some(mut monster) = random_wanderer(game.rogue_depth()) {
		monster.wake_up();
		let rng = &mut thread_rng();
		let spot = game.roll_wanderer_spot(rng);
		if let Some(spot) = spot {
			let (row, col) = spot.i64();
			game.airdrop_monster_at(row, col, monster);
		}
	}
}

pub fn show_monsters(game: &mut GameState) {
	game.level.detect_monster = true;
	if game.player.health.blind.is_active() {
		return;
	}
	for mon_id in game.mash.monster_ids() {
		let monster = game.mash.monster(mon_id);
		game.render_spot(monster.spot);
		let monster = game.mash.monster_mut(mon_id);
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
		let monster = npc::roll_monster(player.cur_depth, 0, &mut thread_rng());
		game.airdrop_monster_at(found.row, found.col, monster);
		game.render_spot(found);
		let monster = game.mash.monster_at_spot_mut(found.row, found.col).expect("created is in monster in mash");
		if monster.wanders_or_wakens() {
			monster.wake_up();
		}
	} else {
		game.diary.add_entry("you hear a faint cry of anguish in the distance");
	}
}

pub fn move_confused(mon_id: MonsterIndex, force_flit: bool, game: &mut RunState) -> bool {
	let monster = game.as_monster_mut(mon_id);
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
				let rogue_row = game.rogue_row();
				let rogue_col = game.rogue_col();
				if spot.is_at(rogue_row, rogue_col) {
					return false;
				}
				if mtry(mon_id, spot.row, spot.col, force_flit, game) {
					return true;
				}
			}
		}
	}
	false
}

pub fn flit(mon_id: MonsterIndex, allow_any_direction: bool, game: &mut impl Dungeon) -> bool {
	if !rand_percent(odds::FLIT_PERCENT) {
		return false;
	}
	if rand_percent(10) {
		return true;
	}

	let monster = game.as_monster(mon_id);
	let mut walk = RandomWalk::new(monster.spot.row, monster.spot.col);
	for _ in 0..9 {
		walk.step();
		let spot = walk.spot();
		if spot.is_at(game.rogue_row(), game.rogue_col()) {
			continue;
		}
		if mtry(mon_id, spot.row, spot.col, allow_any_direction, game) {
			return true;
		}
	}
	true
}


pub fn aim_monster(mon_id: u64, game: &mut GameState) {
	let monster = game.as_monster(mon_id);
	let monster_row = monster.spot.row;
	let monster_col = monster.spot.col;
	let rn = get_room_number(monster_row, monster_col, &game.level) as usize;
	let r = get_rand(0, 12);
	for i in 0..4 {
		let d = ((r + i) % 4) as usize;
		if game.level.rooms[rn].doors[d].oth_room.is_some() {
			let door_row = game.level.rooms[rn].doors[d].door_row;
			let door_col = game.level.rooms[rn].doors[d].door_col;
			let monster = game.as_monster_mut(mon_id);
			monster.set_target_spot(door_row, door_col);
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
	true
}

pub fn aggravate(game: &mut GameState) {
	game.diary.add_entry("you hear a high pitched humming noise");
	for monster in game.mash.monster_ids() {
		let monster = game.mash.monster_mut(monster);
		monster.wake_up();
		monster.m_flags.imitates = false;
		let mon_spot = monster.spot;
		if game.player.can_see(mon_spot.row, mon_spot.col, &game.level) {
			game.render_spot(mon_spot);
		}
	}
}

pub fn mv_aquatars(mut game: RunState) -> RunState {
	for mon_id in game.monster_ids() {
		if MonsterKind::Aquator == game.as_monster(mon_id).kind
			&& mon_can_go_and_reach(mon_id, game.rogue_row(), game.rogue_col(), false, &game) {
			// TODO game = mv_monster(game, mon_id, game.player.rogue.row, game.player.rogue.col, false, rng);
			let monster = game.as_monster_flags_mut(mon_id);
			// TODO Make sure already_moved is read somewhere.
			monster.already_moved = true;
		}
	}
	game
}

pub fn mv_aquatars_legacy(game: &mut GameState) {
	let _rng = &mut thread_rng();
	for mon_id in game.mash.monster_ids() {
		if MonsterKind::Aquator == game.mash.monster(mon_id).kind
			&& mon_can_go_and_reach(mon_id, game.player.rogue.row, game.player.rogue.col, false, game) {
			// TODO game = mv_monster(game, mon_id, game.player.rogue.row, game.player.rogue.col, false, rng);
			let monster = game.mash.monster_flags_mut(mon_id);
			monster.already_moved = true;
		}
	}
}
