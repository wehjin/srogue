#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{addch, chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};
use crate::monster;
use crate::prelude::*;
use crate::prelude::DoorDirection::{Left, Right};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::RoomType::Nothing;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Object, Stairs, Trap, Tunnel, VertWall};
use crate::room::DoorDirection::{Up, Down};
use crate::room::room_visitor::RoomVisitor;
use crate::room::RoomType::{Maze};

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct dr {
	pub oth_room: Option<usize>,
	pub oth_row: Option<i64>,
	pub oth_col: Option<i64>,
	pub door_row: i64,
	pub door_col: i64,
}

impl dr {
	pub fn set_others(&mut self, other_rn: usize, other_spot: &DungeonSpot) {
		self.oth_room = Some(other_rn);
		self.oth_row = Some(other_spot.row);
		self.oth_col = Some(other_spot.col);
	}
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
	pub fn is_cross(&self) -> bool {
		*self == RoomType::Cross
	}
	pub fn is_type(&self, room_types: &Vec<RoomType>) -> bool {
		room_types.iter().position(|rt| self == rt).is_some()
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
	pub fn from_room_to_room(start_room: usize, end_room: usize, level: &Level) -> Option<Self> {
		if same_row(start_room, end_room) && (level.rooms[start_room].left_col > level.rooms[end_room].right_col) {
			Some(DoorDirection::Left)
		} else if same_row(start_room, end_room) && (level.rooms[end_room].left_col > level.rooms[start_room].right_col) {
			Some(DoorDirection::Right)
		} else if same_col(start_room, end_room) && (level.rooms[start_room].top_row > level.rooms[end_room].bottom_row) {
			Some(DoorDirection::Up)
		} else if same_col(start_room, end_room) && (level.rooms[end_room].top_row > level.rooms[start_room].bottom_row) {
			Some(DoorDirection::Down)
		} else {
			None
		}
	}
	pub fn to_index(&self) -> usize {
		match self {
			Up => 0,
			Right => 1,
			Down => 2,
			Left => 3,
		}
	}
	pub fn inverse(&self) -> DoorDirection {
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
	pub fn clear(&mut self) {
		self.room_type = Nothing;
		for dn in 0..4 {
			self.doors[dn].oth_room = None;
		}
	}
	pub fn contains_spot(&self, row: i64, col: i64) -> bool {
		let below_top_wall = row >= self.top_row;
		let above_bottom_wall = row <= self.bottom_row;
		let right_of_left_wall = col >= self.left_col;
		let left_of_right_wall = col <= self.right_col;
		below_top_wall && above_bottom_wall && right_of_left_wall && left_of_right_wall
	}
	pub fn center_spot(&self) -> DungeonSpot {
		DungeonSpot {
			col: (self.left_col + self.right_col) / 2,
			row: (self.top_row + self.bottom_row) / 2,
		}
	}
}

pub unsafe fn light_up_room(rn: i64, level: &Level) {
	if blind == 0 {
		for i in level.rooms[rn as usize].top_row..=level.rooms[rn as usize].bottom_row {
			for j in level.rooms[rn as usize].left_col..=level.rooms[rn as usize].right_col {
				if Monster.is_set(dungeon[i as usize][j as usize]) {
					if let Some(monster) = MASH.monster_at_spot_mut(i, j) {
						Monster.clear(&mut dungeon[monster.spot.row as usize][monster.spot.col as usize]);
						monster.trail_char = get_dungeon_char(monster.spot.row, monster.spot.col);
						Monster.set(&mut dungeon[monster.spot.row as usize][monster.spot.col as usize]);
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

pub unsafe fn darken_room(rn: i64, level: &Level) {
	for i in (level.rooms[rn as usize].top_row as usize + 1)..level.rooms[rn as usize].bottom_row as usize {
		for j in (level.rooms[rn as usize].left_col as usize + 1)..level.rooms[rn as usize].right_col as usize {
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

pub unsafe fn random_spot_with_flag(flags: Vec<SpotFlag>, level: &Level) -> DungeonSpot {
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	gr_row_col(&mut row, &mut col, flags, level);
	DungeonSpot { row, col }
}

pub unsafe fn gr_row_col(row: &mut i64, col: &mut i64, spots: Vec<SpotFlag>, level: &Level) {
	let mut r = 0;
	let mut c = 0;
	loop {
		r = get_rand(MIN_ROW, DROWS as i64 - 2);
		c = get_rand(0, DCOLS as i64 - 1);
		let rn = get_room_number(r, c, level);
		let keep_looking = rn == NO_ROOM
			|| !SpotFlag::is_any_set(&spots, dungeon[r as usize][c as usize])
			|| SpotFlag::are_others_set(&spots, dungeon[r as usize][c as usize])
			|| !(level.rooms[rn as usize].room_type == RoomType::Room || level.rooms[rn as usize].room_type == Maze)
			|| ((r == rogue.row) && (c == rogue.col));
		if !keep_looking {
			break;
		}
	}
	*row = r;
	*col = c;
}

pub unsafe fn gr_room(level: &Level) -> usize {
	loop {
		let i = get_rand(0, MAX_ROOM - 1);
		if level.rooms[i].room_type == RoomType::Room || level.rooms[i].room_type == Maze {
			return i;
		}
	}
}

#[no_mangle]
pub unsafe extern "C" fn party_objects(rn: usize, level_depth: usize, level: &Level) -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut nf: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut n: libc::c_short = 0;
	let mut N: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut found: libc::c_char = 0;
	N = ((level.rooms[rn].bottom_row
		- level.rooms[rn].top_row - 1)
		* (level.rooms[rn].right_col
		- level.rooms[rn].left_col - 1))
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
				level.rooms[rn].top_row + 1,
				level.rooms[rn].bottom_row - 1,
			) as libc::c_short;
			col = get_rand(
				level.rooms[rn].left_col + 1,
				level.rooms[rn].right_col - 1,
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
			obj = gr_object(level_depth);
			place_at(&mut *obj, row as i64, col as i64);
			nf += 1;
		}
		i += 1;
	}
	return nf as i64;
}

pub fn get_room_number(row: i64, col: i64, level: &Level) -> i64 {
	for i in 0..MAX_ROOM {
		if level.rooms[i].contains_spot(row, col) {
			return i as i64;
		}
	}
	return NO_ROOM;
}

pub fn get_opt_room_number(row: i64, col: i64, level: &Level) -> Option<usize> {
	let rn = get_room_number(row, col, level);
	if rn == NO_ROOM {
		None
	} else {
		Some(rn as usize)
	}
}

mod room_visitor {
	use crate::room::{MAX_ROOM, Room, RoomType};

	pub struct Unvisited {
		starting_room: Option<usize>,
	}

	pub struct Visited {
		visited: [bool; MAX_ROOM],
	}

	pub struct RoomVisitor<'a, ST> {
		rooms: &'a [Room; MAX_ROOM],
		state: ST,
	}

	impl<'a, ST> RoomVisitor<'a, ST> {
		fn is_included_room_type(room_type: RoomType) -> bool {
			vec![RoomType::Room, RoomType::Maze].iter().position(|acceptable| room_type == *acceptable).is_some()
		}
	}

	impl<'a> RoomVisitor<'a, Unvisited> {
		pub fn new(rooms: &'a [Room; MAX_ROOM]) -> Self {
			let starting_room = rooms.iter().map(|room| room.room_type).position(Self::is_included_room_type);
			RoomVisitor { rooms, state: Unvisited { starting_room } }
		}
		pub fn visit_rooms(&self) -> RoomVisitor<Visited> {
			let mut visited = [false; MAX_ROOM];
			if let Some(rn) = self.state.starting_room {
				self.visit_each_room(rn, &mut visited);
			}
			RoomVisitor { rooms: self.rooms, state: Visited { visited } }
		}
		fn visit_each_room(&self, rn: usize, visited: &mut [bool; MAX_ROOM]) {
			visited[rn] = true;
			for dn in 0..4 {
				if let Some(other_rn) = self.rooms[rn].doors[dn].oth_room {
					if visited[other_rn] == false {
						self.visit_each_room(other_rn, visited);
					}
				}
			}
		}
	}

	impl<'a> RoomVisitor<'a, Visited> {
		pub fn to_is_all_connected(&self) -> bool {
			for rn in 0..MAX_ROOM {
				if Self::is_included_room_type(self.rooms[rn].room_type) && !self.state.visited[rn] {
					return false;
				}
			}
			true
		}
	}
}

pub unsafe fn is_all_connected(rooms: &[Room; MAX_ROOM]) -> bool {
	RoomVisitor::new(rooms).visit_rooms().to_is_all_connected()
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
						if let Some(monster) = MASH.monster_at_spot_mut(i as i64, j as i64) {
							monster.trail_char = chtype::from(ch);
						}
					}
				}
			}
		}
	}
}

pub unsafe fn dr_course(monster: &mut monster::Monster, entering: bool, row: i64, col: i64, level: &Level) {
	monster.spot.row = row;
	monster.spot.col = col;
	if mon_sees(monster, rogue.row, rogue.col, level) {
		monster.clear_target_spot();
		return;
	}
	let rn = get_opt_room_number(row, col, level);
	if entering {
		/* look for door to some other room */
		let r = get_rand(0, MAX_ROOM - 1);
		for i in 0..MAX_ROOM {
			let rr = (r + i) % MAX_ROOM;
			if !(level.rooms[rr].room_type == RoomType::Room || level.rooms[rr].room_type == Maze) || Some(rr) == rn {
				continue;
			}
			for k in 0..4 {
				if level.rooms[rr].doors[k].oth_room == rn {
					monster.set_target_spot(
						level.rooms[rr].doors[k].oth_row.expect("oth row"),
						level.rooms[rr].doors[k].oth_col.expect("oth col"),
					);
					let monster_target_spot = monster.target_spot.expect("target spot");
					if monster_target_spot.is_at(row, col) {
						continue;
					}
					return;
				}
			}
		}
		/* look for door to dead end */
		let rn = rn.expect("rn");
		for i in level.rooms[rn].top_row..=level.rooms[rn].bottom_row {
			for j in level.rooms[rn].left_col..=level.rooms[rn].right_col {
				if i != monster.spot.row && j != monster.spot.col && Door.is_set(dungeon[i as usize][j as usize]) {
					monster.set_target_spot(i, j);
					return;
				}
			}
		}
		/* return monster to room that he came from */
		for i in 0..MAX_ROOM {
			for j in 0..4usize {
				if level.rooms[i].doors[j].oth_room == Some(rn) {
					for k in 0..4usize {
						if level.rooms[rn].doors[k].oth_room == Some(i) {
							monster.set_target_spot(
								level.rooms[rn].doors[k].oth_row.expect("oth row"),
								level.rooms[rn].doors[k].oth_col.expect("oth col"),
							);
							return;
						}
					}
				}
			}
		}
		monster.clear_target_spot();
	} else {
		//* exiting room */
		if let Some(rn) = rn {
			if let Some((row, col)) = get_oth_room(rn as i64, row, col, level) {
				monster.set_target_spot(row, col);
			} else {
				monster.clear_target_spot();
			}
		} else if let None = rn {
			monster.clear_target_spot();
		}
	}
}

pub unsafe fn get_oth_room(rn: i64, row: i64, col: i64, level: &Level) -> Option<(i64, i64)> {
	let rn = rn as usize;
	let d = if row == level.rooms[rn].top_row {
		Some(Up)
	} else if row == level.rooms[rn].bottom_row {
		Some(Down)
	} else if col == level.rooms[rn].left_col {
		Some(Left)
	} else if col == level.rooms[rn].right_col {
		Some(Right)
	} else {
		None
	};
	if let Some(d) = d {
		let d = d.to_index();
		if level.rooms[rn].doors[d].oth_room.is_some() {
			let row = level.rooms[rn].doors[d].oth_row.expect("oth row");
			let col = level.rooms[rn].doors[d].oth_col.expect("oth col");
			return Some((row, col));
		}
	}
	return None;
}
