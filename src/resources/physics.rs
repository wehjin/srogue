use crate::monster::MonsterIndex;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::infra::Infra;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;
use std::ops::RangeInclusive;

pub fn monster_sees_rogue(mon_id: MonsterIndex, arena: &(impl Arena + ?Sized), infra: &(impl Infra + ?Sized)) -> bool {
	let (rogue_row, rogue_col) = (arena.rogue_row(), arena.rogue_col());
	let rogue_room = infra.get_room_at(rogue_row, rogue_col);

	let (monster_row, monster_col) = arena.as_monster(mon_id).spot.into();
	let monster_room = infra.get_room_at(monster_row, monster_col);
	if let Some(room) = same_room(rogue_room, monster_room) {
		if !room.is_maze() {
			return true;
		}
	}
	is_near(rogue_row, rogue_col, monster_row, monster_col)
}

pub fn rogue_sees_spot(spot: LevelSpot, avatar: &(impl Avatar + ?Sized), arena: &(impl Arena + ?Sized), infra: &(impl Infra + ?Sized)) -> bool {
	if avatar.as_health().blind.is_active() {
		false
	} else {
		let (spot_row, spot_col) = spot.i64();
		let spot_room = infra.get_room_at(spot_row, spot_col);
		let (rogue_row, rogue_col) = (arena.rogue_row(), arena.rogue_col());
		let rogue_room = infra.get_room_at(rogue_row, rogue_col);

		if let Some(room) = same_room(rogue_room, spot_room) {
			if room.is_vault() {
				return true;
			}
		}
		// Different rooms or same room but a maze.
		is_near(rogue_row, rogue_col, spot_row, spot_col)
	}
}

pub fn is_near(row: i64, col: i64, other_row: i64, other_col: i64) -> bool {
	let (row_diff, col_diff) = (row - other_row, col - other_col);
	const SIGHT_RANGE: RangeInclusive<i64> = -1..=1;
	let in_sight_range = SIGHT_RANGE.contains(&row_diff) && SIGHT_RANGE.contains(&col_diff);
	in_sight_range
}

pub fn same_room(room: Option<RoomId>, other_room: Option<RoomId>) -> Option<RoomId> {
	let same_room = match (room, other_room) {
		(None, _) => None,
		(_, None) => None,
		(Some(room), Some(other_room)) => room.same_room(other_room),
	};
	same_room
}

pub fn a_moves_b_away_from_c(spot: (i64, i64), b_spot: (i64, i64), c_spot: (i64, i64)) -> bool {
	let (a_row, a_col) = spot;
	let (b_row, b_col) = b_spot;
	let (c_row, c_col) = c_spot;
	let a_row_away_from_c = {
		let a_row_up_when_c_down = (b_row < c_row) && (a_row < b_row);
		let a_row_down_when_c_up = (b_row > c_row) && (a_row > b_row);
		a_row_down_when_c_up || a_row_up_when_c_down
	};
	let a_col_away_from_c = {
		let a_col_left_when_c_right = (b_col < c_col) && (a_col < b_col);
		let a_col_right_when_c_left = (b_col > c_col) && (a_col > b_col);
		a_col_left_when_c_right || a_col_right_when_c_left
	};
	a_row_away_from_c || a_col_away_from_c
}