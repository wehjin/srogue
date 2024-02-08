#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_ushort};
use ncurses::clear;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
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
use crate::prelude::trap_kind::TrapKind::NoTrap;
use crate::room::RoomType::Nothing;
pub use spot::*;

mod spot;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct RogueDepth {
	pub cur: usize,
	pub max: usize,
}

const LAST_DUNGEON: usize = 99;

impl RogueDepth {
	pub fn new() -> Self {
		RogueDepth { cur: 0, max: 1 }
	}
	pub fn descend(&self) -> Self {
		let cur = (self.cur + 1).min(LAST_DUNGEON);
		let max = self.max.max(cur);
		RogueDepth { cur, max }
	}
	pub fn ascend(&self) -> Self {
		let cur = if self.cur < 3 { 1 } else { self.cur - 2 };
		RogueDepth { cur, max: self.max }
	}
}

pub static mut cur_room: i64 = 0;
pub static mut new_level_message: Option<String> = None;
pub static mut party_room: Option<usize> = None;
pub static mut r_de: Option<usize> = None;
pub const LEVEL_POINTS: [isize; MAX_EXP_LEVEL] = [
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

pub fn shuffled_rns() -> [usize; MAX_ROOM] {
	let mut room_indices: [usize; MAX_ROOM] = [3, 7, 5, 2, 0, 6, 1, 4, 8];
	room_indices.shuffle(&mut thread_rng());
	room_indices
}

pub const MAX_ROOM: usize = 9;
pub const MAX_TRAP: usize = 10;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Level {
	pub rooms: [Room; MAX_ROOM],
	pub traps: [Trap; MAX_TRAP],
}

impl Level {
	pub fn new() -> Self {
		Level {
			rooms: [Room::default(); MAX_ROOM],
			traps: [Trap::default(); MAX_TRAP],
		}
	}
	pub fn clear(&mut self) {
		for rn in 0..MAX_ROOM {
			self.rooms[rn].clear();
		}
		for i in 0..MAX_TRAP {
			self.traps[i].clear();
		}
	}
}

pub unsafe fn make_level(level_depth: usize, level: &mut Level) {
	let (must_exist1, must_exist2, must_exist3) = match get_rand(0, 5) {
		0 => (0, 1, 2),
		1 => (3, 4, 5),
		2 => (6, 7, 8),
		3 => (0, 3, 6),
		4 => (1, 4, 7),
		5 => (2, 5, 8),
		_ => unreachable!("0 <= rand <= 5")
	};
	let big_room = level_depth == party_counter && rand_percent(1);
	if big_room {
		make_room(10, 0, 0, 0, level);
	} else {
		for i in 0..MAX_ROOM {
			make_room(i as i64, must_exist1 as i64, must_exist2 as i64, must_exist3 as i64, level);
		}
	}
	if !big_room {
		add_mazes(level_depth, level);

		let shuffled_rns = shuffled_rns();
		for j in 0..MAX_ROOM {
			let i = shuffled_rns[j];
			if i < (MAX_ROOM - 1) {
				connect_rooms(i, i + 1, level_depth, level);
			}
			if i < (MAX_ROOM - 3) {
				connect_rooms(i, i + 3, level_depth, level);
			}
			if i < (MAX_ROOM - 2) {
				if level.rooms[i + 1].room_type.is_nothing() {
					if connect_rooms(i, i + 2, level_depth, level) {
						level.rooms[i + 1].room_type = RoomType::Cross;
					}
				}
			}
			if i < (MAX_ROOM - 6) {
				if level.rooms[i + 3].room_type.is_nothing() {
					if connect_rooms(i, i + 6, level_depth, level) {
						level.rooms[i + 3].room_type = RoomType::Cross;
					}
				}
			}
			if is_all_connected(&level.rooms) {
				break;
			}
			fill_out_level(level, level_depth);
		}
		if !has_amulet() && level_depth >= AMULET_LEVEL {
			put_amulet(&level);
		}
	}
}

pub unsafe fn make_room(rn: i64, r1: i64, r2: i64, r3: i64, level: &mut Level) {
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
		level.rooms[room_index].room_type = RoomType::Room;
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
					Floor as c_ushort
				};
				dungeon[i as usize][j as usize] = ch;
			}
		}
	}
	level.rooms[rn].top_row = top;
	level.rooms[rn].bottom_row = bottom;
	level.rooms[rn].left_col = left;
	level.rooms[rn].right_col = right;
}

pub unsafe fn connect_rooms(room1: usize, room2: usize, level_depth: usize, level: &mut Level) -> bool {
	if !level.rooms[room1].room_type.is_type(&vec![RoomType::Room, RoomType::Maze])
		|| !level.rooms[room2].room_type.is_type(&vec![RoomType::Room, RoomType::Maze]) {
		return false;
	}
	if let Some(dir1) = DoorDirection::from_room_to_room(room1, room2, level) {
		let dir2 = dir1.inverse();
		let spot1 = put_door(&mut level.rooms[room1], dir1, level_depth);
		let spot2 = put_door(&mut level.rooms[room2], dir2, level_depth);
		let mut draw_again = true;
		while draw_again {
			draw_simple_passage(&spot1, &spot2, dir1, level_depth);
			draw_again = rand_percent(4);
		}
		level.rooms[room1].doors[dir1.to_index()].set_others(room2, &spot2);
		level.rooms[room2].doors[dir2.to_index()].set_others(room1, &spot1);
		true
	} else {
		false
	}
}

pub unsafe fn clear_level(level: &mut Level) {
	level.clear();
	for i in 0..DROWS {
		for j in 0..DCOLS {
			SpotFlag::set_nothing(&mut dungeon[i][j]);
		}
	}
	see_invisible = false;
	detect_monster = false;
	bear_trap = 0;
	being_held = false;
	party_room = None;
	rogue.col = -1;
	rogue.row = -1;
	clear();
}

pub unsafe fn put_door(room: &mut Room, door_dir: DoorDirection, level_depth: usize) -> DungeonSpot {
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
	if (level_depth > 2) && rand_percent(HIDE_PERCENT) {
		dungeon[door_spot.row as usize][door_spot.col as usize] |= HIDDEN;
	}
	let door_index = door_dir.to_index();
	room.doors[door_index].door_row = door_spot.row;
	room.doors[door_index].door_col = door_spot.col;
	door_spot
}

pub unsafe fn draw_simple_passage(spot1: &DungeonSpot, spot2: &DungeonSpot, dir: DoorDirection, level_depth: usize) {
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
		hide_boxed_passage(row1, col1, row2, col2, 1, level_depth);
	}
}

pub fn same_row(room1: usize, room2: usize) -> bool {
	room1 / 3 == room2 / 3
}

pub fn same_col(room1: usize, room2: usize) -> bool {
	room1 % 3 == room2 % 3
}

pub unsafe fn add_mazes(level_depth: usize, level: &mut Level) {
	if level_depth > 1 {
		let start = get_rand(0, MAX_ROOM - 1);
		let maze_percent = {
			let mut nominal_percent = (level_depth * 5) / 4;
			if level_depth > 15 {
				nominal_percent + level_depth
			} else {
				nominal_percent
			}
		};

		for i in 0..MAX_ROOM {
			let j = (start + i) % MAX_ROOM;
			if level.rooms[j].room_type.is_nothing() {
				let do_maze = rand_percent(maze_percent);
				if do_maze {
					level.rooms[j].room_type = RoomType::Maze;
					make_maze(
						get_rand(level.rooms[j].top_row + 1, level.rooms[j].bottom_row - 1) as usize,
						get_rand(level.rooms[j].left_col + 1, level.rooms[j].right_col - 1) as usize,
						level.rooms[j].top_row as usize, level.rooms[j].bottom_row as usize,
						level.rooms[j].left_col as usize, level.rooms[j].right_col as usize,
					);
					hide_boxed_passage(level.rooms[j].top_row, level.rooms[j].left_col, level.rooms[j].bottom_row, level.rooms[j].right_col, get_rand(0, 2), level_depth);
				}
			}
		}
	}
}

pub unsafe fn fill_out_level(level: &mut Level, level_depth: usize) {
	let shuffled_rns = shuffled_rns();
	r_de = None;
	for i in 0..MAX_ROOM {
		let rn = shuffled_rns[i];
		if level.rooms[rn].room_type.is_nothing()
			|| (level.rooms[rn].room_type.is_cross() && coin_toss()) {
			fill_it(rn, true, level_depth, level);
		}
	}
	if let Some(deadend_rn) = r_de {
		fill_it(deadend_rn, false, level_depth, level);
	}
}

pub unsafe fn fill_it(rn: usize, do_rec_de: bool, level_depth: usize, level: &mut Level) {
	let mut did_this = false;
	let mut srow: i64 = 0;
	let mut scol: i64 = 0;
	static mut OFFSETS: [isize; 4] = [-1, 1, 3, -3];
	for _ in 0..10 {
		srow = get_rand(0, 3);
		scol = get_rand(0, 3);
		OFFSETS.swap(srow as usize, scol as usize);
	}
	let mut rooms_found = 0;
	for i in 0..4 {
		let target_room = rn as isize + OFFSETS[i];
		if target_room < 0 || target_room >= MAX_ROOM as isize {
			continue;
		}
		let target_room = target_room as usize;
		if !(same_row(rn, target_room) || same_col(rn, target_room)) {
			continue;
		}
		if !level.rooms[target_room].room_type.is_type(&vec![RoomType::Room, RoomType::Maze]) {
			continue;
		}
		let tunnel_dir = get_tunnel_dir(rn, target_room, level);
		let door_dir = tunnel_dir.inverse();
		if level.rooms[target_room].doors[door_dir.to_index()].oth_room.is_some() {
			continue;
		}
		let start_spot = if (!do_rec_de || did_this) || !mask_room(rn, &mut srow, &mut scol, Tunnel as u16, level) {
			level.rooms[rn].center_spot()
		} else {
			DungeonSpot { col: scol, row: srow }
		};
		let end_spot = put_door(&mut level.rooms[target_room], door_dir, level_depth);
		rooms_found += 1;
		draw_simple_passage(&start_spot, &end_spot, tunnel_dir, level_depth);
		level.rooms[rn].room_type = RoomType::DeadEnd;
		dungeon[srow as usize][scol as usize] = Tunnel as u16;
		if (i < 3) && !did_this {
			did_this = true;
			if coin_toss() {
				continue;
			}
		}
		if rooms_found < 2 && do_rec_de {
			recursive_deadend(rn, &OFFSETS, &start_spot, level_depth, level);
		}
		break;
	}
}

pub unsafe fn recursive_deadend(rn: usize, offsets: &[isize; 4], s_spot: &DungeonSpot, level_depth: usize, level: &mut Level)
{
	level.rooms[rn].room_type = RoomType::DeadEnd;
	dungeon[s_spot.row as usize][s_spot.col as usize] = Tunnel as c_ushort;

	for i in 0..4 {
		let de: isize = rn as isize + offsets[i];
		if de < 0 || de >= MAX_ROOM as isize {
			continue;
		}
		let de: usize = de as usize;
		if !same_row(rn, de) && !same_col(rn, de) {
			continue;
		}
		if level.rooms[de].room_type != Nothing {
			continue;
		}
		let d_spot = level.rooms[de].center_spot();
		let tunnel_dir = get_tunnel_dir(rn, de, level);
		draw_simple_passage(&s_spot, &d_spot, tunnel_dir, level_depth);
		r_de = Some(de);
		recursive_deadend(de, offsets, &d_spot, level_depth, level);
	}
}

unsafe fn get_tunnel_dir(rn: usize, de: usize, level: &Level) -> DoorDirection {
	if same_row(rn, de) {
		if level.rooms[rn].left_col < level.rooms[de].left_col { DoorDirection::Right } else { DoorDirection::Left }
	} else {
		if level.rooms[rn].top_row < level.rooms[de].top_row { DoorDirection::Down } else { DoorDirection::Up }
	}
}

pub unsafe fn mask_room(rn: usize, row: &mut i64, col: &mut i64, mask: u16, level: &Level) -> bool {
	for i in level.rooms[rn].top_row..=level.rooms[rn].bottom_row {
		for j in level.rooms[rn].left_col..=level.rooms[rn].right_col {
			if dungeon[i as usize][j as usize] & mask != 0 {
				*row = i;
				*col = j;
				return true;
			}
		}
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
			let swap = dirs[t1];
			dirs[t1] = dirs[t2];
			dirs[t2] = swap;
		}
	}
	for i in 0..4 {
		match dirs[i] {
			DoorDirection::Up => {
				if ((r - 1) >= tr) &&
					(dungeon[r - 1][c] != TUNNEL) &&
					(dungeon[r - 1][c - 1] != TUNNEL) &&
					(dungeon[r - 1][c + 1] != TUNNEL) &&
					(r >= 2 && dungeon[r - 2][c] != TUNNEL) {
					make_maze(r - 1, c, tr, br, lc, rc);
				}
			}
			DoorDirection::Down => {
				if ((r + 1) <= br) &&
					(dungeon[r + 1][c] != TUNNEL) &&
					(dungeon[r + 1][c - 1] != TUNNEL) &&
					(dungeon[r + 1][c + 1] != TUNNEL) &&
					((r + 2 < DROWS) && dungeon[r + 2][c] != TUNNEL) {
					make_maze(r + 1, c, tr, br, lc, rc);
				}
			}
			DoorDirection::Left => {
				if ((c - 1) >= lc) &&
					(dungeon[r][c - 1] != TUNNEL) &&
					(dungeon[r - 1][c - 1] != TUNNEL) &&
					(dungeon[r + 1][c - 1] != TUNNEL) &&
					(c >= 2 && dungeon[r][c - 2] != TUNNEL) {
					make_maze(r, c - 1, tr, br, lc, rc);
				}
			}
			DoorDirection::Right => {
				if ((c + 1) <= rc) &&
					(dungeon[r][c + 1] != TUNNEL) &&
					(dungeon[r - 1][c + 1] != TUNNEL) &&
					(dungeon[r + 1][c + 1] != TUNNEL) &&
					((c + 2) < DCOLS && dungeon[r][c + 2] != TUNNEL) {
					make_maze(r, c + 1, tr, br, lc, rc);
				}
			}
		}
	}
}

pub unsafe fn hide_boxed_passage(row1: i64, col1: i64, row2: i64, col2: i64, n: i64, level_depth: usize) {
	if level_depth > 2 {
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

pub unsafe fn put_player(nr: Option<usize>, level: &Level) {
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	let mut rn = nr;
	for _misses in 0..2 {
		if rn != nr {
			break;
		}
		gr_row_col(&mut row, &mut col, vec![Floor, Tunnel, Object, Stairs], level);
		rn = get_opt_room_number(row, col, level);
	}
	let rn = rn.expect("room number to put player") as i64;
	rogue.row = row;
	rogue.col = col;
	cur_room = if Tunnel.is_set(dungeon[rogue.row as usize][rogue.col as usize]) {
		PASSAGE
	} else {
		rn
	};
	if cur_room != PASSAGE {
		light_up_room(cur_room, level);
	} else {
		light_passage(rogue.row, rogue.col);
	}
	let rn = get_room_number(rogue.row, rogue.col, level);
	wake_room(rn, true, rogue.row, rogue.col, level);
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

pub unsafe fn check_up(game: &mut GameState) -> bool {
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
	if game.depth.cur == 1 {
		win(&game.depth, &game.level);
	} else {
		game.depth = game.depth.ascend();
		return true;
	}
	return false;
}

pub unsafe fn add_exp(e: isize, promotion: bool, level_depth: usize) {
	rogue.exp_points += e;

	if rogue.exp_points >= LEVEL_POINTS[(rogue.exp - 1) as usize] {
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
			print_stats(STAT_HP | STAT_EXP, level_depth);
		}
	} else {
		print_stats(STAT_EXP, level_depth);
	}
}

pub unsafe fn get_exp_level(e: isize) -> isize {
	for i in 0..(MAX_EXP_LEVEL - 1) {
		if LEVEL_POINTS[i] > e {
			return (i + 1) as isize;
		}
	}
	return MAX_EXP_LEVEL as isize;
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
