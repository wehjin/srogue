use std::ops::RangeInclusive;

use ncurses::{addch, chtype, mvaddch, mvinch};
use serde::{Deserialize, Serialize};

use crate::init::GameState;
use crate::level::{DungeonCell, Level, same_col, same_row};
use crate::level::constants::{DCOLS, DROWS, MAX_ROOM};
use crate::level::materials::{CellMaterial, Fixture, Visibility};
use crate::monster::{gmc_row_col, Monster, MonsterMash};
use crate::motion::can_move;
use crate::objects::{gr_object, place_at};
use crate::player::{Player, RoomMark};
use crate::prelude::*;
use crate::random::get_rand;
use crate::room::DoorDirection::{Down, Left, Right, Up};
use crate::room::room_visitor::RoomVisitor;
use crate::spec_hit::imitating;

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct Dr {
	pub oth_room: Option<usize>,
	pub oth_row: Option<i64>,
	pub oth_col: Option<i64>,
	pub door_row: i64,
	pub door_col: i64,
}

impl Dr {
	pub fn set_others(&mut self, other_rn: usize, other_spot: &DungeonSpot) {
		self.oth_room = Some(other_rn);
		self.oth_row = Some(other_spot.row);
		self.oth_col = Some(other_spot.col);
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RoomType {
	Nothing,
	Room,
	Maze,
	DeadEnd,
	Cross,
}

impl Default for RoomType {
	fn default() -> Self { RoomType::Nothing }
}

impl RoomType {
	pub fn is_nothing(&self) -> bool {
		*self == RoomType::Nothing
	}
	pub fn is_cross(&self) -> bool {
		*self == RoomType::Cross
	}
	pub fn is_type<T: AsRef<[RoomType]>>(&self, room_types: T) -> bool {
		room_types.as_ref().iter().position(|rt| self == rt).is_some()
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DoorDirection {
	Up,
	Down,
	Left,
	Right,
}

impl DoorDirection {
	pub fn is_up_or_down(&self) -> bool {
		match self {
			Up | Down => true,
			Left | Right => false,
		}
	}
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
	pub fn inverse(&self) -> Self {
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
	pub fn cell_material_for_spot(&self, spot: &DungeonSpot) -> CellMaterial {
		if spot.row == self.top || spot.row == self.bottom {
			CellMaterial::HorizontalWall
		} else if spot.col == self.left || spot.col == self.right {
			CellMaterial::VerticalWall
		} else {
			CellMaterial::Floor(Fixture::None)
		}
	}
	pub fn rows(&self) -> RangeInclusive<i64> {
		self.top..=self.bottom
	}
	pub fn rows_usize(&self) -> RangeInclusive<usize> {
		(self.top as usize)..=(self.bottom as usize)
	}
	pub fn cols(&self) -> RangeInclusive<i64> {
		self.left..=self.right
	}
	pub fn cols_usize(&self) -> RangeInclusive<usize> {
		(self.left as usize)..=(self.right as usize)
	}
}

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct Room {
	pub bottom_row: i64,
	pub right_col: i64,
	pub left_col: i64,
	pub top_row: i64,
	pub doors: [Dr; 4],
	pub room_type: RoomType,
}

impl Room {
	pub fn is_type<T: AsRef<[RoomType]>>(&self, room_types: T) -> bool {
		self.room_type.is_type(room_types)
	}
	pub fn clear(&mut self) {
		self.room_type = RoomType::Nothing;
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
	pub fn to_wall_bounds(&self) -> RoomBounds {
		RoomBounds {
			top: self.top_row,
			right: self.right_col,
			bottom: self.bottom_row,
			left: self.left_col,
		}
	}
	pub fn to_floor_bounds(&self) -> RoomBounds {
		RoomBounds {
			top: self.top_row + 1,
			right: self.right_col - 1,
			bottom: self.bottom_row - 1,
			left: self.left_col + 1,
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

pub fn light_up_room(rn: usize, game: &mut GameState) {
	if game.player.blind.is_active() {
		return;
	}
	let wall_bounds = game.level.rooms[rn].to_wall_bounds();
	for i in wall_bounds.rows() {
		for j in wall_bounds.cols() {
			if game.level.cell(i, j).has_monster() {
				if let Some(mon_id) = game.mash.monster_id_at_spot(i, j) {
					let monster_spot = {
						let monster = game.mash.monster_mut(mon_id);
						monster.cell_mut(&mut game.level).set_monster(false);
						monster.spot
					};
					let dungeon_char = get_dungeon_char_spot(monster_spot, game);
					{
						let monster = game.mash.monster_mut(mon_id);
						monster.trail_char = dungeon_char;
						monster.cell_mut(&mut game.level).set_monster(true);
					}
				}
			}
			mvaddch(i as i32, j as i32, get_dungeon_char(i, j, game));
		}
	}
	mvaddch(game.player.rogue.row as i32, game.player.rogue.col as i32, game.player.rogue.fchar as chtype);
}


pub fn light_passage(row: i64, col: i64, game: &mut GameState) {
	if game.player.blind.is_active() {
		return;
	}
	let i_end = if row < DROWS as i64 - 2 { 1 } else { 0 };
	let j_end = if col < DCOLS as i64 - 1 { 1 } else { 0 };
	let i_start = if row > MIN_ROW { -1 } else { 0 };
	let j_start = if col > 0 { -1 } else { 0 };
	for i in i_start..=i_end {
		for j in j_start..=j_end {
			if can_move(row, col, row + i, col + j, &game.level) {
				mvaddch((row + i) as i32, (col + j) as i32, get_dungeon_char(row + i, col + j, game));
			}
		}
	}
}

pub fn darken_room(rn: usize, game: &mut GameState) {
	let floor_bounds = game.level.rooms[rn].to_floor_bounds();
	for i in floor_bounds.rows_usize() {
		for j in floor_bounds.cols_usize() {
			if game.player.blind.is_active() {
				mvaddch(i as i32, j as i32, chtype::from(' '));
			} else {
				let cell = &game.level.dungeon[i][j];
				let cell_remains_lit = {
					let cell_is_detected_monster = game.level.detect_monster && cell.has_monster();
					cell_is_detected_monster || cell.has_object() || cell.is_stairs()
				};
				if !cell_remains_lit {
					if !imitating(i as i64, j as i64, &mut game.mash, &game.level) {
						mvaddch(i as i32, j as i32, chtype::from(' '));
					}
					if cell.is_any_trap() && !game.level.dungeon[i][j].is_any_hidden() {
						mvaddch(i as i32, j as i32, chtype::from('^'));
					}
				}
			}
		}
	}
}

pub fn get_dungeon_char_spot(spot: DungeonSpot, game: &mut GameState) -> chtype {
	get_dungeon_char(spot.row, spot.col, game)
}

pub fn get_dungeon_char(row: i64, col: i64, game: &mut GameState) -> chtype {
	let cell = game.level.dungeon[row as usize][col as usize];
	if cell.has_monster() {
		return gmc_row_col(row, col, game);
	}
	let ch = cell.to_char_ignoring_any_monster();
	chtype::from(ch)
}

pub fn random_spot_with_flag(is_good_cell: impl Fn(&DungeonCell) -> bool, player: &Player, level: &Level) -> DungeonSpot {
	let mut row: i64 = 0;
	let mut col: i64 = 0;
	gr_row_col(&mut row, &mut col, is_good_cell, player, level);
	DungeonSpot { row, col }
}

pub fn gr_row_col(row: &mut i64, col: &mut i64, is_good_cell: impl Fn(&DungeonCell) -> bool, player: &Player, level: &Level) {
	let mut r;
	let mut c;
	loop {
		r = get_rand(MIN_ROW, DROWS as i64 - 2);
		c = get_rand(0, DCOLS as i64 - 1);
		let rn = get_room_number(r, c, level);
		let keep_looking = rn == NO_ROOM
			|| !is_good_cell(&level.dungeon[r as usize][c as usize])
			|| !(level.rooms[rn as usize].room_type == RoomType::Room || level.rooms[rn as usize].room_type == RoomType::Maze)
			|| ((r == player.rogue.row) && (c == player.rogue.col));
		if !keep_looking {
			break;
		}
	}
	*row = r;
	*col = c;
}

pub fn gr_room(level: &Level) -> usize {
	loop {
		let i = get_rand(0, MAX_ROOM - 1);
		if level.rooms[i].room_type == RoomType::Room || level.rooms[i].room_type == RoomType::Maze {
			return i;
		}
	}
}

pub fn party_objects(rn: usize, game: &mut GameState) -> usize {
	let area = (game.level.rooms[rn].bottom_row - game.level.rooms[rn].top_row - 1) * (game.level.rooms[rn].right_col - game.level.rooms[rn].left_col - 1);
	let mut n = get_rand(5, 10);
	if n > area {
		n = area - 2;
	}
	let mut number_found: usize = 0;
	for _i in 0..n {
		let mut found = None;
		for _j in 0..250 {
			let row = get_rand(game.level.rooms[rn].top_row + 1, game.level.rooms[rn].bottom_row - 1);
			let col = get_rand(game.level.rooms[rn].left_col + 1, game.level.rooms[rn].right_col - 1);
			if game.level.dungeon[row as usize][col as usize].is_material_no_others(CellMaterial::Floor(Fixture::None))
				|| game.level.dungeon[row as usize][col as usize].is_material_no_others(CellMaterial::Tunnel(Visibility::Visible, Fixture::None)) {
				found = Some(DungeonSpot { row, col });
				break;
			}
		}
		if let Some(found) = found {
			let obj = gr_object(&mut game.player);
			place_at(obj, found.row, found.col, &mut game.level, &mut game.ground);
			number_found += 1;
		}
	}
	number_found
}

pub fn get_room_number(row: i64, col: i64, level: &Level) -> i64 {
	let room = level.room(row, col);
	match room {
		RoomMark::None => NO_ROOM,
		RoomMark::Passage => PASSAGE,
		RoomMark::Cavern(rn) => rn as i64
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

pub fn is_all_connected(rooms: &[Room; MAX_ROOM]) -> bool {
	RoomVisitor::new(rooms).visit_rooms().to_is_all_connected()
}

mod magic_map {
	use crate::level::DungeonCell;

	pub fn reveals_cell(cell: &DungeonCell) -> bool {
		cell.is_any_wall()
			|| cell.is_any_door()
			|| cell.is_any_tunnel()
			|| cell.is_any_trap()
			|| cell.is_stairs()
			|| cell.has_monster()
	}
}

pub fn draw_magic_map(mash: &mut MonsterMash, level: &mut Level) {
	for i in 0..DROWS {
		for j in 0..DCOLS {
			let cell = level.dungeon[i][j];
			if magic_map::reveals_cell(&cell) {
				let old_ch = mvinch(i as i32, j as i32) as u8 as char;
				if old_ch == ' ' || old_ch.is_ascii_uppercase() || cell.is_any_trap() || cell.is_any_hidden() {
					level.dungeon[i][j].set_visible();
					let ch;
					if cell.is_horizontal_wall() {
						ch = '-';
					} else if cell.is_vertical_wall() {
						ch = '|';
					} else if cell.is_any_door() {
						ch = '+';
					} else if cell.is_any_trap() {
						ch = '^';
					} else if cell.is_stairs() {
						ch = '%';
					} else if cell.is_any_tunnel() {
						ch = '#';
					} else {
						continue;
					}
					if !cell.has_monster() || old_ch == ' ' {
						addch(chtype::from(ch));
					}
					if cell.has_monster() {
						if let Some(monster) = mash.monster_at_spot_mut(i as i64, j as i64) {
							monster.trail_char = chtype::from(ch);
						}
					}
				}
			}
		}
	}
}

const ROOM_OR_MAZE: [RoomType; 2] = [RoomType::Room, RoomType::Maze];

pub fn dr_course(monster: &mut Monster, is_entering: bool, row: i64, col: i64, player: &Player, level: &Level) {
	monster.spot.row = row;
	monster.spot.col = col;
	if monster.sees(player.rogue.row, player.rogue.col, level) {
		monster.clear_target_spot();
		return;
	}
	let mon_room = monster.cur_room(level);
	if is_entering {
		/* look for door to some other room */
		let random_start = get_rand(0, MAX_ROOM - 1);
		for i in 0..MAX_ROOM {
			let rr = (random_start + i) % MAX_ROOM;
			let room = level.rooms[rr];
			let room_mark = RoomMark::Cavern(rr);
			if !room.is_type(&ROOM_OR_MAZE) || room_mark == mon_room {
				continue;
			}
			let mon_rn = mon_room.rn();
			for k in 0..4 {
				if mon_rn == room.doors[k].oth_room {
					monster.set_target_spot(
						room.doors[k].oth_row.expect("oth row"),
						room.doors[k].oth_col.expect("oth col"),
					);
					let target_spot = monster.target_spot.expect("target spot");
					if target_spot.is_at(row, col) {
						continue;
					}
					return;
				}
			}
		}
		/* look for door to dead end */
		let mon_rn = mon_room.rn().expect("not an area");
		let mon_room = level.rooms[mon_rn];
		{
			let wall_bounds = mon_room.to_wall_bounds();
			for i in wall_bounds.rows_usize() {
				for j in wall_bounds.cols_usize() {
					let spot = DungeonSpot::from_usize(i, j);
					if level.dungeon[i][j].is_any_door() && !spot.shares_axis(&monster.spot) {
						monster.set_target_spot(i as i64, j as i64);
						return;
					}
				}
			}
		}
		/* return monster to room that he came from */
		for i in 0..MAX_ROOM {
			for j in 0..4usize {
				if level.rooms[i].doors[j].oth_room == Some(mon_rn) {
					for k in 0..4usize {
						let door_k = mon_room.doors[k];
						if door_k.oth_room == Some(i) {
							monster.set_target_spot(
								door_k.oth_row.expect("oth row"),
								door_k.oth_col.expect("oth col"),
							);
							return;
						}
					}
				}
			}
		}
		/* no place to send monster */
		monster.clear_target_spot();
	} else {
		/* exiting room */
		match mon_room.rn() {
			None => {
				monster.clear_target_spot();
			}
			Some(mon_rn) => {
				if let Some((other_row, other_col)) = get_other_room(mon_rn, row, col, level) {
					monster.set_target_spot(other_row, other_col);
				} else {
					monster.clear_target_spot();
				}
			}
		}
	}
}

fn get_other_room(rn: usize, row: i64, col: i64, level: &Level) -> Option<(i64, i64)> {
	let room = level.rooms[rn];
	let exit_dir = if row == room.top_row {
		Some(Up)
	} else if row == room.bottom_row {
		Some(Down)
	} else if col == room.left_col {
		Some(Left)
	} else if col == room.right_col {
		Some(Right)
	} else {
		None
	};
	if let Some(exit_dir) = exit_dir {
		let exit_dn = exit_dir.to_index();
		if room.doors[exit_dn].oth_room.is_some() {
			let row = room.doors[exit_dn].oth_row.expect("oth row");
			let col = room.doors[exit_dn].oth_col.expect("oth col");
			return Some((row, col));
		}
	}
	return None;
}
