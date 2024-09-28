use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::init::{Dungeon, GameState};
use crate::level::constants::{DCOLS, DROWS, MAX_ROOM, MAX_TRAP};
use crate::level::materials::TunnelFixture;
use crate::objects::put_amulet;
use crate::pack::has_amulet;
use crate::player::constants::INIT_HP;
use crate::player::RoomMark;
use crate::prelude::*;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::render_system::backend;
use crate::resources::avatar::Avatar;
use crate::resources::level::wake::wake_room_legacy;
use crate::room::RoomType::Nothing;
use crate::room::{gr_spot, is_all_connected, visit_room, visit_spot_area, DoorDirection, Room, RoomBounds, RoomType};
use crate::trap::Trap;
pub use cells::*;
pub use dungeon::*;
use materials::{CellMaterial, Visibility};

pub mod constants;
mod cells;
mod dungeon;
pub mod materials;

pub const LEVEL_POINTS: [usize; MAX_EXP_LEVEL] = [
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

impl Level {
	pub fn room_at_spot(&self, spot: DungeonSpot) -> RoomMark {
		self.room(spot.row, spot.col)
	}
	pub fn expect_room(&self, row: i64, col: i64) -> usize {
		self.room(row, col).rn().expect("not an area")
	}
	pub fn room(&self, row: i64, col: i64) -> RoomMark {
		for i in 0..MAX_ROOM {
			if self.rooms[i].contains_spot(row, col) {
				return RoomMark::Cavern(i);
			}
		}
		RoomMark::None
	}
	pub fn cell(&self, spot: DungeonSpot) -> &DungeonCell {
		&self.dungeon[spot.row as usize][spot.col as usize]
	}
	pub fn cell_mut(&mut self, spot: DungeonSpot) -> &mut DungeonCell {
		&mut self.dungeon[spot.row as usize][spot.col as usize]
	}
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Level {
	pub target_room_offsets: [isize; 4],
	pub recursive_deadend: Option<usize>,
	pub rooms: [Room; MAX_ROOM],
	pub traps: [Trap; MAX_TRAP],
	pub dungeon: [DungeonRow; DROWS],
	pub see_invisible: bool,
	pub detect_monster: bool,
	pub party_room: Option<usize>,
	pub new_level_message: Option<String>,
	pub trap_door: bool,
}

impl Level {
	pub fn new() -> Self {
		Level {
			target_room_offsets: [-1, 1, 3, -3],
			recursive_deadend: None,
			rooms: [Room::default(); MAX_ROOM],
			traps: [Trap::default(); MAX_TRAP],
			dungeon: [DungeonRow::default(); DROWS],
			see_invisible: false,
			detect_monster: false,
			party_room: None,
			new_level_message: None,
			trap_door: false,
		}
	}
	pub fn clear(&mut self) {
		for rn in 0..MAX_ROOM {
			self.rooms[rn].clear();
		}
		for tn in 0..MAX_TRAP {
			self.traps[tn].clear();
		}
		for row in 0..DROWS {
			for col in 0..DCOLS {
				self.dungeon[row][col].reset_to_nothing();
			}
		}
		self.see_invisible = false;
		self.detect_monster = false;
		self.party_room = None;
	}
}

pub fn make_level(game: &mut GameState) {
	let (must_exist1, must_exist2, must_exist3) = match get_rand(0, 5) {
		0 => (0, 1, 2),
		1 => (3, 4, 5),
		2 => (6, 7, 8),
		3 => (0, 3, 6),
		4 => (1, 4, 7),
		5 => (2, 5, 8),
		_ => unreachable!("0 <= rand <= 5")
	};
	let level_depth = game.player.cur_depth;
	let big_room = level_depth == game.player.party_counter && rand_percent(1);
	if big_room {
		make_room(BIG_ROOM, 0, 0, 0, &mut game.level);
	} else {
		for rn in 0..MAX_ROOM {
			make_room(rn, must_exist1, must_exist2, must_exist3, &mut game.level);
		}
	}
	if !big_room {
		add_mazes(level_depth, &mut game.level);
		let shuffled_rns = shuffled_rns();
		for i in shuffled_rns {
			if i < (MAX_ROOM - 1) {
				connect_rooms(i, i + 1, level_depth, &mut game.level);
			}
			if i < (MAX_ROOM - 3) {
				connect_rooms(i, i + 3, level_depth, &mut game.level);
			}
			if i < (MAX_ROOM - 2) {
				if game.level.rooms[i + 1].room_type.is_nothing() {
					if connect_rooms(i, i + 2, level_depth, &mut game.level) {
						game.level.rooms[i + 1].room_type = RoomType::Cross;
					}
				}
			}
			if i < (MAX_ROOM - 6) {
				if game.level.rooms[i + 3].room_type.is_nothing() {
					if connect_rooms(i, i + 6, level_depth, &mut game.level) {
						game.level.rooms[i + 3].room_type = RoomType::Cross;
					}
				}
			}
			if is_all_connected(&game.level.rooms) {
				break;
			}
		}
		fill_out_level(&mut game.level, level_depth);
	}
	if !has_amulet(&game.player) && level_depth >= AMULET_LEVEL {
		put_amulet(game);
	}
}

pub fn make_room(rn: usize, r1: usize, r2: usize, r3: usize, level: &mut Level) {
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
		let skip_walls = (room_index != r1) && (room_index != r2) && (room_index != r3) && rand_percent(40);
		(left, right, top, bottom, !skip_walls)
	} else {
		(left, right, top, bottom, true)
	};
	let room_bounds = RoomBounds { top, right, bottom, left };
	if fill_dungeon {
		level.rooms[room_index].room_type = RoomType::Room;
		for row in room_bounds.rows() {
			for col in room_bounds.cols() {
				let spot = &DungeonSpot { row, col };
				let cell_mat = room_bounds.cell_material_for_spot(&spot);
				level.dungeon[spot.row as usize][spot.col as usize].set_material_remove_others(cell_mat);
			}
		}
	}
	level.rooms[rn].set_bounds(&room_bounds);
}

pub fn connect_rooms(room1: usize, room2: usize, level_depth: usize, level: &mut Level) -> bool {
	if !level.rooms[room1].room_type.is_type(&vec![RoomType::Room, RoomType::Maze])
		|| !level.rooms[room2].room_type.is_type(&vec![RoomType::Room, RoomType::Maze]) {
		return false;
	}
	if let Some(dir1) = DoorDirection::from_room_to_room(room1, room2, level) {
		let dir2 = dir1.inverse();
		let spot1 = put_door(room1, dir1, level_depth, level);
		let spot2 = put_door(room2, dir2, level_depth, level);
		let mut draw_again = true;
		while draw_again {
			draw_simple_passage(spot1, spot2, dir1, level_depth, level);
			draw_again = rand_percent(4);
		}
		level.rooms[room1].doors[dir1.to_index()].set_others(room2, &spot2);
		level.rooms[room2].doors[dir2.to_index()].set_others(room1, &spot1);
		true
	} else {
		false
	}
}

impl GameState {
	pub fn clear_level(&mut self) {
		self.level.clear();
		self.player.reset_for_new_level();
		self.diary.cleaned_up = None;
		self.mash.clear();
		self.ground.clear();
		backend::erase_screen();
	}
}

pub fn put_door(rn: usize, door_dir: DoorDirection, level_depth: usize, level: &mut Level) -> DungeonSpot {
	let room = &mut level.rooms[rn];
	let wall_width = if RoomType::Maze == room.room_type { 0 } else { 1 };
	let door_spot = match door_dir {
		DoorDirection::Up | DoorDirection::Down => {
			let row = if door_dir == DoorDirection::Up { room.top_row } else { room.bottom_row };
			let mut col;
			loop {
				col = get_rand(room.left_col + wall_width, room.right_col - wall_width);
				let cell = level.dungeon[row as usize][col as usize];
				if cell.is_any_tunnel() || cell.is_horizontal_wall() {
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
				let cell = level.dungeon[row as usize][col as usize];
				if cell.is_any_tunnel() || cell.is_vertical_wall() {
					break;
				}
			}
			DungeonSpot { row, col }
		}
	};
	if room.room_type == RoomType::Room {
		let cell = &mut level.dungeon[door_spot.row as usize][door_spot.col as usize];
		cell.set_material_remove_others(CellMaterial::Door(door_dir, Visibility::Visible));
	}
	if (level_depth > 2) && rand_percent(HIDE_PERCENT) {
		level.dungeon[door_spot.row as usize][door_spot.col as usize].set_hidden();
	}
	let door_index = door_dir.to_index();
	room.doors[door_index].door_row = door_spot.row;
	room.doors[door_index].door_col = door_spot.col;
	door_spot
}

pub fn draw_simple_passage(spot1: DungeonSpot, spot2: DungeonSpot, dir: DoorDirection, level_depth: usize, level: &mut Level) {
	let (spot1, spot2) =
		match dir {
			DoorDirection::Left | DoorDirection::Right => {
				let (spot1, spot2) = if spot1.col <= spot2.col { (spot1, spot2) } else { (spot2, spot1) };
				let middle_col = get_rand(spot1.col + 1, spot2.col - 1);
				for i in (spot1.col + 1)..=middle_col {
					level.dungeon[spot1.row as usize][i as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				let middle_rows = if spot1.row <= spot2.row { spot1.row..=spot2.row } else { spot2.row..=spot1.row };
				for i in middle_rows {
					level.dungeon[i as usize][middle_col as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				for i in middle_col..spot2.col {
					level.dungeon[spot2.row as usize][i as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				(spot1, spot2)
			}
			DoorDirection::Up | DoorDirection::Down => {
				let (spot1, spot2) = if spot1.row <= spot2.row { (spot1, spot2) } else { (spot2, spot1) };
				let middle_row = get_rand(spot1.row + 1, spot2.row - 1);
				for i in (spot1.row + 1)..middle_row {
					level.dungeon[i as usize][spot1.col as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				let middle_cols = if spot1.col <= spot2.col { spot1.col..=spot2.col } else { spot2.col..=spot1.col };
				for i in middle_cols {
					level.dungeon[middle_row as usize][i as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				for i in middle_row..spot2.row {
					level.dungeon[i as usize][spot2.col as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				(spot1, spot2)
			}
		};
	if rand_percent(HIDE_PERCENT) {
		hide_boxed_passage(spot1.row, spot1.col, spot2.row, spot2.col, 1, level_depth, level);
	}
}

pub fn same_row(room1: usize, room2: usize) -> bool {
	room1 / 3 == room2 / 3
}

pub fn same_col(room1: usize, room2: usize) -> bool {
	room1 % 3 == room2 % 3
}

pub fn add_mazes(level_depth: usize, level: &mut Level) {
	if level_depth > 1 {
		let random_start = get_rand(0, MAX_ROOM - 1);
		let maze_percent = (level_depth * 5) / 4 + if level_depth > 15 { level_depth } else { 0 };
		for i in 0..MAX_ROOM {
			let rn = (random_start + i) % MAX_ROOM;
			if level.rooms[rn].room_type.is_nothing() {
				if rand_percent(maze_percent) {
					level.rooms[rn].room_type = RoomType::Maze;
					let spot_in_room = level.rooms[rn].to_floor_bounds().to_random_spot(&mut thread_rng());
					let room_bounds = level.rooms[rn].to_wall_bounds();
					make_maze(spot_in_room, &room_bounds, level);
					hide_boxed_passage(
						room_bounds.top, room_bounds.left, room_bounds.bottom, room_bounds.right,
						get_rand(0, 2), level_depth, level,
					);
				}
			}
		}
	}
}

pub fn fill_out_level(level: &mut Level, level_depth: usize) {
	let shuffled_rns = shuffled_rns();
	level.recursive_deadend = None;
	for rn in shuffled_rns {
		if level.rooms[rn].room_type.is_nothing()
			|| (level.rooms[rn].room_type.is_cross() && coin_toss()) {
			fill_it(rn, true, level_depth, level);
		}
	}
	if let Some(deadend_rn) = level.recursive_deadend {
		fill_it(deadend_rn, false, level_depth, level);
	}
}

fn fill_it(rn: usize, do_rec_de: bool, level_depth: usize, level: &mut Level) {
	let mut did_this = false;
	level.target_room_offsets.shuffle(&mut thread_rng());
	let mut rooms_found = 0;
	for i in 0..4 {
		let target_rn = {
			let mut rn = (rn + MAX_ROOM) as isize;
			rn += level.target_room_offsets[i];
			rn as usize % MAX_ROOM
		};
		let target = &level.rooms[target_rn];
		if !(same_row(rn, target_rn) || same_col(rn, target_rn))
			|| !target.room_type.is_type(&vec![RoomType::Room, RoomType::Maze]) {
			continue;
		}
		let tunnel_dir = get_tunnel_dir(rn, target_rn, level);
		let door_dir = tunnel_dir.inverse();
		if target.doors[door_dir.to_index()].oth_room.is_some() {
			continue;
		}

		let start_spot = if !do_rec_de || did_this {
			level.rooms[rn].center_spot()
		} else if let Some(spot) = find_tunnel_in_room(rn, level) {
			spot
		} else {
			level.rooms[rn].center_spot()
		};
		let end_spot = put_door(target_rn, door_dir, level_depth, level);
		rooms_found += 1;
		draw_simple_passage(start_spot, end_spot, tunnel_dir, level_depth, level);
		level.rooms[rn].room_type = RoomType::DeadEnd;
		level.cell_mut(start_spot).set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
		if (i < 3) && !did_this {
			did_this = true;
			if coin_toss() {
				continue;
			}
		}
		if rooms_found < 2 && do_rec_de {
			recursive_deadend(rn, start_spot, level_depth, level);
		}
		break;
	}
}

fn recursive_deadend(rn: usize, s_spot: DungeonSpot, level_depth: usize, level: &mut Level) {
	level.rooms[rn].room_type = RoomType::DeadEnd;
	level.dungeon[s_spot.row as usize][s_spot.col as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));

	for i in 0..4 {
		let de: isize = rn as isize + level.target_room_offsets[i];
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
		draw_simple_passage(s_spot, d_spot, tunnel_dir, level_depth, level);
		level.recursive_deadend = Some(de);
		recursive_deadend(de, d_spot, level_depth, level);
	}
}

fn get_tunnel_dir(from_rn: usize, to_rn: usize, level: &Level) -> DoorDirection {
	if same_row(from_rn, to_rn) {
		if level.rooms[from_rn].left_col < level.rooms[to_rn].left_col { DoorDirection::Right } else { DoorDirection::Left }
	} else {
		if level.rooms[from_rn].top_row < level.rooms[to_rn].top_row { DoorDirection::Down } else { DoorDirection::Up }
	}
}

fn find_tunnel_in_room(rn: usize, level: &Level) -> Option<DungeonSpot> {
	let bounds = level.rooms[rn].to_wall_bounds();
	for room_row in bounds.rows() {
		for room_col in bounds.cols() {
			if level.dungeon[room_row as usize][room_col as usize].is_any_tunnel() {
				return Some((room_row, room_col).into());
			}
		}
	}
	None
}

fn make_maze(spot: DungeonSpot, bounds: &RoomBounds, level: &mut Level) {
	let mut dirs: [DoorDirection; 4] = [DoorDirection::Up, DoorDirection::Down, DoorDirection::Left, DoorDirection::Right];
	if rand_percent(33) {
		dirs.shuffle(&mut thread_rng());
	}
	level.cell_mut(spot).set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
	for dir in dirs {
		let (delta_row, delta_col) = dir.as_delta_row_col();
		if let Some(new_spot) = check_spot_for_maze(delta_row, delta_col, spot, &bounds, level) {
			make_maze(new_spot, bounds, level);
		}
	}
}

fn check_spot_for_maze(delta_row: i64, delta_col: i64, spot: DungeonSpot, bounds: &RoomBounds, level: &Level) -> Option<DungeonSpot> {
	let new: DungeonSpot = (spot.row + delta_row, spot.col + delta_col).into();
	let new_new: DungeonSpot = (spot.row + delta_row * 2, spot.col + delta_col * 2).into();
	let (new_a, new_b) =
		if delta_row != 0 {
			let a: DungeonSpot = (new.row, new.col - 1).into();
			let b: DungeonSpot = (new.row, new.col + 1).into();
			(a, b)
		} else {
			let a: DungeonSpot = (new.row - 1, new.col).into();
			let b: DungeonSpot = (new.row + 1, new.col).into();
			(a, b)
		};
	if !new_new.is_out_of_bounds() &&
		new.is_within_bounds(bounds) &&
		level.cell(new).is_not_tunnel() &&
		level.cell(new_a).is_not_tunnel() &&
		level.cell(new_b).is_not_tunnel() &&
		level.cell(new_new).is_not_tunnel() {
		Some(new)
	} else {
		None
	}
}

fn hide_boxed_passage(row1: i64, col1: i64, row2: i64, col2: i64, n: i64, level_depth: usize, level: &mut Level) {
	if level_depth <= 2 {
		return;
	}
	let (row1, row2) = if row1 > row2 { (row2, row1) } else { (row1, row2) };
	let (col1, col2) = if col1 > col2 { (col2, col1) } else { (col1, col2) };
	let h = row2 - row1;
	let w = col2 - col1;
	if (w >= 5) || (h >= 5) {
		let row_cut = if h >= 2 { 1 } else { 0 };
		let col_cut = if w >= 2 { 1 } else { 0 };
		for _ in 0..n {
			for _ in 0..10 {
				let row = get_rand(row1 + row_cut, row2 - row_cut) as usize;
				let col = get_rand(col1 + col_cut, col2 - col_cut) as usize;
				if level.dungeon[row][col].is_any_tunnel() {
					level.dungeon[row][col].set_hidden();
					break;
				}
			}
		}
	}
}

pub fn put_player_legacy(avoid_room: RoomMark, game: &mut GameState) {
	{
		let mut spot = DungeonSpot::default();
		let mut to_room = avoid_room;
		for _misses in 0..2 {
			if to_room != avoid_room {
				break;
			}
			spot = gr_spot(
				|cell| cell.is_any_floor() || cell.is_any_tunnel() || cell.has_object() || cell.is_stairs(),
				&game.player,
				&game.level,
			);
			to_room = game.level.room(spot.row, spot.col);
		}
		game.player.rogue.row = spot.row;
		game.player.rogue.col = spot.col;
		game.player.cur_room = if game.cell_at(spot).is_any_tunnel() {
			RoomMark::Passage
		} else {
			to_room
		};
	}
	match game.player.cur_room {
		RoomMark::None => {}
		RoomMark::Passage => {
			visit_spot_area(game.player.rogue.row, game.player.rogue.col, game);
		}
		RoomMark::Cavern(cur_room) => {
			visit_room(cur_room, game);
			wake_room_legacy(cur_room, true, game.player.rogue.row, game.player.rogue.col, game);
		}
	}
	if let Some(msg) = &game.level.new_level_message {
		game.diary.add_entry(msg);
	}
	game.level.new_level_message = None;
	game.render_spot(game.player.to_spot());
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct RogueExp {
	pub points: usize,
	pub level: usize,
}
impl RogueExp {
	pub fn new() -> Self {
		Self { points: 0, level: 1 }
	}
	pub fn set_level(&mut self, value: usize) {
		self.level = value;
	}
	pub fn raise_level(&mut self) {
		self.points = LEVEL_POINTS[self.level - 1];
	}
	pub fn add_points_take_old(&mut self, points: usize) -> Self {
		let old = self.to_owned();
		let new_level;
		let mut new_points = old.points + points;
		if new_points >= LEVEL_POINTS[old.level - 1] {
			new_level = get_exp_level(new_points);
			new_points = new_points.min((MAX_EXP + 1) as usize);
		} else {
			new_level = old.level;
		}
		self.points = new_points;
		self.level = new_level;
		old
	}
}


pub fn add_exp(e: usize, promotion: bool, game: &mut impl Dungeon, rng: &mut impl Rng) {
	let old = game.as_fighter_mut().exp.add_points_take_old(e);
	let new = game.as_fighter().exp;
	if new.level > old.level {
		for i in old.level + 1..=new.level {
			let msg = format!("welcome to level {}", i);
			game.as_diary_mut().add_entry(&msg);

			if promotion {
				let hp = hp_raise(game.wizard(), rng);
				let fighter = game.as_fighter_mut();
				fighter.hp_current += hp;
				fighter.hp_max += hp;
			}
			// TODO the purpose of setting the level to i is to print the correct exp level
			// every time we print the welcome message.
			game.as_fighter_mut().exp.set_level(i);
			game.as_diary_mut().set_stats_changed(true);
		}
	} else {
		game.as_diary_mut().set_stats_changed(true);
	}
}

pub fn get_exp_level(level: usize) -> usize {
	for i in 0..(MAX_EXP_LEVEL - 1) {
		if LEVEL_POINTS[i] > level {
			return i + 1;
		}
	}
	MAX_EXP_LEVEL
}

pub fn hp_raise(wizard: bool, rng: &mut impl Rng) -> isize {
	match wizard {
		true => 10,
		false => rng.gen_range(3..=10)
	}
}

pub fn show_average_hp(game: &mut GameState) {
	let player = &game.player;
	let exp_level = game.as_fighter().exp.level;
	let (real_average, effective_average) = if exp_level == 1 {
		(0.0, 0.0)
	} else {
		let real = (player.rogue.hp_max - player.extra_hp - INIT_HP + player.less_hp) as f32 / (exp_level - 1) as f32;
		let average = (player.rogue.hp_max - INIT_HP) as f32 / (exp_level - 1) as f32;
		(real, average)
	};
	let msg = format!(
		"R-Hp: {:.2}, E-Hp: {:.2} (!: {}, V: {})",
		real_average,
		effective_average,
		player.extra_hp,
		player.less_hp
	);
	game.diary.add_entry(&msg);
}
