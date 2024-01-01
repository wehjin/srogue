#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;

	fn gr_object() -> *mut object;
}

use ncurses::{addch};
use serde::Serialize;
use crate::objects;
use crate::prelude::*;
use crate::prelude::DoorDirection::{Left, Right};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::SpotFlag::{Door, Floor, Hidden, HorWall, Monster, Object, Stairs, Trap, Tunnel, VertWall};
use crate::room::DoorDirection::{Up, Down};


#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: libc::c_short,
	pub _pad_x: libc::c_short,
	pub _pad_top: libc::c_short,
	pub _pad_left: libc::c_short,
	pub _pad_bottom: libc::c_short,
	pub _pad_right: libc::c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = ncurses::chtype;


#[derive(Copy, Clone, Default, Serialize)]
pub struct dr {
	pub oth_room: Option<i64>,
	pub oth_row: Option<i64>,
	pub oth_col: Option<i64>,
	pub door_row: i64,
	pub door_col: i64,
}

pub type door = dr;

#[derive(Copy, Clone, Eq, PartialEq, Serialize)]
pub enum RoomType {
	Nothing,
	Room,
	Maze,
	DeadEnd,
	Cross,
}

impl RoomType {
	pub fn is_nothing(&self) -> bool {
		self == RoomType::Nothing
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
			DoorDirection::Up => 0,
			DoorDirection::Right => 1,
			DoorDirection::Down => 2,
			DoorDirection::Left => 3,
		}
	}
	pub fn to_inverse(&self) -> DoorDirection {
		match self {
			DoorDirection::Up => Down,
			DoorDirection::Right => Left,
			DoorDirection::Down => Up,
			DoorDirection::Left => Right,
		}
	}
}

#[derive(Copy, Clone, Serialize)]
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

#[no_mangle]
pub unsafe extern "C" fn light_up_room(mut rn: i64) -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	if blind == 0 {
		i = rooms[rn as usize].top_row as libc::c_short;
		while i as i64 <= rooms[rn as usize].bottom_row as i64 {
			j = rooms[rn as usize].left_col as libc::c_short;
			while j as i64 <= rooms[rn as usize].right_col as i64 {
				if dungeon[i as usize][j as usize] as i64
					& 0o2 as i64 as libc::c_ushort as i64 != 0
				{
					let mut monster: *mut object = 0 as *mut object;
					monster = object_at(
						&mut level_monsters,
						i as i64,
						j as i64,
					);
					if !monster.is_null() {
						dungeon[(*monster).row
							as usize][(*monster).col
							as usize] = (dungeon[(*monster).row
							as usize][(*monster).col as usize] as i64
							& !(0o2 as i64 as libc::c_ushort as i64))
							as libc::c_ushort;
						(*monster)
							.d_enchant = get_dungeon_char(
							(*monster).row as i64,
							(*monster).col as i64,
						) as libc::c_short;
						dungeon[(*monster).row
							as usize][(*monster).col
							as usize] = (dungeon[(*monster).row
							as usize][(*monster).col as usize] as i64
							| 0o2 as i64 as libc::c_ushort as i64)
							as libc::c_ushort;
					}
				}
				if ncurses::wmove(ncurses::stdscr(), i as i64, j as i64)
					== -(1)
				{
					-(1);
				} else {
					addch(get_dungeon_char(i as usize, j as usize) as ncurses::chtype);
				};
				j += 1;
				j;
			}
			i += 1;
			i;
		}
		if ncurses::wmove(ncurses::stdscr(), rogue.row as i64, rogue.col as i64)
			== -(1)
		{
			-(1);
		} else {
			addch(rogue.fchar as ncurses::chtype);
		};
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn light_passage(
	mut row: i64,
	mut col: i64,
) -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut i_end: libc::c_short = 0;
	let mut j_end: libc::c_short = 0;
	if blind != 0 {
		return;
	}
	i_end = (if row < 24 as i64 - 2 as i64 {
		1
	} else {
		0 as i64
	}) as libc::c_short;
	j_end = (if col < 80 as i64 - 1 {
		1
	} else {
		0 as i64
	}) as libc::c_short;
	i = (if row > 1 { -(1) } else { 0 as i64 })
		as libc::c_short;
	while i as i64 <= i_end as i64 {
		j = (if col > 0 as i64 { -(1) } else { 0 as i64 })
			as libc::c_short;
		while j as i64 <= j_end as i64 {
			if can_move(row as usize, col as usize, (row + i) as usize, (col + j) as usize) {
				if ncurses::wmove(ncurses::stdscr(), row + i as i64, col + j as i64)
					== -(1)
				{
					-(1);
				} else {
					addch(get_dungeon_char((row + i) as usize, (col + j) as usize) as ncurses::chtype);
				};
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn darken_room(rn: i64) -> i64 {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	i = (rooms[rn as usize].top_row as i64 + 1) as libc::c_short;
	while (i as i64) < rooms[rn as usize].bottom_row as i64 {
		j = (rooms[rn as usize].left_col as i64 + 1)
			as libc::c_short;
		while (j as i64) < rooms[rn as usize].right_col as i64 {
			if blind != 0 {
				if ncurses::wmove(ncurses::stdscr(), i as i32, j as i32)
					== -(1)
				{
					-(1);
				} else {
					addch(' ' as i32 as ncurses::chtype);
				};
			} else if dungeon[i as usize][j as usize] as i64
				& (0o1 as libc::c_ushort as i64
				| 0o4 as i64 as libc::c_ushort as i64) == 0
				&& !(detect_monster as i64 != 0
				&& dungeon[i as usize][j as usize] as i64
				& 0o2 as i64 as libc::c_ushort as i64 != 0)
			{
				if imitating(i, j) == 0 {
					if ncurses::wmove(ncurses::stdscr(), i as i32, j as i32)
						== -(1)
					{
						-(1);
					} else {
						addch(' ' as i32 as ncurses::chtype);
					};
				}
				if dungeon[i as usize][j as usize] as i64
					& 0o400 as i64 as libc::c_ushort as i64 != 0
					&& dungeon[i as usize][j as usize] as i64
					& 0o1000 as i64 as libc::c_ushort as i64 == 0
				{
					if ncurses::wmove(ncurses::stdscr(), i as i32, j as i32)
						== -(1)
					{
						-(1);
					} else {
						addch('^' as i32 as ncurses::chtype);
					};
				}
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn get_dungeon_char(row: i64, col: i64) -> ncurses::chtype {
	let mask = dungeon[row as usize][col as usize];
	if Monster.is_set(mask) {
		return gmc_row_col(row, col);
	}
	if Object.is_set(mask) {
		let obj = objects::object_at(&mut level_objects, row as libc::c_short, col as libc::c_short);
		return get_mask_char((*obj).what_is);
	}
	if SpotFlag::is_any_set(&vec![Tunnel, Stairs, HorWall, VertWall, Floor, Door], mask) {
		if SpotFlag::is_any_set(&vec![Tunnel, Stairs], mask) && !Hidden.is_set(mask) {
			return if Stairs.is_set(mask) {
				ncurses::chtype::from('%')
			} else {
				ncurses::chtype::from('#')
			};
		}
		if HorWall.is_set(mask) {
			return ncurses::chtype::from('-');
		}
		if VertWall.is_set(mask) {
			return ncurses::chtype::from('|');
		}
		if Floor.is_set(mask) {
			if Trap.is_set(mask) {
				if !Hidden.is_set(mask) {
					return ncurses::chtype::from('^');
				}
			}
			return ncurses::chtype::from('.');
		}
		if Door.is_set(mask) {
			return if Hidden.is_set(mask) {
				if (col > 0 && HorWall.is_set(dungeon[row][col - 1]))
					|| (col < (DCOLS - 1) as usize && HorWall.is_set(dungeon[row][col + 1])) {
					ncurses::chtype::from('-')
				} else {
					ncurses::chtype::from('|')
				}
			} else {
				ncurses::chtype::from('+')
			};
		}
	}
	return ncurses::chtype::from(' ');
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
		r = get_rand(1, 24 as i64 - 2 as i64);
		c = get_rand(0 as i64, 80 as i64 - 1);
		let rn = get_room_number(r, c);
		let keep_looking = rn == NO_ROOM
			|| !SpotFlag::is_any_set(&spots, dungeon[r as usize][c as usize])
			|| SpotFlag::are_others_set(&spots, dungeon[r as usize][c as usize])
			|| !(rooms[rn as usize].room_type == RoomType::Room || rooms[rn as usize].room_type == RoomType::Maze)
			|| ((r == rogue.row) && (c == rogue.col));
		if !keep_looking {
			break;
		}
	}
	*row = r;
	*col = c;
}

pub unsafe fn gr_room() -> i64 {
	let mut i: libc::c_short = 0;
	loop {
		i = get_rand(0 as i64, 9 as i64 - 1)
			as libc::c_short;
		if !(rooms[i as usize].room_type as i64
			& (0o2 as i64 as libc::c_ushort as i64
			| 0o4 as i64 as libc::c_ushort as i64) == 0)
		{
			break;
		}
	}
	return i as i64;
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
	N = ((rooms[rn as usize].bottom_row as i64
		- rooms[rn as usize].top_row as i64 - 1)
		* (rooms[rn as usize].right_col as i64
		- rooms[rn as usize].left_col as i64 - 1))
		as libc::c_short;
	n = get_rand(5 as i64, 10 as i64) as libc::c_short;
	if n as i64 > N as i64 {
		n = (N as i64 - 2 as i64) as libc::c_short;
	}
	i = 0;
	while (i as i64) < n as i64 {
		found = 0 as i64 as libc::c_char;
		j = found as libc::c_short;
		while found == 0 && (j as i64) < 250 as i64 {
			row = get_rand(
				rooms[rn as usize].top_row as i64 + 1,
				rooms[rn as usize].bottom_row as i64 - 1,
			) as libc::c_short;
			col = get_rand(
				rooms[rn as usize].left_col as i64 + 1,
				rooms[rn as usize].right_col as i64 - 1,
			) as libc::c_short;
			if dungeon[row as usize][col as usize] as i64
				== 0o100 as i64 as libc::c_ushort as i64
				|| dungeon[row as usize][col as usize] as i64
				== 0o200 as i64 as libc::c_ushort as i64
			{
				found = 1 as libc::c_char;
			}
			j += 1;
			j;
		}
		if found != 0 {
			obj = gr_object();
			place_at(obj, row as i64, col as i64);
			nf += 1;
			nf;
		}
		i += 1;
		i;
	}
	return nf as i64;
}

pub fn get_room_number(row: i64, col: i64) -> i64 {
	unsafe {
		for i in 0i64..MAXROOMS {
			let below_top_wall = row >= rooms[i].top_row;
			let above_bottom_wall = row <= rooms[i].bottom_row;
			let right_of_left_wall = col >= rooms[i].left_col;
			let left_of_right_wall = col <= rooms[i].right_col;
			if below_top_wall && above_bottom_wall && right_of_left_wall && left_of_right_wall {
				return i;
			}
		}
	}
	return NO_ROOM;
}

#[no_mangle]
pub unsafe extern "C" fn is_all_connected() -> i64 {
	let mut i: libc::c_short = 0;
	let mut starting_room: libc::c_short = 0;
	i = 0;
	while (i as i64) < 9 as libc::c_int {
		rooms_visited[i as usize] = 0 as libc::c_int as libc::c_char;
		if rooms[i as usize].room_type as libc::c_int
			& (0o2 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) != 0
		{
			starting_room = i;
		}
		i += 1;
		i;
	}
	visit_rooms(starting_room as libc::c_int);
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 9 as libc::c_int {
		if rooms[i as usize].room_type as libc::c_int
			& (0o2 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) != 0
			&& rooms_visited[i as usize] == 0
		{
			return 0 as libc::c_int;
		}
		i += 1;
		i;
	}
	return 1 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn draw_magic_map() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut ch: libc::c_short = 0;
	let mut och: libc::c_short = 0;
	let mut mask: libc::c_ushort = (0o10 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o20 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o40 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o200 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o400 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o4 as libc::c_int as libc::c_ushort as libc::c_int
		| 0o2 as libc::c_int as libc::c_ushort as libc::c_int) as libc::c_ushort;
	let mut s: libc::c_ushort = 0;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 24 as libc::c_int {
		let mut current_block_24: u64;
		j = 0 as libc::c_int as libc::c_short;
		while (j as libc::c_int) < 80 as libc::c_int {
			s = dungeon[i as usize][j as usize];
			if s as libc::c_int & mask as libc::c_int != 0 {
				ch = (if ncurses::wmove(ncurses::stdscr(), i as libc::c_int, j as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int) as ncurses::chtype
				} else {
					ncurses::winch(ncurses::stdscr())
				}) as libc::c_short;
				if ch as libc::c_int == ' ' as i32
					|| ch as libc::c_int >= 'A' as i32 && ch as libc::c_int <= 'Z' as i32
					|| s as libc::c_int
					& (0o400 as libc::c_int as libc::c_ushort as libc::c_int
					| 0o1000 as libc::c_int as libc::c_ushort as libc::c_int)
					!= 0
				{
					och = ch;
					dungeon[i
						as usize][j
						as usize] = (dungeon[i as usize][j as usize] as libc::c_int
						& !(0o1000 as libc::c_int as libc::c_ushort as libc::c_int))
						as libc::c_ushort;
					if s as libc::c_int
						& 0o10 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						ch = '-' as i32 as libc::c_short;
						current_block_24 = 14576567515993809846;
					} else if s as libc::c_int
						& 0o20 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						ch = '|' as i32 as libc::c_short;
						current_block_24 = 14576567515993809846;
					} else if s as libc::c_int
						& 0o40 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						ch = '+' as i32 as libc::c_short;
						current_block_24 = 14576567515993809846;
					} else if s as libc::c_int
						& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						ch = '^' as i32 as libc::c_short;
						current_block_24 = 14576567515993809846;
					} else if s as libc::c_int
						& 0o4 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						ch = '%' as i32 as libc::c_short;
						current_block_24 = 14576567515993809846;
					} else if s as libc::c_int
						& 0o200 as libc::c_int as libc::c_ushort as libc::c_int != 0
					{
						ch = '#' as i32 as libc::c_short;
						current_block_24 = 14576567515993809846;
					} else {
						current_block_24 = 10680521327981672866;
					}
					match current_block_24 {
						10680521327981672866 => {}
						_ => {
							if s as libc::c_int
								& 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
								|| och as libc::c_int == ' ' as i32
							{
								addch(ch as ncurses::chtype);
							}
							if s as libc::c_int
								& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
							{
								let mut monster: *mut object = 0 as *mut object;
								monster = object_at(&mut level_monsters, i, j);
								if !monster.is_null() {
									(*monster).d_enchant = ch;
								}
							}
						}
					}
				}
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn dr_course(mut monster: *mut object, entering: bool, mut row: i64, mut col: i64) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut k: libc::c_short = 0;
	let mut rn: libc::c_short = 0;
	let mut r: libc::c_short = 0;
	let mut rr: libc::c_short = 0;
	(*monster).row = row;
	(*monster).col = col;
	if mon_sees(monster, rogue.row, rogue.col) != 0 {
		(*monster).trow = -(1);
		return;
	}
	rn = get_room_number(row as libc::c_int, col as libc::c_int) as libc::c_short;
	if entering {
		r = get_rand(0 as libc::c_int, 9 as libc::c_int - 1 as libc::c_int)
			as libc::c_short;
		i = 0 as libc::c_int as libc::c_short;
		while (i as libc::c_int) < 9 as libc::c_int {
			rr = ((r as libc::c_int + i as libc::c_int) % 9 as libc::c_int)
				as libc::c_short;
			if !(rooms[rr as usize].room_type as libc::c_int
				& (0o2 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) == 0
				|| rr as libc::c_int == rn as libc::c_int)
			{
				k = 0 as libc::c_int as libc::c_short;
				while (k as libc::c_int) < 4 as libc::c_int {
					if rooms[rr as usize].doors[k as usize].oth_room as libc::c_int
						== rn as libc::c_int
					{
						(*monster).trow = rooms[rr as usize].doors[k as usize].oth_row;
						(*monster).tcol = rooms[rr as usize].doors[k as usize].oth_col;
						if !((*monster).trow as libc::c_int == row as libc::c_int
							&& (*monster).tcol as libc::c_int == col as libc::c_int)
						{
							return;
						}
					}
					k += 1;
					k;
				}
			}
			i += 1;
			i;
		}
		i = rooms[rn as usize].top_row as libc::c_short;
		while i as libc::c_int <= rooms[rn as usize].bottom_row as libc::c_int {
			j = rooms[rn as usize].left_col as libc::c_short;
			while j as libc::c_int <= rooms[rn as usize].right_col as libc::c_int {
				if i as libc::c_int != (*monster).row as libc::c_int
					&& j as libc::c_int != (*monster).col as libc::c_int
					&& dungeon[i as usize][j as usize] as libc::c_int
					& 0o40 as libc::c_int as libc::c_ushort as libc::c_int != 0
				{
					(*monster).trow = i;
					(*monster).tcol = j;
					return;
				}
				j += 1;
				j;
			}
			i += 1;
			i;
		}
		i = 0 as libc::c_int as libc::c_short;
		while (i as libc::c_int) < 9 as libc::c_int {
			j = 0 as libc::c_int as libc::c_short;
			while (j as libc::c_int) < 4 as libc::c_int {
				if rooms[i as usize].doors[j as usize].oth_room as libc::c_int
					== rn as libc::c_int
				{
					k = 0 as libc::c_int as libc::c_short;
					while (k as libc::c_int) < 4 as libc::c_int {
						if rooms[rn as usize].doors[k as usize].oth_room as libc::c_int
							== i as libc::c_int
						{
							(*monster)
								.trow = rooms[rn as usize].doors[k as usize].oth_row;
							(*monster)
								.tcol = rooms[rn as usize].doors[k as usize].oth_col;
							return;
						}
						k += 1;
						k;
					}
				}
				j += 1;
				j;
			}
			i += 1;
			i;
		}
		(*monster).trow = -(1 as libc::c_int) as libc::c_short;
	} else if get_oth_room(rn as libc::c_int, &mut row, &mut col) == 0 {
		(*monster).trow = -(1 as libc::c_int) as libc::c_short;
	} else {
		(*monster).trow = row;
		(*monster).tcol = col;
	}
	panic!("Reached end of non-void function without returning");
}
