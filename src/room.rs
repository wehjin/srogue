#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{addch, chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};
use crate::prelude::*;
use crate::prelude::DoorDirection::{Left, Right};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Object, Stairs, Trap, Tunnel, VertWall};
use crate::room::DoorDirection::{Up, Down};
use crate::room::RoomType::{Maze};

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct dr {
	pub oth_room: Option<usize>,
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
	pub fn is_type(&self, room_types: &Vec<RoomType>) -> bool {
		for room_type in room_types {
			if self == room_type {
				return true;
			}
		}
		return false;
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
	pub fn invert(&self) -> DoorDirection {
		match self {
			Up => Down,
			Right => Left,
			Down => Up,
			Left => Right,
		}
	}
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Room {
	pub bottom_row: i64,
	pub right_col: i64,
	pub left_col: i64,
	pub top_row: i64,
	pub doors: [door; 4],
	pub room_type: RoomType,
}

impl Room {
	pub fn center_spot(&self) -> DungeonSpot {
		DungeonSpot {
			col: (self.left_col + self.right_col) / 2,
			row: (self.top_row + self.bottom_row) / 2,
		}
	}
}

pub const MAX_ROOM: usize = 9;
pub static mut ROOMS: [Room; MAX_ROOM] = [Room {
	bottom_row: 0,
	right_col: 0,
	left_col: 0,
	top_row: 0,
	doors: [dr { oth_room: None, oth_row: None, oth_col: None, door_row: 0, door_col: 0 }; 4],
	room_type: RoomType::Nothing,
}; MAX_ROOM];
pub static mut ROOMS_VISITED: [bool; MAX_ROOM] = [false; MAX_ROOM];


pub unsafe fn light_up_room(rn: i64) {
	if blind == 0 {
		for i in ROOMS[rn as usize].top_row..=ROOMS[rn as usize].bottom_row {
			for j in ROOMS[rn as usize].left_col..=ROOMS[rn as usize].right_col {
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

pub unsafe fn darken_room(rn: i64) {
	for i in (ROOMS[rn as usize].top_row as usize + 1)..ROOMS[rn as usize].bottom_row as usize {
		for j in (ROOMS[rn as usize].left_col as usize + 1)..ROOMS[rn as usize].right_col as usize {
			if blind != 0 {
				mvaddch(i as i32, j as i32, chtype::from(' '));
			} else if !SpotFlag::is_any_set(&vec![Object, Stairs], dungeon[i][j])
				&& !(detect_monster && Monster.is_set(dungeon[i][j])) {
				if !imitating(i as i64, j as i64) {
					mvaddch(i as i32, j as i32, chtype::from(' '));
				}
				if Trap.is_set(dungeon[i][j]) && !Hidden.is_set(dungeon[i][j]) {
					mvaddch(i as i32, j as i32, chtype::from('^'));
				}
			}
		}
	}
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
			|| !(ROOMS[rn as usize].room_type == RoomType::Room || ROOMS[rn as usize].room_type == Maze)
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
		let i = get_rand(0, MAX_ROOM - 1);
		if ROOMS[i].room_type == RoomType::Room || ROOMS[i].room_type == Maze {
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
	N = ((ROOMS[rn as usize].bottom_row
		- ROOMS[rn as usize].top_row - 1)
		* (ROOMS[rn as usize].right_col
		- ROOMS[rn as usize].left_col - 1))
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
				ROOMS[rn as usize].top_row + 1,
				ROOMS[rn as usize].bottom_row - 1,
			) as libc::c_short;
			col = get_rand(
				ROOMS[rn as usize].left_col + 1,
				ROOMS[rn as usize].right_col - 1,
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
			place_at(&mut *obj, row as i64, col as i64);
			nf += 1;
		}
		i += 1;
	}
	return nf as i64;
}

pub fn get_room_number(row: i64, col: i64) -> i64 {
	unsafe {
		for i in 0..MAX_ROOM {
			let below_top_wall = row >= ROOMS[i].top_row;
			let above_bottom_wall = row <= ROOMS[i].bottom_row;
			let right_of_left_wall = col >= ROOMS[i].left_col;
			let left_of_right_wall = col <= ROOMS[i].right_col;
			if below_top_wall && above_bottom_wall && right_of_left_wall && left_of_right_wall {
				return i as i64;
			}
		}
	}
	return NO_ROOM;
}

pub fn get_opt_room_number(row: i64, col: i64) -> Option<usize> {
	let rn = get_room_number(row, col);
	if rn == NO_ROOM {
		None
	} else {
		Some(rn as usize)
	}
}

pub unsafe fn is_all_connected() -> bool {
	let mut starting_room = None;
	for i in 0..MAX_ROOM {
		ROOMS_VISITED[i] = false;
		if ROOMS[i].room_type == RoomType::Room || ROOMS[i].room_type == Maze {
			starting_room = Some(i);
		}
	}
	if let Some(rn) = starting_room {
		visit_rooms(rn);
	}
	for i in 0..MAX_ROOM {
		if (ROOMS[i].room_type == RoomType::Room || ROOMS[i].room_type == Maze)
			&& ROOMS_VISITED[i] == false {
			return false;
		}
	}
	return true;
}

pub unsafe fn visit_rooms(rn: usize) {
	ROOMS_VISITED[rn] = true;
	for i in 0..4 {
		if let Some(oth_rn) = ROOMS[rn].doors[i].oth_room {
			if ROOMS_VISITED[oth_rn] == false {
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
		let r = get_rand(0, MAX_ROOM - 1);
		for i in 0..MAX_ROOM {
			let rr = (r + i) % MAX_ROOM;
			if !(ROOMS[rr].room_type == RoomType::Room || ROOMS[rr].room_type == Maze) || Some(rr) == rn {
				continue;
			}
			for k in 0..4 {
				if ROOMS[rr].doors[k].oth_room == rn {
					(*monster).trow = ROOMS[rr].doors[k].oth_row.expect("oth row");
					(*monster).tcol = ROOMS[rr].doors[k].oth_col.expect("oth col");
					if (*monster).trow == row && (*monster).tcol == col {
						continue;
					}
					return;
				}
			}
		}
		/* look for door to dead end */
		let rn = rn.expect("rn");
		for i in ROOMS[rn].top_row..=ROOMS[rn].bottom_row {
			for j in ROOMS[rn].left_col..=ROOMS[rn].right_col {
				if i != (*monster).row && j != (*monster).col && Door.is_set(dungeon[i as usize][j as usize]) {
					(*monster).trow = i;
					(*monster).tcol = j;
					return;
				}
			}
		}
		/* return monster to room that he came from */
		for i in 0..MAX_ROOM {
			for j in 0..4usize {
				if ROOMS[i].doors[j].oth_room == Some(rn) {
					for k in 0..4usize {
						if ROOMS[rn].doors[k].oth_room == Some(i) {
							(*monster).trow = ROOMS[rn].doors[k].oth_row.expect("oth row");
							(*monster).tcol = ROOMS[rn].doors[k].oth_col.expect("oth col");
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
				if let Some((row, col)) = get_oth_room(rn as i64, row, col) {
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
	let d = if row == ROOMS[rn].top_row {
		Some(Up)
	} else if row == ROOMS[rn].bottom_row {
		Some(Down)
	} else if col == ROOMS[rn].left_col {
		Some(Left)
	} else if col == ROOMS[rn].right_col {
		Some(Right)
	} else {
		None
	};
	if let Some(d) = d {
		let d = d.to_index();
		if ROOMS[rn].doors[d].oth_room.is_some() {
			let row = ROOMS[rn].doors[d].oth_row.expect("oth row");
			let col = ROOMS[rn].doors[d].oth_col.expect("oth col");
			return Some((row, col));
		}
	}
	return None;
}
