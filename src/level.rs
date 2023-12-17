#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::mem;
use std::os::raw::c_int;
use libc::{c_long, c_short, c_ushort};
use SpotFlag::Door;
use crate::message::{message, print_stats};
use crate::monster::wake_room;
use crate::objects::put_amulet;
use crate::pack::has_amulet;
use crate::random::{get_rand, rand_percent};
use crate::room::{gr_row_col, is_all_connected, light_passage, light_up_room};
use crate::score::win;

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn wclear(_: *mut WINDOW) -> libc::c_int;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut rooms: [room; 0];
	static mut traps: [trap; 0];
	static mut dungeon: [[c_ushort; 80]; 24];
	static mut being_held: libc::c_char;
	static mut wizard: libc::c_char;
	static mut detect_monster: libc::c_char;
	static mut see_invisible: libc::c_char;
	static mut bear_trap: c_short;
	static mut levitate: c_short;
	static mut extra_hp: c_short;
	static mut less_hp: c_short;
	static mut party_counter: c_short;
}

use crate::prelude::*;

pub type chtype = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: c_short,
	pub _curx: c_short,
	pub _maxy: c_short,
	pub _maxx: c_short,
	pub _begy: c_short,
	pub _begx: c_short,
	pub _flags: c_short,
	pub _attrs: attr_t,
	pub _bkgd: chtype,
	pub _notimeout: libc::c_int,
	pub _clear: libc::c_int,
	pub _leaveok: libc::c_int,
	pub _scroll: libc::c_int,
	pub _idlok: libc::c_int,
	pub _idcok: libc::c_int,
	pub _immed: libc::c_int,
	pub _sync: libc::c_int,
	pub _use_keypad: libc::c_int,
	pub _delay: libc::c_int,
	pub _line: *mut ldat,
	pub _regtop: c_short,
	pub _regbottom: c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: c_short,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: c_short,
	pub _pad_x: c_short,
	pub _pad_top: c_short,
	pub _pad_left: c_short,
	pub _pad_bottom: c_short,
	pub _pad_right: c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = chtype;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: c_short,
	pub hp_max: c_short,
	pub str_current: c_short,
	pub str_max: c_short,
	pub pack: object,
	pub gold: c_long,
	pub exp: c_short,
	pub exp_points: c_long,
	pub row: c_short,
	pub col: c_short,
	pub fchar: c_short,
	pub moves_left: c_short,
}

pub type fighter = fight;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dr {
	pub oth_room: c_short,
	pub oth_row: c_short,
	pub oth_col: c_short,
	pub door_row: c_short,
	pub door_col: c_short,
}

pub type door = dr;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct tr {
	pub trap_type: c_short,
	pub trap_row: c_short,
	pub trap_col: c_short,
}

pub type trap = tr;

#[no_mangle]
pub static mut cur_level: c_short = 0 as libc::c_int as c_short;
#[no_mangle]
pub static mut max_level: c_short = 1 as libc::c_int as c_short;
#[no_mangle]
pub static mut cur_room: c_short = 0;
#[no_mangle]
pub static mut new_level_message: *mut libc::c_char = 0 as *const libc::c_char
	as *mut libc::c_char;
#[no_mangle]
pub static mut party_room: c_short = -(1 as libc::c_int) as c_short;
#[no_mangle]
pub static mut r_de: c_short = 0;
#[no_mangle]
pub static mut level_points: [c_long; 21] = [
	10 as c_long,
	20 as c_long,
	40 as c_long,
	80 as c_long,
	160 as c_long,
	320 as c_long,
	640 as c_long,
	1300 as c_long,
	2600 as c_long,
	5200 as c_long,
	10000 as c_long,
	20000 as c_long,
	40000 as c_long,
	80000 as c_long,
	160000 as c_long,
	320000 as c_long,
	1000000 as c_long,
	3333333 as c_long,
	6666666 as c_long,
	10000000 as c_long,
	99900000 as c_long,
];
#[no_mangle]
pub static mut random_rooms: [libc::c_char; 10] = [
	3 as libc::c_int as libc::c_char,
	7 as libc::c_int as libc::c_char,
	5 as libc::c_int as libc::c_char,
	2 as libc::c_int as libc::c_char,
	0 as libc::c_int as libc::c_char,
	6 as libc::c_int as libc::c_char,
	1 as libc::c_int as libc::c_char,
	4 as libc::c_int as libc::c_char,
	8 as libc::c_int as libc::c_char,
	0,
];

#[no_mangle]
pub unsafe extern "C" fn make_level() -> libc::c_int {
	let mut i: c_short = 0;
	let mut j: c_short = 0;
	let mut must_exist1: c_short = 0;
	let mut must_exist2: c_short = 0;
	let mut must_exist3: c_short = 0;
	let mut big_room: libc::c_char = 0;
	if (cur_level as libc::c_int) < 99 as libc::c_int {
		cur_level += 1;
	}
	if cur_level as libc::c_int > max_level as libc::c_int {
		max_level = cur_level;
	}
	must_exist1 = get_rand(0 as libc::c_int, 5 as libc::c_int) as c_short;
	match must_exist1 as libc::c_int {
		0 => {
			must_exist1 = 0 as libc::c_int as c_short;
			must_exist2 = 1 as libc::c_int as c_short;
			must_exist3 = 2 as libc::c_int as c_short;
		}
		1 => {
			must_exist1 = 3 as libc::c_int as c_short;
			must_exist2 = 4 as libc::c_int as c_short;
			must_exist3 = 5 as libc::c_int as c_short;
		}
		2 => {
			must_exist1 = 6 as libc::c_int as c_short;
			must_exist2 = 7 as libc::c_int as c_short;
			must_exist3 = 8 as libc::c_int as c_short;
		}
		3 => {
			must_exist1 = 0 as libc::c_int as c_short;
			must_exist2 = 3 as libc::c_int as c_short;
			must_exist3 = 6 as libc::c_int as c_short;
		}
		4 => {
			must_exist1 = 1 as libc::c_int as c_short;
			must_exist2 = 4 as libc::c_int as c_short;
			must_exist3 = 7 as libc::c_int as c_short;
		}
		5 => {
			must_exist1 = 2 as libc::c_int as c_short;
			must_exist2 = 5 as libc::c_int as c_short;
			must_exist3 = 8 as libc::c_int as c_short;
		}
		_ => {}
	}
	big_room = (cur_level as libc::c_int == party_counter as libc::c_int
		&& rand_percent(1 as libc::c_int)) as libc::c_int as libc::c_char;
	if big_room != 0 {
		make_room(
			10 as libc::c_int,
			0 as libc::c_int,
			0 as libc::c_int,
			0 as libc::c_int,
		);
	} else {
		i = 0 as libc::c_int as c_short;
		while (i as libc::c_int) < 9 as libc::c_int {
			make_room(
				i as libc::c_int,
				must_exist1 as libc::c_int,
				must_exist2 as libc::c_int,
				must_exist3 as libc::c_int,
			);
			i += 1;
		}
	}
	if big_room == 0 {
		add_mazes();
		mix_random_rooms();
		j = 0 as libc::c_int as c_short;
		while (j as libc::c_int) < 9 as libc::c_int {
			i = random_rooms[j as usize] as c_short;
			if (i as libc::c_int) < 9 as libc::c_int - 1 as libc::c_int {
				connect_rooms(i as libc::c_int, i as libc::c_int + 1 as libc::c_int);
			}
			if (i as libc::c_int) < 9 as libc::c_int - 3 as libc::c_int {
				connect_rooms(i as libc::c_int, i as libc::c_int + 3 as libc::c_int);
			}
			if (i as libc::c_int) < 9 as libc::c_int - 2 as libc::c_int {
				if (*rooms
					.as_mut_ptr()
					.offset((i as libc::c_int + 1 as libc::c_int) as isize))
					.room_type as libc::c_int
					& 0o1 as libc::c_int as c_ushort as libc::c_int != 0
				{
					if connect_rooms(i as libc::c_int, i as libc::c_int + 2 as libc::c_int) {
						(*rooms
							.as_mut_ptr()
							.offset((i as libc::c_int + 1 as libc::c_int) as isize))
							.room_type = RoomType::Room;
					}
				}
			}
			if (i as libc::c_int) < 9 as libc::c_int - 6 as libc::c_int {
				if (*rooms
					.as_mut_ptr()
					.offset((i as libc::c_int + 3 as libc::c_int) as isize))
					.room_type as libc::c_int
					& 0o1 as libc::c_int as c_ushort as libc::c_int != 0
				{
					if connect_rooms(i as libc::c_int, i as libc::c_int + 6 as libc::c_int) {
						(*rooms
							.as_mut_ptr()
							.offset((i as libc::c_int + 3 as libc::c_int) as isize))
							.room_type = RoomType::Room;
					}
				}
			}
			if is_all_connected() != 0 {
				break;
			}
			j += 1;
		}
		fill_out_level();
	}
	if has_amulet() == 0 && cur_level as libc::c_int >= 26 as libc::c_int {
		put_amulet();
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn make_room(rn: libc::c_int, r1: libc::c_int, r2: libc::c_int, r3: libc::c_int) {
	let rn: usize = rn as usize;
	let (left, right, top, bottom, do_shrink, room_index) =
		match rn {
			0 => (0 as c_int, COL1 - 1 as c_int, MIN_ROW, ROW1 - 1 as c_int, true, rn),
			1 => (COL1 + 1, COL2 - 1, MIN_ROW, ROW1 - 1, true, rn),
			2 => (COL2 + 1, DCOLS - 1, MIN_ROW, ROW1 - 1, true, rn),
			3 => (0, COL1 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			4 => (COL1 + 1, COL2 - 1, ROW1 + 1, ROW2 - 1, true, rn),
			5 => (COL2 + 1, DCOLS - 1, ROW1 + 1, ROW2 - 1, true, rn),
			6 => (0, COL1 - 1, ROW2 + 1, DROWS - 2, true, rn),
			7 => (COL1 + 1, COL2 - 1, ROW2 + 1, DROWS - 2, true, rn),
			8 => (COL2 + 1, DCOLS - 1, ROW2 + 1, DROWS - 2, true, rn),
			BIG_ROOM => {
				let top = get_rand(MIN_ROW, MIN_ROW + 5);
				let bottom = get_rand(DROWS - 7, DROWS - 2);
				let left = get_rand(0, 10);
				let right = get_rand(DCOLS - 11, DCOLS - 1);
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
		let bottom = top + height - 1 as c_int;
		let left = left + col_offset;
		let right = left + width - 1 as c_int;
		let skip_walls = (room_index != r1 as usize) && (room_index != r2 as usize) && (room_index != r3 as usize) && rand_percent(40 as c_int);
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
					SpotFlag::HorWall as c_ushort
				} else if !top_border && !bottom_border && (left_border || right_border) {
					SpotFlag::VertWall as c_ushort
				} else {
					SpotFlag::Floor as c_ushort
				};
				dungeon[i as usize][j as usize] = ch;
			}
		}
	}
	rooms[rn].top_row = top;
	rooms[rn].bottom_row = bottom;
	rooms[rn].left_col = left;
	rooms[rn].right_col = right;
}

pub unsafe fn connect_rooms(room1: libc::c_int, room2: libc::c_int) -> bool {
	let (room1, room2) = (room1 as usize, room2 as usize);
	if rooms[room1].room_type.is_nothing() || rooms[room2].room_type.is_nothing() {
		return false;
	}
	let dir1 = if same_row(room1, room2) && (rooms[room1].left_col > rooms[room2].right_col) {
		DoorDirection::Left
	} else if same_row(room1, room2) && (rooms[room2].left_col > rooms[room1].right_col) {
		DoorDirection::Right
	} else if same_col(room1, room2) && (rooms[room1].top_row > rooms[room2].bottom_row) {
		DoorDirection::Up
	} else if same_col(room1, room2) && (rooms[room2].top_row > rooms[room1].bottom_row) {
		DoorDirection::Down
	} else {
		return false;
	};
	let dir2 = dir1.to_inverse();
	let spot1 = put_door(&mut rooms[room1], dir1);
	let spot2 = put_door(&mut rooms[room2], dir2);
	let mut do_draw = true;
	while do_draw {
		draw_simple_passage(&spot1, &spot2, dir1);
		do_draw = rand_percent(4);
	}

	let door1_index = dir1.to_index();
	let door2_index = dir2.to_index();
	rooms[room1].doors[door1_index].oth_room = Some(room2);
	rooms[room1].doors[door1_index].oth_row = Some(spot2.row);
	rooms[room1].doors[door1_index].oth_col = Some(spot2.col);
	rooms[room2].doors[door2_index].oth_room = Some(room1);
	rooms[room2].doors[door2_index].oth_row = Some(spot1.row);
	rooms[room2].doors[door2_index].oth_col = Some(spot1.col);
	return true;
}

#[no_mangle]
pub unsafe extern "C" fn clear_level() -> libc::c_int {
	let mut i: c_short = 0;
	let mut j: c_short = 0;
	i = 0 as libc::c_int as c_short;
	while (i as libc::c_int) < 9 as libc::c_int {
		(*rooms.as_mut_ptr().offset(i as isize)).room_type = RoomType::Nothing;
		j = 0 as libc::c_int as c_short;
		while (j as libc::c_int) < 4 as libc::c_int {
			(*rooms.as_mut_ptr().offset(i as isize)).doors[j as usize].oth_room = None;
			j += 1;
		}
		i += 1;
	}
	i = 0 as libc::c_int as c_short;
	while (i as libc::c_int) < 10 as libc::c_int {
		(*traps.as_mut_ptr().offset(i as isize))
			.trap_type = -(1 as libc::c_int) as c_short;
		i += 1;
	}
	i = 0 as libc::c_int as c_short;
	while (i as libc::c_int) < 24 as libc::c_int {
		j = 0 as libc::c_int as c_short;
		while (j as libc::c_int) < 80 as libc::c_int {
			dungeon[i as usize][j as usize] = 0 as libc::c_int as c_ushort;
			j += 1;
		}
		i += 1;
	}
	see_invisible = 0 as libc::c_int as libc::c_char;
	detect_monster = see_invisible;
	bear_trap = 0 as libc::c_int as c_short;
	being_held = bear_trap as libc::c_char;
	party_room = -(1 as libc::c_int) as c_short;
	rogue.col = -(1 as libc::c_int) as c_short;
	rogue.row = rogue.col;
	wclear(stdscr);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn put_door(room: &mut room, door_dir: DoorDirection) -> DungeonSpot {
	let wall_width = if RoomType::Maze == room.room_type { 0 as c_int } else { 1 };
	let door_spot = match door_dir {
		DoorDirection::Up | DoorDirection::Down => {
			let row = if door_dir == DoorDirection::Up { room.top_row } else { room.bottom_row } as usize;
			let mut col;
			loop {
				col = get_rand(room.left_col + wall_width, room.right_col - wall_width) as usize;
				if SpotFlag::is_horwall_or_tunnel(dungeon[row][col]) {
					break;
				}
			}
			DungeonSpot { row, col }
		}
		DoorDirection::Left | DoorDirection::Right => {
			let col = if door_dir == DoorDirection::Left { room.left_col } else { room.right_col } as usize;
			let mut row;
			loop {
				row = get_rand(room.top_row + wall_width, room.bottom_row - wall_width) as usize;
				if SpotFlag::is_vertwall_or_tunnel(dungeon[row][col]) {
					break;
				}
			}
			DungeonSpot { row, col }
		}
	};
	if room.room_type == RoomType::Room {
		dungeon[door_spot.row][door_spot.col] = Door as c_ushort;
	}
	if (cur_level > 2) && rand_percent(HIDE_PERCENT) {
		dungeon[door_spot.row][door_spot.col] |= HIDDEN;
	}
	let door_index = door_dir.to_index();
	room.doors[door_index].door_row = door_spot.row as c_short;
	room.doors[door_index].door_col = door_spot.col as c_short;
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
				let middle = get_rand((col1 + 1) as c_int, (col2 - 1) as c_int);
				for i in (col1 as c_int + 1)..middle {
					dungeon[row1][i as usize] = TUNNEL;
				}
				let mut i = row1;
				let step = if row1 > row2 { -1 } else { 1 };
				while i != row2 {
					dungeon[i][middle as usize] = TUNNEL;
					i = ((i as isize) + step) as usize;
				}
				for i in (middle as usize)..col2 {
					dungeon[row2][i] = TUNNEL;
				}
				(col1, row1, col2, row2)
			}
			DoorDirection::Up | DoorDirection::Down => {
				let (col1, row1, col2, row2) = if spot1.row > spot2.row {
					(spot2.col, spot2.row, spot1.col, spot1.row)
				} else {
					(spot1.col, spot1.row, spot2.col, spot2.row)
				};
				let middle = get_rand((row1 + 1) as c_int, (row2 - 1) as c_int) as usize;
				for i in (row1 + 1)..middle {
					dungeon[i][col1] = TUNNEL;
				}
				let mut i = col1;
				let step = if col1 > col2 { -1 } else { 1 };
				while i != col2 {
					dungeon[middle][i] = TUNNEL;
					i = ((i as isize) + step) as usize;
				}
				for i in middle..row2 {
					dungeon[i][col2] = TUNNEL;
				}
				(col1, row1, col2, row2)
			}
		};
	if rand_percent(HIDE_PERCENT) {
		hide_boxed_passage(row1 as c_int, col1 as c_int, row2 as c_int, col2 as c_int, 1);
	}
}


pub fn same_row(room1: usize, room2: usize) -> bool {
	room1 / 3 == room2 / 3
}

pub fn same_col(room1: usize, room2: usize) -> bool {
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
		};

		for i in 0..MAXROOMS {
			let j = ((start + i) % MAXROOMS) as usize;
			if rooms[j].room_type.is_nothing() {
				let do_maze = rand_percent(maze_percent as c_int);
				if do_maze {
					rooms[j].room_type = RoomType::Maze;
					make_maze(
						get_rand(rooms[j].top_row + 1, rooms[j].bottom_row - 1) as usize,
						get_rand(rooms[j].left_col + 1, rooms[j].right_col - 1) as usize,
						rooms[j].top_row as usize, rooms[j].bottom_row as usize,
						rooms[j].left_col as usize, rooms[j].right_col as usize,
					);
					hide_boxed_passage(rooms[j].top_row, rooms[j].left_col,
					                   rooms[j].bottom_row, rooms[j].right_col,
					                   get_rand(0, 2));
				}
			}
		}
	}
}

pub unsafe fn fill_out_level() {
	mix_random_rooms();
	r_de = NO_ROOM as c_short;

	for i in 0..MAXROOMS {
		let rn = random_rooms[i as usize] as usize;
		// Note: original C uses (rooms[rn].is_room & R_NOTHING) for the
		// first clause of the conditional. Since R_NOTHING is 0, the first clause would always evaluate
		// to false.
		// Question is, was that a bad bug that should be fixed? Or was it a good bug that will break this
		// function if fixed.  For now, we will fix the bug.
		if (rooms[rn].room_type == RoomType::Nothing) || ((rooms[rn].room_type == RoomType::Cross) && coin_toss()) {
			fill_it(rn, true);
		}
	}
	if r_de != NO_ROOM as c_short {
		fill_it(r_de as usize, false);
	}
}


pub unsafe fn fill_it(rn: usize, do_rec_de: bool) {
	let mut did_this = false;
	static mut OFFSETS: [isize; 4] = [-1, 1, 3, -3];

	let mut srow: usize = 0;
	let mut scol: usize = 0;
	for _ in 0..10 {
		srow = get_rand(0 as c_int, 3 as c_int) as usize;
		scol = get_rand(0 as c_int, 3 as c_int) as usize;
		let t = OFFSETS[srow];
		OFFSETS[srow] = OFFSETS[scol];
		OFFSETS[scol] = t;
	}
	for i in 0..4 {
		let target_room = (rn as isize + OFFSETS[i]) as usize;
		let mut rooms_found: usize = 0;

		if ((target_room < 0) || (target_room >= MAXROOMS as usize))
			|| (!same_row(rn, target_room) && !same_col(rn, target_room))
			|| (rooms[target_room].room_type != RoomType::Room && rooms[target_room].room_type != RoomType::Maze) {
			continue;
		}
		let tunnel_dir = get_tunnel_dir(rn, target_room);
		let door_dir = tunnel_dir.to_inverse();
		if rooms[target_room].doors[door_dir.to_index()].oth_room.is_some() {
			continue;
		}
		let (mut srow, mut scol) = (srow as c_short, scol as c_short);
		if ((!do_rec_de) || did_this) || (!mask_room(rn as c_short, &mut srow, &mut scol, SpotFlag::Tunnel as c_ushort)) {
			srow = ((rooms[rn].top_row + rooms[rn].bottom_row) / 2) as c_short;
			scol = ((rooms[rn].left_col + rooms[rn].right_col) / 2) as c_short;
		}
		let d_spot = put_door(&mut rooms[target_room], door_dir);
		rooms_found += 1;
		let s_spot = DungeonSpot { col: scol as usize, row: srow as usize };
		draw_simple_passage(&s_spot, &d_spot, tunnel_dir);
		rooms[rn].room_type = RoomType::DeadEnd;
		dungeon[srow as usize][scol as usize] = SpotFlag::Tunnel as c_ushort;

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

pub unsafe fn recursive_deadend(rn: usize, offsets: &[isize; 4], s_spot: &DungeonSpot)
{
	rooms[rn].room_type = RoomType::DeadEnd;
	dungeon[s_spot.row][s_spot.col] = SpotFlag::Tunnel as c_ushort;

	for i in 0..4 {
		let de = (rn as isize + offsets[i]) as usize;
		if ((de < 0) || (de >= MAXROOMS as usize))
			|| (!same_row(rn, de) && !same_col(rn, de)) {
			continue;
		}
		if rooms[de].room_type != RoomType::Nothing {
			continue;
		}
		let d_spot = DungeonSpot {
			col: ((rooms[de].left_col + rooms[de].right_col) / 2) as usize,
			row: ((rooms[de].top_row + rooms[de].bottom_row) / 2) as usize,
		};
		let tunnel_dir = get_tunnel_dir(rn, de);
		draw_simple_passage(&s_spot, &d_spot, tunnel_dir);
		r_de = de as c_short;
		recursive_deadend(de, offsets, &d_spot);
	}
}

unsafe fn get_tunnel_dir(rn: usize, de: usize) -> DoorDirection {
	if same_row(rn, de) {
		if rooms[rn].left_col < rooms[de].left_col { DoorDirection::Right } else { DoorDirection::Left }
	} else {
		if rooms[rn].top_row < rooms[de].top_row { DoorDirection::Down } else { DoorDirection::Up }
	}
}


#[no_mangle]
pub unsafe extern "C" fn mask_room(
	mut rn: c_short,
	mut row: *mut c_short,
	mut col: *mut c_short,
	mut mask: c_ushort,
) -> bool {
	let mut i: c_short = 0;
	let mut j: c_short = 0;
	i = (*rooms.as_mut_ptr().offset(rn as isize)).top_row as c_short;
	while i as libc::c_int
		<= (*rooms.as_mut_ptr().offset(rn as isize)).bottom_row as libc::c_int
	{
		j = (*rooms.as_mut_ptr().offset(rn as isize)).left_col as c_short;
		while j as libc::c_int
			<= (*rooms.as_mut_ptr().offset(rn as isize)).right_col as libc::c_int
		{
			if dungeon[i as usize][j as usize] as libc::c_int & mask as libc::c_int != 0
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

const TUNNEL: c_ushort = SpotFlag::Tunnel as c_ushort;
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

pub unsafe fn hide_boxed_passage(row1: c_int, col1: c_int, row2: c_int, col2: c_int, n: c_int) {
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
pub unsafe extern "C" fn put_player(mut nr: c_short) -> libc::c_int {
	let mut rn: c_short = nr;
	let mut misses: c_short = 0;
	let mut row: c_short = 0;
	let mut col: c_short = 0;
	misses = 0 as libc::c_int as c_short;
	while (misses as libc::c_int) < 2 as libc::c_int
		&& rn as libc::c_int == nr as libc::c_int
	{
		gr_row_col(
			&mut row,
			&mut col,
			(0o100 as libc::c_int as c_ushort as libc::c_int
				| 0o200 as libc::c_int as c_ushort as libc::c_int
				| 0o1 as libc::c_int as c_ushort as libc::c_int
				| 0o4 as libc::c_int as c_ushort as libc::c_int) as c_ushort,
		);
		rn = get_room_number(row as libc::c_int, col as libc::c_int) as c_short;
		misses += 1;
	}
	rogue.row = row;
	rogue.col = col;
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o200 as libc::c_int as c_ushort as libc::c_int != 0
	{
		cur_room = -(3 as libc::c_int) as c_short;
	} else {
		cur_room = rn;
	}
	if cur_room as libc::c_int != -(3 as libc::c_int) {
		light_up_room(cur_room as libc::c_int);
	} else {
		light_passage(rogue.row as libc::c_int, rogue.col as libc::c_int);
	}
	wake_room(
		get_room_number(rogue.row as libc::c_int, rogue.col as libc::c_int) as c_short,
		1 as libc::c_char,
		rogue.row as c_short,
		rogue.col as c_short,
	);
	if !new_level_message.is_null() {
		message(new_level_message, 0 as libc::c_int);
		new_level_message = 0 as *mut libc::c_char;
	}
	if wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int)
		== -(1 as libc::c_int)
	{
		-(1 as libc::c_int);
	} else {
		waddch(stdscr, rogue.fchar as chtype);
	};
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn drop_check() -> libc::c_int {
	if wizard != 0 {
		return 1 as libc::c_int;
	}
	if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
		& 0o4 as libc::c_int as c_ushort as libc::c_int != 0
	{
		if levitate != 0 {
			message(
				b"you're floating in the air!\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			return 0 as libc::c_int;
		}
		return 1 as libc::c_int;
	}
	message(
		b"I see no way down\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
	);
	return 0 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn check_up() -> libc::c_int {
	if wizard == 0 {
		if dungeon[rogue.row as usize][rogue.col as usize] as libc::c_int
			& 0o4 as libc::c_int as c_ushort as libc::c_int == 0
		{
			message(
				b"I see no way up\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			return 0 as libc::c_int;
		}
		if has_amulet() == 0 {
			message(
				b"Your way is magically blocked\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			return 0 as libc::c_int;
		}
	}
	new_level_message = b"you feel a wrenching sensation in your gut\0" as *const u8
		as *const libc::c_char as *mut libc::c_char;
	if cur_level as libc::c_int == 1 as libc::c_int {
		win();
	} else {
		cur_level = (cur_level as libc::c_int - 2 as libc::c_int) as c_short;
		return 1 as libc::c_int;
	}
	return 0 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn add_exp(
	mut e: libc::c_int,
	mut promotion: libc::c_char,
) {
	let mut mbuf: [libc::c_char; 40] = [0; 40];
	let mut new_exp: c_short = 0;
	let mut i: c_short = 0;
	let mut hp: c_short = 0;
	rogue.exp_points += e as c_long;
	if rogue.exp_points
		>= level_points[(rogue.exp as libc::c_int - 1 as libc::c_int) as usize]
	{
		new_exp = get_exp_level(rogue.exp_points) as c_short;
		if rogue.exp_points > 10000000 as c_long {
			rogue
				.exp_points = 10000000 as c_long
				+ 1 as libc::c_int as c_long;
		}
		i = (rogue.exp as libc::c_int + 1 as libc::c_int) as c_short;
		while i as libc::c_int <= new_exp as libc::c_int {
			sprintf(
				mbuf.as_mut_ptr(),
				b"welcome to level %d\0" as *const u8 as *const libc::c_char,
				i as libc::c_int,
			);
			message(mbuf.as_mut_ptr(), 0 as libc::c_int);
			if promotion != 0 {
				hp = hp_raise() as c_short;
				rogue
					.hp_current = (rogue.hp_current as libc::c_int + hp as libc::c_int)
					as c_short;
				rogue
					.hp_max = (rogue.hp_max as libc::c_int + hp as libc::c_int)
					as c_short;
			}
			rogue.exp = i;
			print_stats(0o4 as libc::c_int | 0o40 as libc::c_int);
			i += 1;
		}
	} else {
		print_stats(0o40 as libc::c_int);
	}
}

pub unsafe fn get_exp_level(e: c_long) -> usize {
	const MAX_EXP_LEVEL: usize = 21;
	for i in 0..(MAX_EXP_LEVEL - 1) {
		if level_points[i] > e {
			return i + 1;
		}
	}
	return MAX_EXP_LEVEL;
}

pub unsafe fn hp_raise() -> usize {
	if wizard != 0 {
		10
	} else {
		get_rand(3, 10) as usize
	}
}

#[no_mangle]
pub unsafe extern "C" fn show_average_hp() -> libc::c_int {
	let mut mbuf: [libc::c_char; 80] = [0; 80];
	let mut real_average: libc::c_int = 0;
	let mut effective_average: libc::c_int = 0;
	if rogue.exp as libc::c_int == 1 as libc::c_int {
		effective_average = 0.00f64 as libc::c_int;
		real_average = effective_average;
	} else {
		real_average = (rogue.hp_max as libc::c_int - extra_hp as libc::c_int
			- 12 as libc::c_int + less_hp as libc::c_int)
			/ (rogue.exp as libc::c_int - 1 as libc::c_int);
		effective_average = (rogue.hp_max as libc::c_int - 12 as libc::c_int)
			/ (rogue.exp as libc::c_int - 1 as libc::c_int);
	}
	sprintf(
		mbuf.as_mut_ptr(),
		b"R-Hp: %.2f, E-Hp: %.2f (!: %d, V: %d)\0" as *const u8 as *const libc::c_char,
		real_average,
		effective_average,
		extra_hp as libc::c_int,
		less_hp as libc::c_int,
	);
	message(mbuf.as_mut_ptr(), 0 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn mix_random_rooms() {
	for _i in 0..(3 * MAXROOMS) {
		let mut x: usize = 0;
		let mut y: usize = 0;
		loop {
			x = get_rand(0, MAXROOMS - 1) as usize;
			y = get_rand(0, MAXROOMS - 1) as usize;
			if x != y {
				break;
			}
		}
		mem::swap(&mut random_rooms[x], &mut random_rooms[y]);
	}
}
