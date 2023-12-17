#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	static mut level_monsters: object;
	fn mon_sees() -> libc::c_char;
	fn gr_object() -> *mut object;
	fn object_at() -> *mut object;
	static mut blind: libc::c_short;
	static mut detect_monster: libc::c_char;
}

use crate::prelude::*;
use crate::prelude::DoorDirection::{Left, Right};
use crate::room::DoorDirection::{Up, Down};

pub type chtype = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: libc::c_short,
	pub _curx: libc::c_short,
	pub _maxy: libc::c_short,
	pub _maxx: libc::c_short,
	pub _begy: libc::c_short,
	pub _begx: libc::c_short,
	pub _flags: libc::c_short,
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
	pub _regtop: libc::c_short,
	pub _regbottom: libc::c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: libc::c_short,
}

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
pub type attr_t = chtype;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: libc::c_short,
	pub hp_max: libc::c_short,
	pub str_current: libc::c_short,
	pub str_max: libc::c_short,
	pub pack: object,
	pub gold: libc::c_long,
	pub exp: libc::c_short,
	pub exp_points: libc::c_long,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub fchar: libc::c_short,
	pub moves_left: libc::c_short,
}

pub type fighter = fight;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dr {
	pub oth_room: Option<usize>,
	pub oth_row: Option<usize>,
	pub oth_col: Option<usize>,
	pub door_row: libc::c_short,
	pub door_col: libc::c_short,
}

pub type door = dr;

#[derive(Copy, Clone, Eq, PartialEq)]
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct rm {
	pub bottom_row: libc::c_int,
	pub right_col: libc::c_int,
	pub left_col: libc::c_int,
	pub top_row: libc::c_int,
	pub doors: [door; 4],
	pub room_type: RoomType,
}

pub type room = rm;

#[no_mangle]
pub static mut rooms: [room; 9] = [rm {
	bottom_row: 0,
	right_col: 0,
	left_col: 0,
	top_row: 0,
	doors: [dr {
		oth_room: None,
		oth_row: None,
		oth_col: None,
		door_row: 0,
		door_col: 0,
	}; 4],
	room_type: RoomType::Nothing,
}; 9];
#[no_mangle]
pub static mut rooms_visited: [libc::c_char; 9] = [0; 9];

#[no_mangle]
pub unsafe extern "C" fn light_up_room(mut rn: libc::c_int) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	if blind == 0 {
		i = rooms[rn as usize].top_row as libc::c_short;
		while i as libc::c_int <= rooms[rn as usize].bottom_row as libc::c_int {
			j = rooms[rn as usize].left_col as libc::c_short;
			while j as libc::c_int <= rooms[rn as usize].right_col as libc::c_int {
				if dungeon[i as usize][j as usize] as libc::c_int
					& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
				{
					let mut monster: *mut object = 0 as *mut object;
					monster = object_at(
						&mut level_monsters,
						i as libc::c_int,
						j as libc::c_int,
					);
					if !monster.is_null() {
						dungeon[(*monster).row
							as usize][(*monster).col
							as usize] = (dungeon[(*monster).row
							as usize][(*monster).col as usize] as libc::c_int
							& !(0o2 as libc::c_int as libc::c_ushort as libc::c_int))
							as libc::c_ushort;
						(*monster)
							.d_enchant = get_dungeon_char(
							(*monster).row as libc::c_int,
							(*monster).col as libc::c_int,
						) as libc::c_short;
						dungeon[(*monster).row
							as usize][(*monster).col
							as usize] = (dungeon[(*monster).row
							as usize][(*monster).col as usize] as libc::c_int
							| 0o2 as libc::c_int as libc::c_ushort as libc::c_int)
							as libc::c_ushort;
					}
				}
				if wmove(stdscr, i as libc::c_int, j as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					waddch(
						stdscr,
						get_dungeon_char(i as libc::c_int, j as libc::c_int) as chtype,
					);
				};
				j += 1;
				j;
			}
			i += 1;
			i;
		}
		if wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int);
		} else {
			waddch(stdscr, rogue.fchar as chtype);
		};
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn light_passage(
	mut row: libc::c_int,
	mut col: libc::c_int,
) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut i_end: libc::c_short = 0;
	let mut j_end: libc::c_short = 0;
	if blind != 0 {
		return;
	}
	i_end = (if row < 24 as libc::c_int - 2 as libc::c_int {
		1 as libc::c_int
	} else {
		0 as libc::c_int
	}) as libc::c_short;
	j_end = (if col < 80 as libc::c_int - 1 as libc::c_int {
		1 as libc::c_int
	} else {
		0 as libc::c_int
	}) as libc::c_short;
	i = (if row > 1 as libc::c_int { -(1 as libc::c_int) } else { 0 as libc::c_int })
		as libc::c_short;
	while i as libc::c_int <= i_end as libc::c_int {
		j = (if col > 0 as libc::c_int { -(1 as libc::c_int) } else { 0 as libc::c_int })
			as libc::c_short;
		while j as libc::c_int <= j_end as libc::c_int {
			if can_move(row, col, row + i as libc::c_int, col + j as libc::c_int) != 0 {
				if wmove(stdscr, row + i as libc::c_int, col + j as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					waddch(
						stdscr,
						get_dungeon_char(row + i as libc::c_int, col + j as libc::c_int)
							as chtype,
					);
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
pub unsafe extern "C" fn darken_room(mut rn: libc::c_short) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	i = (rooms[rn as usize].top_row as libc::c_int + 1 as libc::c_int) as libc::c_short;
	while (i as libc::c_int) < rooms[rn as usize].bottom_row as libc::c_int {
		j = (rooms[rn as usize].left_col as libc::c_int + 1 as libc::c_int)
			as libc::c_short;
		while (j as libc::c_int) < rooms[rn as usize].right_col as libc::c_int {
			if blind != 0 {
				if wmove(stdscr, i as libc::c_int, j as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					waddch(stdscr, ' ' as i32 as chtype);
				};
			} else if dungeon[i as usize][j as usize] as libc::c_int
				& (0o1 as libc::c_int as libc::c_ushort as libc::c_int
				| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) == 0
				&& !(detect_monster as libc::c_int != 0
				&& dungeon[i as usize][j as usize] as libc::c_int
				& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0)
			{
				if imitating(i as libc::c_int, j as libc::c_int) == 0 {
					if wmove(stdscr, i as libc::c_int, j as libc::c_int)
						== -(1 as libc::c_int)
					{
						-(1 as libc::c_int);
					} else {
						waddch(stdscr, ' ' as i32 as chtype);
					};
				}
				if dungeon[i as usize][j as usize] as libc::c_int
					& 0o400 as libc::c_int as libc::c_ushort as libc::c_int != 0
					&& dungeon[i as usize][j as usize] as libc::c_int
					& 0o1000 as libc::c_int as libc::c_ushort as libc::c_int == 0
				{
					if wmove(stdscr, i as libc::c_int, j as libc::c_int)
						== -(1 as libc::c_int)
					{
						-(1 as libc::c_int);
					} else {
						waddch(stdscr, '^' as i32 as chtype);
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

#[no_mangle]
pub unsafe extern "C" fn gr_row_col(
	mut row: *mut libc::c_short,
	mut col: *mut libc::c_short,
	mut mask: libc::c_ushort,
) -> libc::c_int {
	let mut rn: libc::c_short = 0;
	let mut r: libc::c_short = 0;
	let mut c: libc::c_short = 0;
	loop {
		r = get_rand(1 as libc::c_int, 24 as libc::c_int - 2 as libc::c_int)
			as libc::c_short;
		c = get_rand(0 as libc::c_int, 80 as libc::c_int - 1 as libc::c_int)
			as libc::c_short;
		rn = get_room_number(r as libc::c_int, c as libc::c_int) as libc::c_short;
		if !(rn as libc::c_int == -(1 as libc::c_int)
			|| dungeon[r as usize][c as usize] as libc::c_int & mask as libc::c_int == 0
			|| dungeon[r as usize][c as usize] as libc::c_int & !(mask as libc::c_int)
			!= 0
			|| rooms[rn as usize].room_type as libc::c_int
			& (0o2 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) == 0
			|| r as libc::c_int == rogue.row as libc::c_int
			&& c as libc::c_int == rogue.col as libc::c_int)
		{
			break;
		}
	}
	*row = r;
	*col = c;
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn gr_room() -> libc::c_int {
	let mut i: libc::c_short = 0;
	loop {
		i = get_rand(0 as libc::c_int, 9 as libc::c_int - 1 as libc::c_int)
			as libc::c_short;
		if !(rooms[i as usize].room_type as libc::c_int
			& (0o2 as libc::c_int as libc::c_ushort as libc::c_int
			| 0o4 as libc::c_int as libc::c_ushort as libc::c_int) == 0)
		{
			break;
		}
	}
	return i as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn party_objects(mut rn: libc::c_int) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut nf: libc::c_short = 0 as libc::c_int as libc::c_short;
	let mut obj: *mut object = 0 as *mut object;
	let mut n: libc::c_short = 0;
	let mut N: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut found: libc::c_char = 0;
	N = ((rooms[rn as usize].bottom_row as libc::c_int
		- rooms[rn as usize].top_row as libc::c_int - 1 as libc::c_int)
		* (rooms[rn as usize].right_col as libc::c_int
		- rooms[rn as usize].left_col as libc::c_int - 1 as libc::c_int))
		as libc::c_short;
	n = get_rand(5 as libc::c_int, 10 as libc::c_int) as libc::c_short;
	if n as libc::c_int > N as libc::c_int {
		n = (N as libc::c_int - 2 as libc::c_int) as libc::c_short;
	}
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < n as libc::c_int {
		found = 0 as libc::c_int as libc::c_char;
		j = found as libc::c_short;
		while found == 0 && (j as libc::c_int) < 250 as libc::c_int {
			row = get_rand(
				rooms[rn as usize].top_row as libc::c_int + 1 as libc::c_int,
				rooms[rn as usize].bottom_row as libc::c_int - 1 as libc::c_int,
			) as libc::c_short;
			col = get_rand(
				rooms[rn as usize].left_col as libc::c_int + 1 as libc::c_int,
				rooms[rn as usize].right_col as libc::c_int - 1 as libc::c_int,
			) as libc::c_short;
			if dungeon[row as usize][col as usize] as libc::c_int
				== 0o100 as libc::c_int as libc::c_ushort as libc::c_int
				|| dungeon[row as usize][col as usize] as libc::c_int
				== 0o200 as libc::c_int as libc::c_ushort as libc::c_int
			{
				found = 1 as libc::c_int as libc::c_char;
			}
			j += 1;
			j;
		}
		if found != 0 {
			obj = gr_object();
			place_at(obj, row as libc::c_int, col as libc::c_int);
			nf += 1;
			nf;
		}
		i += 1;
		i;
	}
	return nf as libc::c_int;
}

pub unsafe fn get_room_number(row: libc::c_int, col: libc::c_int) -> libc::c_int {
	for i in 0..MAXROOMS {
		let below_top_wall = row >= rooms[i].top_row;
		let above_bottom_wall = row <= rooms[i].bottom_row;
		let right_of_left_wall = col >= rooms[i].left_col;
		let left_of_right_wall = col <= rooms[i].right_col;
		if below_top_wall && above_bottom_wall && right_of_left_wall && left_of_right_wall {
			return i;
		}
	}
	return NO_ROOM;
}

#[no_mangle]
pub unsafe extern "C" fn is_all_connected() -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut starting_room: libc::c_short = 0;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < 9 as libc::c_int {
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
				ch = (if wmove(stdscr, i as libc::c_int, j as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int) as chtype
				} else {
					winch(stdscr)
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
								waddch(stdscr, ch as chtype);
							}
							if s as libc::c_int
								& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
							{
								let mut monster: *mut object = 0 as *mut object;
								monster = object_at(
									&mut level_monsters,
									i as libc::c_int,
									j as libc::c_int,
								);
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
pub unsafe extern "C" fn dr_course(
	mut monster: *mut object,
	mut entering: libc::c_char,
	mut row: libc::c_short,
	mut col: libc::c_short,
) -> libc::c_int {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut k: libc::c_short = 0;
	let mut rn: libc::c_short = 0;
	let mut r: libc::c_short = 0;
	let mut rr: libc::c_short = 0;
	(*monster).row = row;
	(*monster).col = col;
	if mon_sees(monster, rogue.row as libc::c_int, rogue.col as libc::c_int) != 0 {
		(*monster).trow = -(1 as libc::c_int) as libc::c_short;
		return;
	}
	rn = get_room_number(row as libc::c_int, col as libc::c_int) as libc::c_short;
	if entering != 0 {
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
