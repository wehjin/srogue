#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{addch, chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::prelude::DoorDirection::{Left, Right};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Object, Stairs, Trap, Tunnel, VertWall};
use crate::room::DoorDirection::{Up, Down};
use crate::room::RoomType::{Maze, Room};

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct dr {
	pub oth_room: Option<i64>,
	pub oth_row: Option<i64>,
	pub oth_col: Option<i64>,
	pub door_row: i64,
	pub door_col: i64,
}

pub type door = dr;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
	Nothing,
	Room,
	Maze,
	DeadEnd,
	Cross,
}

impl RoomType {
	pub fn is_nothing(&self) -> bool {
		*self == RoomType::Nothing
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DoorDirection {
	Up,
	Down,
	Left,
	Right,
}

impl DoorDirection {
	pub fn to_index(&self) -> usize {
		match self {
			Up => 0,
			Right => 1,
			Down => 2,
			Left => 3,
		}
	}
	pub fn to_inverse(&self) -> DoorDirection {
		match self {
			Up => Down,
			Right => Left,
			Down => Up,
			Left => Right,
		}
	}
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct rm {
	pub bottom_row: i64,
	pub right_col: i64,
	pub left_col: i64,
	pub top_row: i64,
	pub doors: [door; 4],
	pub room_type: RoomType,
}

pub type room = rm;

pub static mut rooms: [room; MAXROOMS as usize] = [rm {
	bottom_row: 0,
	right_col: 0,
	left_col: 0,
	top_row: 0,
	doors: [dr::default(); 4],
	room_type: RoomType::Nothing,
}; MAXROOMS as usize];
#[no_mangle]
pub static mut rooms_visited: [libc::c_char; 9] = [0; 9];


pub unsafe fn light_up_room(rn: i64) {
	if blind == 0 {
		for i in rooms[rn as usize].top_row..=rooms[rn as usize].bottom_row {
			for j in rooms[rn as usize].left_col..=rooms[rn as usize].right_col {
				if Monster.is_set(dungeon[i as usize][j as usize]) {
					let monster = object_at(&level_monsters, i, j);
					if !monster.is_null() {
						Monster.clear(&mut dungeon[(*monster).row as usize][(*monster).col as usize]);
						(*monster).set_trail_char(get_dungeon_char((*monster).row, (*monster).col));
						Monster.set(&mut dungeon[(*monster).row as usize][(*monster).col as usize]);
					}
				}
				mvaddch(i as i32, j as i32, get_dungeon_char(i, j));
			}
		}
		mvaddch(rogue.row as i32, rogue.col as i32, rogue.fchar as chtype);
	}
}


pub unsafe fn light_passage(row: i64, col: i64) {
	if blind != 0 {
		return;
	}
	let i_end = if row < DROWS as i64 - 2 { 1 } else { 0 };
	let j_end = if col < DCOLS as i64 - 1 { 1 } else { 0 };
	let i_start = if row > MIN_ROW { -1 } else { 0 };
	let j_start = if col > 0 { -1 } else { 0 };
	for i in i_start..=i_end {
		for j in j_start..=j_end {
			if can_move(row, col, row + i, col + j) {
				mvaddch((row + i) as i32, (col + j) as i32, get_dungeon_char(row + i, col + j));
			}
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn darken_room(rn: i64) -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	i = (rooms[rn as usize].top_row + 1) as libc::c_short;
	while (i as i64) < rooms[rn as usize].bottom_row {
		j = (rooms[rn as usize].left_col + 1)
			as libc::c_short;
		while (j as i64) < rooms[rn as usize].right_col {
			if blind != 0 {
				if ncurses::wmove(ncurses::stdscr(), i as i32, j as i32)
					== -1
				{
					-1;
				} else {
					addch(' ' as i32 as chtype);
				};
			} else if dungeon[i as usize][j as usize] as i64
				& (0o1 as libc::c_ushort as i64
				| 0o4 as libc::c_ushort as i64) == 0
				&& !(detect_monster as i64 != 0
				&& dungeon[i as usize][j as usize] as i64
				& 0o2 as libc::c_ushort as i64 != 0)
			{
				if !imitating(i as i64, j as i64) {
					if ncurses::wmove(ncurses::stdscr(), i as i32, j as i32)
						== -1
					{
						-1;
					} else {
						addch(' ' as i32 as chtype);
					};
				}
				if dungeon[i as usize][j as usize] as i64
					& 0o400 as libc::c_ushort as i64 != 0
					&& dungeon[i as usize][j as usize] as i64
					& 0o1000 as libc::c_ushort as i64 == 0
				{
					if ncurses::wmove(ncurses::stdscr(), i as i32, j as i32)
						== -1
					{
						-1;
					} else {
						addch('^' as i32 as chtype);
					};
				}
			}
			j += 1;
		}
		i += 1;
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn get_dungeon_char(row: i64, col: i64) -> chtype {
	let mask = dungeon[row as usize][col as usize];
	if Monster.is_set(mask) {
		return gmc_row_col(row, col);
	}
	if Object.is_set(mask) {
		let obj = object_at(&level_objects, row, col);
		return get_mask_char((*obj).what_is) as chtype;
	}
	if SpotFlag::is_any_set(&vec![Tunnel, Stairs, HorWall, VertWall, Floor, Door], mask) {
		if SpotFlag::is_any_set(&vec![Tunnel, Stairs], mask) && !Hidden.is_set(mask) {
			return if Stairs.is_set(mask) {
				chtype::from('%')
			} else {
				chtype::from('#')
			};
		}
		if HorWall.is_set(mask) {
			return chtype::from('-');
		}
		if VertWall.is_set(mask) {
			return chtype::from('|');
		}
		if Floor.is_set(mask) {
			if Trap.is_set(mask) {
				if !Hidden.is_set(mask) {
					return chtype::from('^');
				}
			}
			return chtype::from('.');
		}
		if Door.is_set(mask) {
			return if Hidden.is_set(mask) {
				if (col > 0 && HorWall.is_set(dungeon[row as usize][col as usize - 1]))
					|| (col < (DCOLS as i64 - 1) && HorWall.is_set(dungeon[row as usize][col as usize + 1])) {
					chtype::from('-')
				} else {
					chtype::from('|')
				}
			} else {
				chtype::from('+')
			};
		}
	}
	return chtype::from(' ');
}

pub fn get_mask_char(mask: ObjectWhat) -> char {
	match mask {
		ObjectWhat::Scroll => '?',
		ObjectWhat::Potion => '!',
		ObjectWhat::Gold => '*',
		ObjectWhat::Food => ':',
		ObjectWhat::Wand => '/',
		ObjectWhat::Armor => ']',
		ObjectWhat::Weapon => ')',
		ObjectWhat::Ring => '=',
		ObjectWhat::Amulet => ',',
		_ => '~',
	}
}


pub unsafe fn gr_row_col(row: &mut i64, col: &mut i64, spots: Vec<SpotFlag>) {
	let mut r = 0;
	let mut c = 0;
	loop {
		r = get_rand(MIN_ROW, DROWS as i64 - 2);
		c = get_rand(0, DCOLS as i64 - 1);
		let rn = get_room_number(r, c);
		let keep_looking = rn == NO_ROOM
			|| !SpotFlag::is_any_set(&spots, dungeon[r as usize][c as usize])
			|| SpotFlag::are_others_set(&spots, dungeon[r as usize][c as usize])
			|| !(rooms[rn as usize].room_type == Room || rooms[rn as usize].room_type == Maze)
			|| ((r == rogue.row) && (c == rogue.col));
		if !keep_looking {
			break;
		}
	}
	*row = r;
	*col = c;
}

pub unsafe fn gr_room() -> i64 {
	loop {
		let i = get_rand(0, MAXROOMS as usize - 1);
		if rooms[i].room_type == Room || rooms[i].room_type == Maze {
			return i as i64;
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn party_objects(mut rn: i64) -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut nf: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut n: libc::c_short = 0;
	let mut N: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut found: libc::c_char = 0;
	N = ((rooms[rn as usize].bottom_row
		- rooms[rn as usize].top_row - 1)
		* (rooms[rn as usize].right_col
		- rooms[rn as usize].left_col - 1))
		as libc::c_short;
	n = get_rand(5, 10);
	if n as i64 > N as i64 {
		n = N - 2;
	}
	i = 0;
	while (i as i64) < n as i64 {
		found = 0;
		j = found as libc::c_short;
		while found == 0 && (j as i64) < 250 {
			row = get_rand(
				rooms[rn as usize].top_row + 1,
				rooms[rn as usize].bottom_row - 1,
			) as libc::c_short;
			col = get_rand(
				rooms[rn as usize].left_col + 1,
				rooms[rn as usize].right_col - 1,
			) as libc::c_short;
			if dungeon[row as usize][col as usize] as i64
				== 0o100 as libc::c_ushort as i64
				|| dungeon[row as usize][col as usize] as i64
				== 0o200 as libc::c_ushort as i64
			{
				found = 1 as libc::c_char;
			}
			j += 1;
		}
		if found != 0 {
			obj = gr_object();
			place_at(obj, row as i64, col as i64);
			nf += 1;
		}
		i += 1;
	}
	return nf as i64;
}

pub fn get_room_number(row: i64, col: i64) -> i64 {
	unsafe {
		for i in 0..MAXROOMS as usize {
			let below_top_wall = row >= rooms[i].top_row;
			let above_bottom_wall = row <= rooms[i].bottom_row;
			let right_of_left_wall = col >= rooms[i].left_col;
			let left_of_right_wall = col <= rooms[i].right_col;
			if below_top_wall && above_bottom_wall && right_of_left_wall && left_of_right_wall {
				return i as i64;
			}
		}
	}
	return NO_ROOM;
}

pub fn get_opt_room_number(row: i64, col: i64) -> Option<i64> {
	let rn = get_room_number(row, col);
	if rn == NO_ROOM {
		None
	} else {
		Some(rn)
	}
}

pub unsafe fn is_all_connected() -> bool {
	let mut starting_room = NO_ROOM;
	for i in 0..MAXROOMS as usize {
		rooms_visited[i] = 0;
		if rooms[i].room_type == Room || rooms[i].room_type == Maze {
			starting_room = i as i64;
		}
	}
	visit_rooms(starting_room);
	for i in 0..MAXROOMS as usize {
		if (rooms[i].room_type == Room || rooms[i].room_type == Maze) && rooms_visited[i] == 0 {
			return false;
		}
	}
	return true;
}

pub unsafe fn visit_rooms(rn: i64) {
	let rn = rn as usize;
	rooms_visited[rn] = 1;
	for i in 0..4usize {
		if let Some(oth_rn) = rooms[rn].doors[i].oth_room {
			if oth_rn >= 0 && rooms_visited[oth_rn as usize] == 0 {
				visit_rooms(oth_rn);
			}
		}
	}
}

pub unsafe fn draw_magic_map() {
	let mask = vec![HorWall, VertWall, Door, Tunnel, Trap, Stairs, Monster];
	for i in 0..DROWS {
		for j in 0..DCOLS {
			let s = dungeon[i][j];
			if SpotFlag::is_any_set(&mask, s) {
				let mut ch = mvinch(i as i32, j as i32) as u8 as char;
				if ch == ' ' || ch.is_ascii_uppercase() || SpotFlag::is_any_set(&vec![Trap, Hidden], s) {
					let och = ch;
					Hidden.clear(&mut dungeon[i][j]);
					if HorWall.is_set(s) {
						ch = '-';
					} else if VertWall.is_set(s) {
						ch = '|';
					} else if Door.is_set(s) {
						ch = '+';
					} else if Trap.is_set(s) {
						ch = '^';
					} else if Stairs.is_set(s) {
						ch = '%';
					} else if Tunnel.is_set(s) {
						ch = '#';
					} else {
						continue;
					}
					if !Monster.is_set(s) || och == ' ' {
						addch(chtype::from(ch));
					}
					if Monster.is_set(s) {
						let monster = object_at(&mut level_monsters, i as i64, j as i64);
						if !monster.is_null() {
							(*monster).set_trail_char(chtype::from(ch));
						}
					}
				}
			}
		}
	}
}

pub unsafe fn dr_course(mut monster: *mut object, entering: bool, row: i64, col: i64) {
	(*monster).row = row;
	(*monster).col = col;
	if mon_sees(monster, rogue.row, rogue.col) {
		(*monster).trow = -1;
		return;
	}
	let rn = get_opt_room_number(row, col);
	if entering {
		/* look for door to some other room */
		let r = get_rand(0, MAXROOMS as usize - 1);
		for i in 0..MAXROOMS as usize {
			let rr = (r + i) % MAXROOMS as usize;
			if !(rooms[rr].room_type == Room || rooms[rr].room_type == Maze) || Some(rr as i64) == rn {
				continue;
			}
			for k in 0..4 {
				if rooms[rr].doors[k].oth_room == rn {
					(*monster).trow = rooms[rr].doors[k].oth_row.expect("oth row");
					(*monster).tcol = rooms[rr].doors[k].oth_col.expect("oth col");
					if (*monster).trow == row && (*monster).tcol == col {
						continue;
					}
					return;
				}
			}
		}
		/* look for door to dead end */
		let rn = rn.expect("rn") as usize;
		for i in rooms[rn].top_row..=rooms[rn].bottom_row {
			for j in rooms[rn].left_col..=rooms[rn].right_col {
				if i != (*monster).row && j != (*monster).col && Door.is_set(dungeon[i as usize][j as usize]) {
					(*monster).trow = i;
					(*monster).tcol = j;
					return;
				}
			}
		}
		/* return monster to room that he came from */
		for i in 0..MAXROOMS as usize {
			for j in 0..4usize {
				if rooms[i].doors[j].oth_room == Some(rn as i64) {
					for k in 0..4usize {
						if rooms[rn].doors[k].oth_room == Some(i as i64) {
							(*monster).trow = rooms[rn].doors[k].oth_row.expect("oth row");
							(*monster).tcol = rooms[rn].doors[k].oth_col.expect("oth col");
							return;
						}
					}
				}
			}
		}
		(*monster).trow = -1;
	} else {
		//* exiting room */
		match rn {
			None => {
				(*monster).trow = -1;
			}
			Some(rn) => {
				if let Some((row, col)) = get_oth_room(rn, row, col) {
					(*monster).trow = row;
					(*monster).tcol = col;
				} else {
					(*monster).trow = -1;
				}
			}
		}
	}
}

pub unsafe fn get_oth_room(rn: i64, row: i64, col: i64) -> Option<(i64, i64)> {
	let rn = rn as usize;
	let d = if row == rooms[rn].top_row {
		Some(Up)
	} else if row == rooms[rn].bottom_row {
		Some(Down)
	} else if col == rooms[rn].left_col {
		Some(Left)
	} else if col == rooms[rn].right_col {
		Some(Right)
	} else {
		None
	};
	if let Some(d) = d {
		let d = d.to_index();
		if rooms[rn].doors[d].oth_room.is_some() {
			let row = rooms[rn].doors[d].oth_row.expect("oth row");
			let col = rooms[rn].doors[d].oth_col.expect("oth col");
			return Some((row, col));
		}
	}
	return None;
}
