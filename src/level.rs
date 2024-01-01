#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::mem;
use std::os::raw::c_int;
use libc::{c_short, c_ushort};
use SpotFlag::Door;
use crate::message::{message, print_stats};
use crate::monster::wake_room;
use crate::objects::put_amulet;
use crate::pack::has_amulet;
use crate::random::{get_rand, rand_percent};
use crate::room::{gr_row_col, is_all_connected, light_passage, light_up_room};
use crate::score::win;
use crate::prelude::*;
use crate::prelude::SpotFlag::{Floor, HorWall, Object, Stairs, Tunnel, VertWall};
use crate::prelude::stat_const::{STAT_EXP, STAT_HP};


pub type WINDOW = _win_st;
pub type attr_t = ncurses::chtype;

pub static mut cur_level: isize = 0;
pub static mut max_level: isize = 1;
pub static mut cur_room: i64 = 0;
pub static mut new_level_message: Option<String> = None;
pub static mut party_room: i64 = NO_ROOM;
pub static mut r_de: i64 = 0;
#[no_mangle]
pub static mut level_points: [isize; 21] = [
	10,
	20,
	40,
	80,
	160,
	320,
	640,
	1300,
	2600,
	5200,
	10000,
	20000,
	40000,
	80000,
	160000,
	320000,
	1000000,
	3333333,
	6666666,
	10000000,
	99900000,
];
#[no_mangle]
pub static mut random_rooms: [libc::c_char; 10] = [
	3,
	7,
	5,
	2,
	0,
	6,
	1,
	4,
	8,
	0,
];

pub unsafe fn make_level() {
	let mut i: c_short = 0;
	let mut j: c_short = 0;
	let mut must_exist1: c_short = 0;
	let mut must_exist2: c_short = 0;
	let mut must_exist3: c_short = 0;
	let mut big_room: libc::c_char = 0;
	if (cur_level as i64) < 99 {
		cur_level += 1;
	}
	if cur_level as i64 > max_level as i64 {
		max_level = cur_level;
	}
	must_exist1 = get_rand(0, 5) as c_short;
	match must_exist1 {
		0 => {
			must_exist1 = 0 as i64 as c_short;
			must_exist2 = 1 as c_short;
			must_exist3 = 2 as i64 as c_short;
		}
		1 => {
			must_exist1 = 3 as i64 as c_short;
			must_exist2 = 4 as i64 as c_short;
			must_exist3 = 5 as i64 as c_short;
		}
		2 => {
			must_exist1 = 6 as i64 as c_short;
			must_exist2 = 7 as i64 as c_short;
			must_exist3 = 8 as i64 as c_short;
		}
		3 => {
			must_exist1 = 0 as i64 as c_short;
			must_exist2 = 3 as i64 as c_short;
			must_exist3 = 6 as i64 as c_short;
		}
		4 => {
			must_exist1 = 1 as c_short;
			must_exist2 = 4 as i64 as c_short;
			must_exist3 = 7 as i64 as c_short;
		}
		5 => {
			must_exist1 = 2 as i64 as c_short;
			must_exist2 = 5 as i64 as c_short;
			must_exist3 = 8 as i64 as c_short;
		}
		_ => {}
	}
	big_room = (cur_level as i64 == party_counter as i64
		&& rand_percent(1)) as i64 as libc::c_char;
	if big_room != 0 {
		make_room(
			10 as i64,
			0 as i64,
			0 as i64,
			0 as i64,
		);
	} else {
		i = 0 as i64 as c_short;
		while (i as i64) < 9 as i64 {
			make_room(
				i as i64,
				must_exist1 as i64,
				must_exist2 as i64,
				must_exist3 as i64,
			);
			i += 1;
		}
	}
	if big_room == 0 {
		add_mazes();
		mix_random_rooms();
		j = 0 as i64 as c_short;
		while (j as i64) < 9 as i64 {
			i = random_rooms[j as usize] as c_short;
			if (i as i64) < 9 as i64 - 1 {
				connect_rooms(i as i64, i as i64 + 1);
			}
			if (i as i64) < 9 as i64 - 3 as i64 {
				connect_rooms(i as i64, i as i64 + 3 as i64);
			}
			if (i as i64) < 9 as i64 - 2 as i64 {
				if (*rooms
					.as_mut_ptr()
					.offset((i as i64 + 1) as isize))
					.room_type as i64
					& 0o1 as c_ushort as i64 != 0
				{
					if connect_rooms(i as i64, i as i64 + 2 as i64) {
						(*rooms
							.as_mut_ptr()
							.offset((i as i64 + 1) as isize))
							.room_type = RoomType::Room;
					}
				}
			}
			if (i as i64) < 9 as i64 - 6 as i64 {
				if (*rooms
					.as_mut_ptr()
					.offset((i as i64 + 3 as i64) as isize))
					.room_type as i64
					& 0o1 as c_ushort as i64 != 0
				{
					if connect_rooms(i as i64, i as i64 + 6 as i64) {
						(*rooms
							.as_mut_ptr()
							.offset((i as i64 + 3 as i64) as isize))
							.room_type = RoomType::Room;
					}
				}
			}
			if is_all_connected() {
				break;
			}
			j += 1;
		}
		fill_out_level();
	}
	if !has_amulet() && cur_level >= AMULET_LEVEL {
		put_amulet();
	}
}

pub unsafe fn make_room(rn: i64, r1: i64, r2: i64, r3: i64) {
	let rn: usize = rn as usize;
	let (left, right, top, bottom, do_shrink, room_index) =
		match rn {
			0 => (0, COL1 - 1, MIN_ROW, ROW1 - 1, true, rn),
			1 => (COL1 + 1, COL2 - 1, MIN_ROW, ROW1 - 1, true, rn),
			2 => (COL2 + 1, DCOLS as i64 - 1, MIN_ROW, ROW1 - 1, true, rn),
			3 => (0, COL1 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			4 => (COL1 + 1, COL2 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			5 => (COL2 + 1, DCOLS as i64 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			6 => (0, COL1 - 1, ROW2 + 1, DROWS as i64 - 2, true, rn),
			7 => (COL1 + 1, COL2 - 1, ROW2 + 1, DROWS as i64 - 2, true, rn),
			8 => (COL2 + 1, DCOLS as i64 - 1, ROW2 + 1, DROWS as i64 - 2, true, rn),
			BIG_ROOM => {
				let top = get_rand(MIN_ROW, MIN_ROW + 5);
				let bottom = get_rand(DROWS as i64 - 7, DROWS as i64 - 2);
				let left = get_rand(0, 10);
				let right = get_rand(DCOLS as i64 - 11, DCOLS as i64 - 1);
				(left, right, top, bottom, false, 0)
			}
			_ => panic!("Invalid value in parameter rn")
		};
	let (left, right, top, bottom, fill_dungeon) = if do_shrink {
		let height = get_rand(4, bottom - top + 1);
		let width = get_rand(7, (right - left) - 2);
		let row_offset = get_rand(0, (bottom - top) - height + 1);
		let col_offset = get_rand(0, (right - left) - width + 1);
		let top = top + row_offset;
		let bottom = top + height - 1;
		let left = left + col_offset;
		let right = left + width - 1;
		let skip_walls = (room_index != r1 as usize) && (room_index != r2 as usize) && (room_index != r3 as usize) && rand_percent(40);
		(left, right, top, bottom, !skip_walls)
	} else {
		(left, right, top, bottom, true)
	};
	let (left, right, top, bottom) = (left, right, top, bottom);
	if fill_dungeon {
		rooms[room_index].room_type = RoomType::Room;
		for i in top..(bottom + 1) {
			let top_border = i == top;
			let bottom_border = i == bottom;
			for j in left..(right + 1) {
				let left_border = j == left;
				let right_border = j == right;
				let ch = if top_border || bottom_border {
					HorWall as c_ushort
				} else if !top_border && !bottom_border && (left_border || right_border) {
					VertWall as c_ushort
				} else {
					SpotFlag::Floor as c_ushort
				};
				dungeon[i as usize][j as usize] = ch;
			}
		}
	}
	rooms[rn as usize].top_row = top as i64;
	rooms[rn as usize].bottom_row = bottom as i64;
	rooms[rn as usize].left_col = left as i64;
	rooms[rn as usize].right_col = right as i64;
}

pub unsafe fn connect_rooms(room1: i64, room2: i64) -> bool {
	let (room1, room2) = (room1, room2);
	if rooms[room1 as usize].room_type.is_nothing() || rooms[room2 as usize].room_type.is_nothing() {
		return false;
	}
	let dir1 = if same_row(room1, room2) && (rooms[room1 as usize].left_col > rooms[room2 as usize].right_col) {
		DoorDirection::Left
	} else if same_row(room1, room2) && (rooms[room2 as usize].left_col > rooms[room1 as usize].right_col) {
		DoorDirection::Right
	} else if same_col(room1, room2) && (rooms[room1 as usize].top_row > rooms[room2 as usize].bottom_row) {
		DoorDirection::Up
	} else if same_col(room1, room2) && (rooms[room2 as usize].top_row > rooms[room1 as usize].bottom_row) {
		DoorDirection::Down
	} else {
		return false;
	};
	let dir2 = dir1.to_inverse();
	let spot1 = put_door(&mut rooms[room1 as usize], dir1);
	let spot2 = put_door(&mut rooms[room2 as usize], dir2);
	let mut do_draw = true;
	while do_draw {
		draw_simple_passage(&spot1, &spot2, dir1);
		do_draw = rand_percent(4);
	}

	let door1_index = dir1.to_index();
	let door2_index = dir2.to_index();
	rooms[room1 as usize].doors[door1_index].oth_room = Some(room2);
	rooms[room1 as usize].doors[door1_index].oth_row = Some(spot2.row);
	rooms[room1 as usize].doors[door1_index].oth_col = Some(spot2.col);
	rooms[room2 as usize].doors[door2_index].oth_room = Some(room1);
	rooms[room2 as usize].doors[door2_index].oth_row = Some(spot1.row);
	rooms[room2 as usize].doors[door2_index].oth_col = Some(spot1.col);
	return true;
}

#[no_mangle]
pub unsafe extern "C" fn clear_level() -> i64 {
	let mut i: c_short = 0;
	let mut j: c_short = 0;
	i = 0 as i64 as c_short;
	while (i as i64) < 9 as i64 {
		(*rooms.as_mut_ptr().offset(i as isize)).room_type = RoomType::Nothing;
		j = 0 as i64 as c_short;
		while (j as i64) < 4 as i64 {
			(*rooms.as_mut_ptr().offset(i as isize)).doors[j as usize].oth_room = None;
			j += 1;
		}
		i += 1;
	}
	i = 0 as i64 as c_short;
	while (i as i64) < 10 as i64 {
		(*traps.as_mut_ptr().offset(i as isize))
			.trap_type = -(1) as c_short;
		i += 1;
	}
	i = 0 as i64 as c_short;
	while (i as i64) < 24 as i64 {
		j = 0 as i64 as c_short;
		while (j as i64) < 80 as i64 {
			dungeon[i as usize][j as usize] = 0 as i64 as c_ushort;
			j += 1;
		}
		i += 1;
	}
	see_invisible = false;
	detect_monster = see_invisible;
	bear_trap = 0;
	being_held = bear_trap > 0;
	party_room = NO_ROOM;
	rogue.col = -1;
	rogue.row = rogue.col;
	ncurses::wclear(ncurses::stdscr());
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn put_door(room: &mut room, door_dir: DoorDirection) -> DungeonSpot {
	let wall_width = if RoomType::Maze == room.room_type { 0 } else { 1 };
	let door_spot = match door_dir {
		DoorDirection::Up | DoorDirection::Down => {
			let row = if door_dir == DoorDirection::Up { room.top_row } else { room.bottom_row };
			let mut col;
			loop {
				col = get_rand(room.left_col + wall_width, room.right_col - wall_width);
				if SpotFlag::is_any_set(&vec![HorWall, Tunnel], dungeon[row as usize][col as usize]) {
					break;
				}
			}
			DungeonSpot { row, col }
		}
		DoorDirection::Left | DoorDirection::Right => {
			let col = if door_dir == DoorDirection::Left { room.left_col } else { room.right_col };
			let mut row;
			loop {
				row = get_rand(room.top_row + wall_width, room.bottom_row - wall_width);
				if SpotFlag::is_any_set(&vec![VertWall, Tunnel], dungeon[row as usize][col as usize]) {
					break;
				}
			}
			DungeonSpot { row, col }
		}
	};
	if room.room_type == RoomType::Room {
		dungeon[door_spot.row as usize][door_spot.col as usize] = Door as c_ushort;
	}
	if (cur_level > 2) && rand_percent(HIDE_PERCENT) {
		dungeon[door_spot.row as usize][door_spot.col as usize] |= HIDDEN;
	}
	let door_index = door_dir.to_index();
	room.doors[door_index].door_row = door_spot.row;
	room.doors[door_index].door_col = door_spot.col;
	door_spot
}

pub unsafe fn draw_simple_passage(spot1: &DungeonSpot, spot2: &DungeonSpot, dir: DoorDirection) {
	let (col1, row1, col2, row2) =
		match dir {
			DoorDirection::Left | DoorDirection::Right => {
				let (col1, row1, col2, row2) = if spot1.col > spot2.col {
					(spot2.col, spot2.row, spot1.col, spot1.row)
				} else {
					(spot1.col, spot1.row, spot2.col, spot2.row)
				};
				let middle = get_rand(col1 + 1, col2 - 1);
				for i in (col1 + 1)..middle {
					dungeon[row1 as usize][i as usize] = TUNNEL;
				}
				let mut i = row1;
				let step = if row1 > row2 { -1 } else { 1 };
				while i != row2 {
					dungeon[i as usize][middle as usize] = TUNNEL;
					i = (i) + step;
				}
				for i in middle..col2 {
					dungeon[row2 as usize][i as usize] = TUNNEL;
				}
				(col1, row1, col2, row2)
			}
			DoorDirection::Up | DoorDirection::Down => {
				let (col1, row1, col2, row2) = if spot1.row > spot2.row {
					(spot2.col, spot2.row, spot1.col, spot1.row)
				} else {
					(spot1.col, spot1.row, spot2.col, spot2.row)
				};
				let middle = get_rand(row1 + 1, row2 - 1);
				for i in (row1 + 1)..middle {
					dungeon[i as usize][col1 as usize] = TUNNEL;
				}
				let mut i = col1;
				let step = if col1 > col2 { -1 } else { 1 };
				while i != col2 {
					dungeon[middle as usize][i as usize] = TUNNEL;
					i = (i) + step;
				}
				for i in middle..row2 {
					dungeon[i as usize][col2 as usize] = TUNNEL;
				}
				(col1, row1, col2, row2)
			}
		};
	if rand_percent(HIDE_PERCENT) {
		hide_boxed_passage(row1, col1, row2, col2, 1);
	}
}


pub fn same_row(room1: i64, room2: i64) -> bool {
	room1 / 3 == room2 / 3
}

pub fn same_col(room1: i64, room2: i64) -> bool {
	room1 % 3 == room2 % 3
}

pub unsafe fn add_mazes() {
	if cur_level > 1 {
		let start = get_rand(0, MAXROOMS - 1);
		let maze_percent = {
			let mut nominal_percent = (cur_level * 5) / 4;
			if cur_level > 15 {
				nominal_percent + cur_level
			} else {
				nominal_percent
			}
		} as usize;

		for i in 0..MAXROOMS {
			let j = (start + i) % MAXROOMS;
			if rooms[j as usize].room_type.is_nothing() {
				let do_maze = rand_percent(maze_percent);
				if do_maze {
					rooms[j as usize].room_type = RoomType::Maze;
					make_maze(
						get_rand(rooms[j as usize].top_row + 1, rooms[j as usize].bottom_row - 1) as usize,
						get_rand(rooms[j as usize].left_col + 1, rooms[j as usize].right_col - 1) as usize,
						rooms[j as usize].top_row as usize, rooms[j as usize].bottom_row as usize,
						rooms[j as usize].left_col as usize, rooms[j as usize].right_col as usize,
					);
					hide_boxed_passage(rooms[j as usize].top_row, rooms[j as usize].left_col, rooms[j as usize].bottom_row, rooms[j as usize].right_col, get_rand(0, 2));
				}
			}
		}
	}
}

pub unsafe fn fill_out_level() {
	mix_random_rooms();
	r_de = NO_ROOM;

	for i in 0..MAXROOMS {
		let rn = random_rooms[i as usize] as i64;
		// Note: original C uses (rooms[rn as usize].is_room & R_NOTHING) for the
		// first clause of the conditional. Since R_NOTHING is 0, the first clause would always evaluate
		// to false.
		// Question is, was that a bad bug that should be fixed? Or was it a good bug that will break this
		// function if fixed.  For now, we will fix the bug.
		if (rooms[rn as usize].room_type == RoomType::Nothing) || ((rooms[rn as usize].room_type == RoomType::Cross) && coin_toss()) {
			fill_it(rn, true);
		}
	}
	if r_de != NO_ROOM {
		fill_it(r_de, false);
	}
}


pub unsafe fn fill_it(rn: i64, do_rec_de: bool) {
	let mut did_this = false;
	static mut OFFSETS: [i64; 4] = [-1, 1, 3, -3];

	let mut srow: i64 = 0;
	let mut scol: i64 = 0;
	for _ in 0..10 {
		srow = get_rand(0, 3);
		scol = get_rand(0, 3);
		let t = OFFSETS[srow as usize];
		OFFSETS[srow as usize] = OFFSETS[scol as usize];
		OFFSETS[scol as usize] = t;
	}
	for i in 0..4 {
		let target_room = rn + OFFSETS[i];
		let mut rooms_found: usize = 0;

		if ((target_room < 0) || (target_room >= MAXROOMS))
			|| (!same_row(rn, target_room) && !same_col(rn, target_room))
			|| (rooms[target_room as usize].room_type != RoomType::Room && rooms[target_room as usize].room_type != RoomType::Maze) {
			continue;
		}
		let tunnel_dir = get_tunnel_dir(rn, target_room);
		let door_dir = tunnel_dir.to_inverse();
		if rooms[target_room as usize].doors[door_dir.to_index()].oth_room.is_some() {
			continue;
		}
		let (mut srow, mut scol) = (srow, scol);
		if ((!do_rec_de) || did_this) || (!mask_room(rn, &mut srow, &mut scol, Tunnel as c_ushort)) {
			srow = (rooms[rn as usize].top_row + rooms[rn as usize].bottom_row) / 2;
			scol = (rooms[rn as usize].left_col + rooms[rn as usize].right_col) / 2;
		}
		let d_spot = put_door(&mut rooms[target_room as usize], door_dir);
		rooms_found += 1;
		let s_spot = DungeonSpot { col: scol, row: srow };
		draw_simple_passage(&s_spot, &d_spot, tunnel_dir);
		rooms[rn as usize].room_type = RoomType::DeadEnd;
		dungeon[srow as usize][scol as usize] = Tunnel as c_ushort;

		if (i < 3) && !did_this {
			did_this = true;
			if coin_toss() {
				continue;
			}
		}
		if (rooms_found < 2) && do_rec_de {
			recursive_deadend(rn, &OFFSETS, &s_spot);
		}
		break;
	}
}

pub unsafe fn recursive_deadend(rn: i64, offsets: &[i64; 4], s_spot: &DungeonSpot)
{
	rooms[rn as usize].room_type = RoomType::DeadEnd;
	dungeon[s_spot.row as usize][s_spot.col as usize] = Tunnel as c_ushort;

	for i in 0..4 {
		let de = rn + offsets[i];
		if ((de < 0) || (de >= MAXROOMS))
			|| (!same_row(rn, de) && !same_col(rn, de)) {
			continue;
		}
		if rooms[de as usize].room_type != RoomType::Nothing {
			continue;
		}
		let d_spot = DungeonSpot {
			col: (rooms[de as usize].left_col + rooms[de as usize].right_col) / 2,
			row: (rooms[de as usize].top_row + rooms[de as usize].bottom_row) / 2,
		};
		let tunnel_dir = get_tunnel_dir(rn, de);
		draw_simple_passage(&s_spot, &d_spot, tunnel_dir);
		r_de = de;
		recursive_deadend(de, offsets, &d_spot);
	}
}

unsafe fn get_tunnel_dir(rn: i64, de: i64) -> DoorDirection {
	if same_row(rn, de) {
		if rooms[rn as usize].left_col < rooms[de as usize].left_col { DoorDirection::Right } else { DoorDirection::Left }
	} else {
		if rooms[rn as usize].top_row < rooms[de as usize].top_row { DoorDirection::Down } else { DoorDirection::Up }
	}
}


#[no_mangle]
pub unsafe extern "C" fn mask_room(
	mut rn: i64,
	mut row: *mut i64,
	mut col: *mut i64,
	mut mask: c_ushort,
) -> bool {
	let mut i = 0;
	let mut j = 0;
	i = (*rooms.as_mut_ptr().offset(rn as isize)).top_row;
	while i
		<= (*rooms.as_mut_ptr().offset(rn as isize)).bottom_row
	{
		j = (*rooms.as_mut_ptr().offset(rn as isize)).left_col;
		while j
			<= (*rooms.as_mut_ptr().offset(rn as isize)).right_col
		{
			if dungeon[i as usize][j as usize] as i64 & mask as i64 != 0
			{
				*row = i;
				*col = j;
				return true;
			}
			j += 1;
		}
		i += 1;
	}
	return false;
}

const TUNNEL: c_ushort = Tunnel as c_ushort;
const HIDDEN: c_ushort = SpotFlag::Hidden as c_ushort;

pub unsafe fn make_maze(r: usize, c: usize, tr: usize, br: usize, lc: usize, rc: usize) {
	let mut dirs: [DoorDirection; 4] = [DoorDirection::Up, DoorDirection::Down, DoorDirection::Left, DoorDirection::Right];
	dungeon[r][c] = TUNNEL;

	if rand_percent(33) {
		for _i in 0..10 {
			let t1 = get_rand(0, 3) as usize;
			let t2 = get_rand(0, 3) as usize;
			mem::swap(&mut dirs[t1], &mut dirs[t2]);
		}
	}
	for i in 0..4 {
		match dirs[i] {
			DoorDirection::Up => {
				if ((r - 1) >= tr) &&
					(dungeon[r - 1][c] != TUNNEL) &&
					(dungeon[r - 1][c - 1] != TUNNEL) &&
					(dungeon[r - 1][c + 1] != TUNNEL) &&
					(dungeon[r - 2][c] != TUNNEL) {
					make_maze(r - 1, c, tr, br, lc, rc);
				}
			}
			DoorDirection::Down => {
				if ((r + 1) <= br) &&
					(dungeon[r + 1][c] != TUNNEL) &&
					(dungeon[r + 1][c - 1] != TUNNEL) &&
					(dungeon[r + 1][c + 1] != TUNNEL) &&
					(dungeon[r + 2][c] != TUNNEL) {
					make_maze(r + 1, c, tr, br, lc, rc);
				}
			}
			DoorDirection::Left => {
				if ((c - 1) >= lc) &&
					(dungeon[r][c - 1] != TUNNEL) &&
					(dungeon[r - 1][c - 1] != TUNNEL) &&
					(dungeon[r + 1][c - 1] != TUNNEL) &&
					(dungeon[r][c - 2] != TUNNEL) {
					make_maze(r, c - 1, tr, br, lc, rc);
				}
			}
			DoorDirection::Right => {
				if ((c + 1) <= rc) &&
					(dungeon[r][c + 1] != TUNNEL) &&
					(dungeon[r - 1][c + 1] != TUNNEL) &&
					(dungeon[r + 1][c + 1] != TUNNEL) &&
					(dungeon[r][c + 2] != TUNNEL) {
					make_maze(r, c + 1, tr, br, lc, rc);
				}
			}
		}
	}
}

pub unsafe fn hide_boxed_passage(row1: i64, col1: i64, row2: i64, col2: i64, n: i64) {
	if cur_level > 2 {
		let (row1, row2) = if row1 > row2 { (row2, row1) } else { (row1, row2) };
		let (col1, col2) = if col1 > col2 { (col2, col1) } else { (col1, col2) };
		let h = row2 - row1;
		let w = col2 - col1;

		if (w >= 5) || (h >= 5) {
			let row_cut = match h >= 2 {
				true => 1,
				false => 0,
			};
			let col_cut = match w >= 2 {
				true => 1,
				false => 0,
			};
			for _i in 0..n {
				for _j in 0..10 {
					let row = get_rand(row1 + row_cut, row2 - row_cut) as usize;
					let col = get_rand(col1 + col_cut, col2 - col_cut) as usize;
					if dungeon[row][col] == TUNNEL {
						dungeon[row][col] |= HIDDEN;
						break;
					}
				}
			}
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn put_player(mut nr: i64) {
	let mut rn: i64 = nr;
	let mut misses: c_short = 0;
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	misses = 0;
	while (misses as i64) < 2
		&& rn == nr
	{
		gr_row_col(&mut row, &mut col, vec![Floor, Tunnel, Object, Stairs]);
		rn = get_room_number(row, col);
		misses += 1;
	}
	rogue.row = row;
	rogue.col = col;
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o200 as libc::c_int as c_ushort as libc::c_int != 0
	{
		cur_room = PASSAGE;
	} else {
		cur_room = rn;
	}
	if cur_room as libc::c_int != -(3 as libc::c_int) {
		light_up_room(cur_room);
	} else {
		light_passage(rogue.row, rogue.col);
	}
	wake_room(get_room_number(rogue.row, rogue.col), true, rogue.row, rogue.col);
	if let Some(msg) = &new_level_message {
		message(msg, 0);
		new_level_message = None;
	}
	ncurses::mvaddch(rogue.row as i32, rogue.col as i32, rogue.fchar as ncurses::chtype);
}

#[no_mangle]
pub unsafe extern "C" fn drop_check() -> bool {
	if wizard {
		return true;
	}
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o4 as libc::c_int as c_ushort as libc::c_int != 0
	{
		if levitate != 0 {
			message("you're floating in the air!", 0);
			return false;
		}
		return true;
	}
	message("I see no way down", 0);
	return false;
}

pub unsafe fn check_up() -> bool {
	if !wizard {
		if !Stairs.is_set(dungeon[rogue.row as usize][rogue.col as usize]) {
			message("I see no way up", 0);
			return false;
		}
		if !has_amulet() {
			message("Your way is magically blocked", 0);
			return false;
		}
	}
	new_level_message = Some("you feel a wrenching sensation in your gut".to_string());
	if cur_level == 1 {
		win();
	} else {
		cur_level -= 2;
		return true;
	}
	return false;
}

pub unsafe fn add_exp(e: isize, promotion: bool) {
	rogue.exp_points += e;

	if rogue.exp_points >= level_points[(rogue.exp - 1) as usize] {
		let new_exp = get_exp_level(rogue.exp_points);
		if rogue.exp_points > MAX_EXP {
			rogue.exp_points = MAX_EXP + 1;
		}
		for i in (rogue.exp + 1)..=new_exp {
			let msg = format!("welcome to level {}", i);
			message(&msg, 0);
			if promotion {
				let hp = hp_raise();
				rogue.hp_current += hp;
				rogue.hp_max += hp;
			}
			rogue.exp = i;
			print_stats(STAT_HP | STAT_EXP);
		}
	} else {
		print_stats(STAT_EXP);
	}
}

pub unsafe fn get_exp_level(e: isize) -> isize {
	for i in 0..(MAX_EXP_LEVEL - 1) {
		if level_points[i as usize] > e {
			return i + 1;
		}
	}
	return MAX_EXP_LEVEL;
}

pub unsafe fn hp_raise() -> isize {
	if wizard {
		10
	} else {
		get_rand(3, 10) as isize
	}
}

#[no_mangle]
pub unsafe extern "C" fn show_average_hp() {
	let (real_average, effective_average) = if rogue.exp == 1 {
		(0.0, 0.0)
	} else {
		let real = (rogue.hp_max - extra_hp - INIT_HP + less_hp) as f32 / (rogue.exp - 1) as f32;
		let average = (rogue.hp_max - INIT_HP) as f32 / (rogue.exp - 1) as f32;
		(real, average)
	};
	let msg = format!("R-Hp: {:.2}, E-Hp: {:.2} (!: {}, V: {})", real_average, effective_average, extra_hp, less_hp);
	message(&msg, 0);
}

pub unsafe fn mix_random_rooms() {
	for _i in 0..(3 * MAXROOMS) {
		let mut x: usize = 0;
		let mut y: usize = 0;
		loop {
			x = get_rand(0, (MAXROOMS - 1) as c_int) as usize;
			y = get_rand(0, (MAXROOMS - 1) as c_int) as usize;
			if x != y {
				break;
			}
		}
		mem::swap(&mut random_rooms[x], &mut random_rooms[y]);
	}
}
