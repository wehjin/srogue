#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments)]

use std::ops::{RangeInclusive};
use ncurses::{addch, chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};
use crate::level::constants::{DCOLS, DROWS, MAX_ROOM};
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::DoorDirection::{Left, Right};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::RoomType::Nothing;
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

impl Default for RoomType {
	fn default() -> Self { Nothing }
}

impl RoomType {
	pub fn is_nothing(&self) -> bool {
		*self == Nothing
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
			Some(Left)
		} else if same_row(start_room, end_room) && (level.rooms[end_room].left_col > level.rooms[start_room].right_col) {
			Some(Right)
		} else if same_col(start_room, end_room) && (level.rooms[start_room].top_row > level.rooms[end_room].bottom_row) {
			Some(Up)
		} else if same_col(start_room, end_room) && (level.rooms[end_room].top_row > level.rooms[start_room].bottom_row) {
			Some(Down)
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

#[derive(Copy, Clone, Default)]
pub struct RoomBounds {
	pub top: i64,
	pub right: i64,
	pub bottom: i64,
	pub left: i64,
}

impl RoomBounds {
	pub fn cell_kind_for_spot(&self, spot: &DungeonSpot) -> CellKind {
		if spot.row == self.top || spot.row == self.bottom {
			CellKind::HorizontalWall
		} else if spot.col == self.left || spot.col == self.right {
			CellKind::VerticalWall
		} else {
			CellKind::Floor
		}
	}
	pub fn rows(&self) -> RangeInclusive<i64> {
		self.top..=self.bottom
	}
	pub fn cols(&self) -> RangeInclusive<i64> {
		self.left..=self.right
	}
}

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
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
	pub fn set_bounds(&mut self, bounds: &RoomBounds) {
		self.top_row = bounds.top;
		self.right_col = bounds.right;
		self.bottom_row = bounds.bottom;
		self.left_col = bounds.left;
	}
	pub fn to_bounds(&self) -> RoomBounds {
		RoomBounds { top: self.top_row, right: self.right_col, bottom: self.bottom_row, left: self.left_col }
	}
	pub fn to_floor_bounds(&self) -> RoomBounds {
		RoomBounds { top: self.top_row + 1, right: self.right_col + 1, bottom: self.bottom_row - 1, left: self.left_col - 1 }
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

pub unsafe fn light_up_room(rn: i64, player: &Player, level: &mut Level) {
	if blind == 0 {
		for i in level.rooms[rn as usize].top_row..=level.rooms[rn as usize].bottom_row {
			for j in level.rooms[rn as usize].left_col..=level.rooms[rn as usize].right_col {
				if level.dungeon[i as usize][j as usize].is_monster() {
					if let Some(monster) = MASH.monster_at_spot_mut(i, j) {
						level.dungeon[monster.spot.row as usize][monster.spot.col as usize].remove_kind(CellKind::Monster);
						monster.trail_char = get_dungeon_char(monster.spot.row, monster.spot.col, level);
						level.dungeon[monster.spot.row as usize][monster.spot.col as usize].add_kind(CellKind::Monster);
					}
				}
				mvaddch(i as i32, j as i32, get_dungeon_char(i, j, level));
			}
		}
		mvaddch(player.rogue.row as i32, player.rogue.col as i32, player.rogue.fchar as chtype);
	}
}


pub unsafe fn light_passage(row: i64, col: i64, level: &Level) {
	if blind != 0 {
		return;
	}
	let i_end = if row < DROWS as i64 - 2 { 1 } else { 0 };
	let j_end = if col < DCOLS as i64 - 1 { 1 } else { 0 };
	let i_start = if row > MIN_ROW { -1 } else { 0 };
	let j_start = if col > 0 { -1 } else { 0 };
	for i in i_start..=i_end {
		for j in j_start..=j_end {
			if can_move(row, col, row + i, col + j, level) {
				mvaddch((row + i) as i32, (col + j) as i32, get_dungeon_char(row + i, col + j, level));
			}
		}
	}
}

pub unsafe fn darken_room(rn: i64, level: &Level) {
	for i in (level.rooms[rn as usize].top_row as usize + 1)..level.rooms[rn as usize].bottom_row as usize {
		for j in (level.rooms[rn as usize].left_col as usize + 1)..level.rooms[rn as usize].right_col as usize {
			if blind != 0 {
				mvaddch(i as i32, j as i32, chtype::from(' '));
			} else if !level.dungeon[i][j].is_any_kind(&[CellKind::Object, CellKind::Stairs])
				&& !(level.detect_monster && level.dungeon[i][j].is_monster()) {
				if !imitating(i as i64, j as i64, level) {
					mvaddch(i as i32, j as i32, chtype::from(' '));
				}
				if level.dungeon[i][j].is_trap() && !level.dungeon[i][j].is_hidden() {
					mvaddch(i as i32, j as i32, chtype::from('^'));
				}
			}
		}
	}
}

pub unsafe fn get_dungeon_char(row: i64, col: i64, level: &Level) -> chtype {
	let mask = level.dungeon[row as usize][col as usize];
	if mask.is_monster() {
		return gmc_row_col(row, col, level);
	}
	if mask.is_object() {
		let obj = level_objects.find_object_at(row, col).expect("obj at row,col in level object");
		return get_mask_char(obj.what_is) as chtype;
	}
	let CHAR_CELL_KINDS = [CellKind::Tunnel, CellKind::Stairs, CellKind::HorizontalWall, CellKind::VerticalWall, CellKind::Floor, CellKind::Door];
	if mask.is_any_kind(&CHAR_CELL_KINDS) {
		if mask.is_any_kind(&[CellKind::Tunnel, CellKind::Stairs]) && !mask.is_hidden() {
			return if mask.is_stairs() { chtype::from('%') } else { chtype::from('#') };
		}
		if mask.is_kind(CellKind::HorizontalWall) {
			return chtype::from('-');
		}
		if mask.is_kind(CellKind::VerticalWall) {
			return chtype::from('|');
		}
		if mask.is_floor() {
			if mask.is_trap() {
				if !mask.is_hidden() {
					return chtype::from('^');
				}
			}
			return chtype::from('.');
		}
		if mask.is_door() {
			return if mask.is_hidden() {
				if (col > 0 && level.dungeon[row as usize][col as usize - 1].is_kind(CellKind::HorizontalWall))
					|| (col < (DCOLS - 1) as i64 && level.dungeon[row as usize][col as usize + 1].is_kind(CellKind::HorizontalWall)) {
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

pub unsafe fn random_spot_with_flag(flags: &[CellKind], player: &Player, level: &Level) -> DungeonSpot {
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	gr_row_col(&mut row, &mut col, flags, player, level);
	DungeonSpot { row, col }
}

pub unsafe fn gr_row_col(row: &mut i64, col: &mut i64, kinds: &[CellKind], player: &Player, level: &Level) {
	let mut r = 0;
	let mut c = 0;
	loop {
		r = get_rand(MIN_ROW, DROWS as i64 - 2);
		c = get_rand(0, DCOLS as i64 - 1);
		let rn = get_room_number(r, c, level);
		let keep_looking = rn == NO_ROOM
			|| !level.dungeon[r as usize][c as usize].is_any_kind(kinds)
			|| level.dungeon[r as usize][c as usize].is_other_kind(&kinds)
			|| !(level.rooms[rn as usize].room_type == RoomType::Room || level.rooms[rn as usize].room_type == Maze)
			|| ((r == player.rogue.row) && (c == player.rogue.col));
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

pub unsafe fn party_objects(rn: usize, level_depth: usize, level: &mut Level) -> usize {
	let N = (level.rooms[rn].bottom_row - level.rooms[rn].top_row - 1) * (level.rooms[rn].right_col - level.rooms[rn].left_col - 1);
	let mut n = get_rand(5, 10);
	if n > N {
		n = N - 2;
	}
	let mut number_found: usize = 0;
	for _i in 0..n {
		let mut found = None;
		for _j in 0..250 {
			let row = get_rand(level.rooms[rn].top_row + 1, level.rooms[rn].bottom_row - 1);
			let col = get_rand(level.rooms[rn].left_col + 1, level.rooms[rn].right_col - 1);
			if level.dungeon[row as usize][col as usize].is_only_kind(CellKind::Floor)
				|| level.dungeon[row as usize][col as usize].is_only_kind(CellKind::Tunnel) {
				found = Some(DungeonSpot { row, col });
				break;
			}
		}
		if let Some(found) = found {
			let obj = gr_object(level_depth);
			place_at(obj, found.row, found.col, level);
			number_found += 1;
		}
	}
	number_found
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
	use crate::level::constants::MAX_ROOM;
	use crate::room::{Room, RoomType};

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

pub unsafe fn draw_magic_map(level: &mut Level) {
	let mask = [
		CellKind::HorizontalWall, CellKind::VerticalWall, CellKind::Door,
		CellKind::Tunnel, CellKind::Trap, CellKind::Stairs,
		CellKind::Monster
	];
	for i in 0..DROWS {
		for j in 0..DCOLS {
			let s = level.dungeon[i][j];
			if s.is_any_kind(&mask) {
				let mut ch = mvinch(i as i32, j as i32) as u8 as char;
				if ch == ' ' || ch.is_ascii_uppercase() || s.is_any_kind(&[CellKind::Trap, CellKind::Hidden]) {
					let och = ch;
					level.dungeon[i][j].remove_kind(CellKind::Hidden);
					if s.is_kind(CellKind::HorizontalWall) {
						ch = '-';
					} else if s.is_kind(CellKind::VerticalWall) {
						ch = '|';
					} else if s.is_door() {
						ch = '+';
					} else if s.is_trap() {
						ch = '^';
					} else if s.is_stairs() {
						ch = '%';
					} else if s.is_tunnel() {
						ch = '#';
					} else {
						continue;
					}
					if !s.is_monster() || och == ' ' {
						addch(chtype::from(ch));
					}
					if s.is_monster() {
						if let Some(monster) = MASH.monster_at_spot_mut(i as i64, j as i64) {
							monster.trail_char = chtype::from(ch);
						}
					}
				}
			}
		}
	}
}

pub unsafe fn dr_course(monster: &mut Monster, entering: bool, row: i64, col: i64, player: &Player, level: &Level) {
	monster.spot.row = row;
	monster.spot.col = col;
	if mon_sees(monster, player.rogue.row, player.rogue.col, level) {
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
				if (i != monster.spot.row && j != monster.spot.col)
					&& level.dungeon[i as usize][j as usize].is_door() {
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
