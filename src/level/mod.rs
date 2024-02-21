use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

pub use cells::*;
pub use dungeon::*;
use materials::{CellMaterial, Visibility};
use UpResult::{KeepLevel, WonGame};

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS, MAX_ROOM, MAX_TRAP};
use crate::level::materials::TunnelFixture;
use crate::level::UpResult::UpLevel;
use crate::monster::wake_room;
use crate::objects::put_amulet;
use crate::pack::has_amulet;
use crate::player::{Player, RoomMark};
use crate::player::constants::INIT_HP;
use crate::prelude::*;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::render_system::backend;
use crate::room::{DoorDirection, gr_spot, is_all_connected, Room, RoomBounds, RoomType, visit_room, visit_spot_area};
use crate::room::RoomType::Nothing;
use crate::score::win;
use crate::trap::Trap;

pub mod constants;
mod cells;
mod dungeon;
pub mod materials;

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
		return RoomMark::None;
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
	pub bear_trap: usize,
	pub being_held: bool,
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
			bear_trap: 0,
			being_held: false,
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
		self.bear_trap = 0;
		self.being_held = false;
		self.party_room = None;
	}
}

pub fn make_level(game: &mut GameState) {
	// TODO Sometimes this puts individual tunnel spots near the origin in the wall.  Also there
	// are paths between rooms that are missing a spot.
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
		for j in 0..MAX_ROOM {
			let i = shuffled_rns[j];
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
			fill_out_level(&mut game.level, level_depth);
		}
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
		let skip_walls = (room_index != r1 as usize) && (room_index != r2 as usize) && (room_index != r3 as usize) && rand_percent(40);
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

pub fn connect_rooms(room1: usize, room2: usize, level_depth: isize, level: &mut Level) -> bool {
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
			draw_simple_passage(&spot1, &spot2, dir1, level_depth, level);
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
		self.player.reset_spot();
		self.player.cleaned_up = None;
		self.mash.clear();
		self.ground.clear();
	}
}

pub fn clear_level(game: &mut GameState) {
	game.clear_level();
	backend::erase_screen();
}

pub fn put_door(rn: usize, door_dir: DoorDirection, level_depth: isize, level: &mut Level) -> DungeonSpot {
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

pub fn draw_simple_passage(spot1: &DungeonSpot, spot2: &DungeonSpot, dir: DoorDirection, level_depth: isize, level: &mut Level) {
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
					level.dungeon[row1 as usize][i as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				let mut i = row1;
				let step = if row1 > row2 { -1 } else { 1 };
				while i != row2 {
					level.dungeon[i as usize][middle as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
					i = (i) + step;
				}
				for i in middle..col2 {
					level.dungeon[row2 as usize][i as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
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
					level.dungeon[i as usize][col1 as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				let mut i = col1;
				let step = if col1 > col2 { -1 } else { 1 };
				while i != col2 {
					level.dungeon[middle as usize][i as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
					i = (i) + step;
				}
				for i in middle..row2 {
					level.dungeon[i as usize][col2 as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
				}
				(col1, row1, col2, row2)
			}
		};
	if rand_percent(HIDE_PERCENT) {
		let bounds = RoomBounds { top: row1, bottom: row2, left: col1, right: col2 };
		hide_boxed_passage(bounds, 1, level_depth, level);
	}
}

pub fn same_row(room1: usize, room2: usize) -> bool {
	room1 / 3 == room2 / 3
}

pub fn same_col(room1: usize, room2: usize) -> bool {
	room1 % 3 == room2 % 3
}

pub fn add_mazes(level_depth: isize, level: &mut Level) {
	if level_depth > 1 {
		let random_start = get_rand(0, MAX_ROOM - 1);
		let maze_percent = (level_depth * 5) / 4 + if level_depth > 15 { level_depth } else { 0 };
		for i in 0..MAX_ROOM {
			let rn = (random_start + i) % MAX_ROOM;
			if level.rooms[rn].room_type.is_nothing() {
				if rand_percent(maze_percent as usize) {
					level.rooms[rn].room_type = RoomType::Maze;
					let spot_in_room = level.rooms[rn].to_floor_bounds().to_random_spot();
					let room_bounds = level.rooms[rn].to_wall_bounds();
					make_maze(spot_in_room, &room_bounds, level);
					hide_boxed_passage(room_bounds, get_rand(0, 2), level_depth, level);
				}
			}
		}
	}
}

pub fn fill_out_level(level: &mut Level, level_depth: isize) {
	let shuffled_rns = shuffled_rns();
	level.recursive_deadend = None;
	for i in 0..MAX_ROOM {
		let rn = shuffled_rns[i];
		if level.rooms[rn].room_type.is_nothing()
			|| (level.rooms[rn].room_type.is_cross() && coin_toss()) {
			fill_it(rn, true, level_depth, level);
		}
	}
	if let Some(deadend_rn) = level.recursive_deadend {
		fill_it(deadend_rn, false, level_depth, level);
	}
}

fn fill_it(rn: usize, do_rec_de: bool, level_depth: isize, level: &mut Level) {
	let mut did_this = false;
	let mut srow: i64 = 0;
	let mut scol: i64 = 0;
	for _ in 0..10 {
		srow = get_rand(0, 3);
		scol = get_rand(0, 3);
		level.target_room_offsets.swap(srow as usize, scol as usize);
	}
	let mut rooms_found = 0;
	for i in 0..4 {
		let target_room = rn as isize + level.target_room_offsets[i];
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
		let start_spot = if (!do_rec_de || did_this) || !find_tunnel_in_room(rn, &mut srow, &mut scol, level) {
			level.rooms[rn].center_spot()
		} else {
			DungeonSpot { col: scol, row: srow }
		};
		let end_spot = put_door(target_room, door_dir, level_depth, level);
		rooms_found += 1;
		draw_simple_passage(&start_spot, &end_spot, tunnel_dir, level_depth, level);
		level.rooms[rn].room_type = RoomType::DeadEnd;
		level.dungeon[srow as usize][scol as usize].set_material_remove_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None));
		if (i < 3) && !did_this {
			did_this = true;
			if coin_toss() {
				continue;
			}
		}
		if rooms_found < 2 && do_rec_de {
			recursive_deadend(rn, &start_spot, level_depth, level);
		}
		break;
	}
}

fn recursive_deadend(rn: usize, s_spot: &DungeonSpot, level_depth: isize, level: &mut Level) {
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
		draw_simple_passage(&s_spot, &d_spot, tunnel_dir, level_depth, level);
		level.recursive_deadend = Some(de);
		recursive_deadend(de, &d_spot, level_depth, level);
	}
}

fn get_tunnel_dir(rn: usize, de: usize, level: &Level) -> DoorDirection {
	if same_row(rn, de) {
		if level.rooms[rn].left_col < level.rooms[de].left_col { DoorDirection::Right } else { DoorDirection::Left }
	} else {
		if level.rooms[rn].top_row < level.rooms[de].top_row { DoorDirection::Down } else { DoorDirection::Up }
	}
}

fn find_tunnel_in_room(rn: usize, row: &mut i64, col: &mut i64, level: &Level) -> bool {
	let bounds = level.rooms[rn].to_wall_bounds();
	for room_row in bounds.rows() {
		for room_col in bounds.cols() {
			if level.dungeon[room_row as usize][room_col as usize].is_any_tunnel() {
				*row = room_row;
				*col = room_col;
				return true;
			}
		}
	}
	return false;
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

fn hide_boxed_passage(bounds: RoomBounds, n: i64, level_depth: isize, level: &mut Level) {
	if level_depth <= 2 {
		return;
	}
	let RoomBounds { top: row1, left: col1, bottom: row2, right: col2 } = bounds;
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

pub fn put_player(avoid_room: RoomMark, game: &mut GameState) {
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
			wake_room(cur_room, true, game.player.rogue.row, game.player.rogue.col, game);
		}
	}
	if let Some(msg) = &game.level.new_level_message {
		game.dialog.message(msg, 0);
	}
	game.level.new_level_message = None;
	game.render_spot(game.player.to_spot());
}

pub fn drop_check(game: &mut GameState) -> bool {
	if game.player.wizard {
		return true;
	}
	if game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_stairs() {
		if game.player.levitate.is_active() {
			game.dialog.message("you're floating in the air!", 0);
			return false;
		}
		return true;
	}
	game.dialog.message("I see no way down", 0);
	return false;
}

pub enum UpResult {
	KeepLevel,
	UpLevel,
	WonGame,
}

pub fn check_up(game: &mut GameState) -> UpResult {
	if !game.player.wizard {
		if !game.level.dungeon[game.player.rogue.row as usize][game.player.rogue.col as usize].is_stairs() {
			game.dialog.message("I see no way up", 0);
			return KeepLevel;
		}
		if !has_amulet(&game.player) {
			game.dialog.message("Your way is magically blocked", 0);
			return KeepLevel;
		}
	}
	game.level.new_level_message = Some("you feel a wrenching sensation in your gut".to_string());
	if game.player.cur_depth == 1 {
		win(game);
		WonGame
	} else {
		game.player.ascend();
		UpLevel
	}
}

pub fn add_exp(e: isize, promotion: bool, game: &mut GameState) {
	game.player.rogue.exp_points += e;
	if game.player.rogue.exp_points >= LEVEL_POINTS[(game.player.rogue.exp - 1) as usize] {
		let new_exp = get_exp_level(game.player.rogue.exp_points);
		if game.player.rogue.exp_points > MAX_EXP {
			game.player.rogue.exp_points = MAX_EXP + 1;
		}
		for i in (game.player.rogue.exp + 1)..=new_exp {
			let msg = format!("welcome to level {}", i);
			game.dialog.message(&msg, 0);
			if promotion {
				let hp = hp_raise(&mut game.player);
				game.player.rogue.hp_current += hp;
				game.player.rogue.hp_max += hp;
			}
			game.player.rogue.exp = i;
			game.stats_changed = true;
		}
	} else {
		game.stats_changed = true;
	}
}

pub fn get_exp_level(e: isize) -> isize {
	for i in 0..(MAX_EXP_LEVEL - 1) {
		if LEVEL_POINTS[i] > e {
			return (i + 1) as isize;
		}
	}
	return MAX_EXP_LEVEL as isize;
}

pub fn hp_raise(player: &Player) -> isize {
	if player.wizard {
		10
	} else {
		get_rand(3, 10) as isize
	}
}

pub fn show_average_hp(game: &mut GameState) {
	let player = &game.player;
	let (real_average, effective_average) = if player.rogue.exp == 1 {
		(0.0, 0.0)
	} else {
		let real = (player.rogue.hp_max - player.extra_hp - INIT_HP + player.less_hp) as f32 / (player.rogue.exp - 1) as f32;
		let average = (player.rogue.hp_max - INIT_HP) as f32 / (player.rogue.exp - 1) as f32;
		(real, average)
	};
	let msg = format!(
		"R-Hp: {:.2}, E-Hp: {:.2} (!: {}, V: {})",
		real_average,
		effective_average,
		player.extra_hp,
		player.less_hp
	);
	game.dialog.message(&msg, 0);
}
