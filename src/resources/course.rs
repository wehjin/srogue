use crate::init::Dungeon;
use crate::resources::arena::Arena;
use crate::resources::level::room::{ExitSide, ALL_EXIT_SIDES};
use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::DungeonLevel;
use crate::resources::play::state::RunState;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DoorId {
	pub room_id: RoomId,
	pub exit_side: ExitSide,
}
pub fn dr_course(mon_id: u64, is_entering: bool, row: i64, col: i64, game: &mut RunState) {
	let rng = &mut thread_rng();
	game.set_monster_spot(mon_id, row, col);
	if game.monster_sees_rogue(mon_id) {
		game.as_monster_mut(mon_id).clear_target_reset_stuck();
		return;
	}
	let monster_spot = LevelSpot::from_i64(row, col);
	let monster_door = game.level.get_door_at(monster_spot).unwrap();
	if is_entering {
		/* look for door to some other room */
		if let Some(exit_door) = roll_exit_door_for_entering_monster(monster_door, game, rng) {
			let exit_spot = game.level[exit_door].get_near_spot().unwrap();
			let (exit_row, exit_col) = exit_spot.i64();
			game.as_monster_mut(mon_id).set_target_spot(exit_row, exit_col);
			return;
		}
		/* look for door to dead end */
		if let Some(door_spot) = find_other_door_spot_in_same_room_for_entering_monster(monster_door, game) {
			let (exit_row, exit_col) = door_spot.i64();
			game.as_monster_mut(mon_id).set_target_spot(exit_row, exit_col);
			return;
		}
	}
	/*
		when entering, return monster to room that he came from.
		when exiting, send monster to the room at the other side of the passage.
	*/
	if let Some(far_spot) = game.level[monster_door].get_far_spot() {
		let (far_row, far_col) = far_spot.i64();
		game.as_monster_mut(mon_id).set_target_spot(far_row, far_col);
		return;
	}
	/* no place to send monster */
	game.as_monster_mut(mon_id).clear_target_reset_stuck();
}

fn find_other_door_spot_in_same_room_for_entering_monster(monster_door: DoorId, game: &RunState) -> Option<LevelSpot> {
	let monster_spot = game.level[monster_door].get_near_spot().unwrap().clone();
	let monster_room_bounds = &game.level[monster_door.room_id].bounds;
	for spot in monster_room_bounds.to_spots() {
		let not_monster_spot = spot != monster_spot;
		let is_door = game.level.features.feature_at(spot).is_any_door();
		if is_door && not_monster_spot {
			return Some(spot);
		}
	}
	None
}

fn roll_exit_door_for_entering_monster(monster_door: DoorId, game: &RunState, rng: &mut impl Rng) -> Option<DoorId> {
	let monster_spot = game.level[monster_door].get_near_spot().unwrap().clone();
	let monster_room_id = monster_door.room_id;
	for room_id in shuffled_room_ids(&game.level, rng) {
		if !room_id.is_maze_or_vault() || room_id == monster_door.room_id {
			continue;
		}
		let other_room = game.level.as_room(room_id);
		for exit_side in ALL_EXIT_SIDES {
			if other_room[exit_side].leads_to_room(monster_room_id) {
				let spot = other_room[exit_side].get_far_spot().unwrap().clone();
				if monster_spot != spot {
					let door = DoorId { room_id: monster_room_id, exit_side: exit_side.flip() };
					return Some(door);
				}
			}
		}
	}
	None
}

fn shuffled_room_ids(level: &DungeonLevel, rng: &mut impl Rng) -> Vec<RoomId> {
	let mut room_ids = level.all_room_ids();
	room_ids.shuffle(rng);
	room_ids
}
