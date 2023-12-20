#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;

	fn wattrset(_: *mut WINDOW, _: libc::c_int) -> libc::c_int;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;

	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut rooms: [room; 0];
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	static mut level_monsters: object;
	fn mon_sees() -> libc::c_char;
	fn alloc_object() -> *mut object;
	fn gr_object() -> *mut object;
	static mut cur_level: libc::c_short;
	static mut max_level: libc::c_short;
	static mut blind: libc::c_short;
	static mut levitate: libc::c_short;
	static mut ring_exp: libc::c_short;
	static mut level_points: [libc::c_long; 0];
	static mut mon_disappeared: libc::c_char;
	static mut sustain_strength: libc::c_char;
	static mut maintain_armor: libc::c_char;
	static mut you_can_move_again: *mut libc::c_char;
}

use libc::sprintf;
use ncurses::{addch, refresh};
use crate::prelude::*;


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


#[no_mangle]
pub static mut less_hp: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut flame_name: *mut libc::c_char = b"flame\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;
#[no_mangle]
pub static mut being_held: libc::c_char = 0;

#[no_mangle]
pub unsafe extern "C" fn special_hit(mut monster: *mut object) -> libc::c_int {
	if (*monster).m_flags & 0o1000 as libc::c_long as libc::c_ulong != 0
		&& rand_percent(66 as libc::c_int) != 0
	{
		return;
	}
	if (*monster).m_flags & 0o2000 as libc::c_long as libc::c_ulong != 0 {
		rust(monster);
	}
	if (*monster).m_flags & 0o4000 as libc::c_long as libc::c_ulong != 0 && levitate == 0
	{
		being_held = 1 as libc::c_int as libc::c_char;
	}
	if (*monster).m_flags & 0o10000 as libc::c_long as libc::c_ulong != 0 {
		freeze(monster);
	}
	if (*monster).m_flags & 0o100000 as libc::c_long as libc::c_ulong != 0 {
		sting(monster);
	}
	if (*monster).m_flags & 0o200000 as libc::c_long as libc::c_ulong != 0 {
		drain_life();
	}
	if (*monster).m_flags & 0o400000 as libc::c_long as libc::c_ulong != 0 {
		drop_level();
	}
	if (*monster).m_flags & 0o20000 as libc::c_long as libc::c_ulong != 0 {
		steal_gold(monster);
	} else if (*monster).m_flags & 0o40000 as libc::c_long as libc::c_ulong != 0 {
		steal_item(monster);
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn cough_up(mut monster: *mut object) -> libc::c_int {
	let mut obj: *mut object = 0 as *mut object;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	if (cur_level as libc::c_int) < max_level as libc::c_int {
		return;
	}
	if (*monster).m_flags & 0o20000 as libc::c_long as libc::c_ulong != 0 {
		obj = alloc_object();
		(*obj).what_is = 0o20 as libc::c_int as libc::c_ushort;
		(*obj)
			.quantity = get_rand(
			cur_level as libc::c_int * 15 as libc::c_int,
			cur_level as libc::c_int * 30 as libc::c_int,
		) as libc::c_short;
	} else {
		if rand_percent((*monster).which_kind as libc::c_int) == 0 {
			return;
		}
		obj = gr_object();
	}
	row = (*monster).row;
	col = (*monster).col;
	n = 0 as libc::c_int as libc::c_short;
	while n as libc::c_int <= 5 as libc::c_int {
		i = -(n as libc::c_int) as libc::c_short;
		while i as libc::c_int <= n as libc::c_int {
			if try_to_cough(
				row as libc::c_int + n as libc::c_int,
				col as libc::c_int + i as libc::c_int,
				obj,
			) != 0
			{
				return;
			}
			if try_to_cough(
				row as libc::c_int - n as libc::c_int,
				col as libc::c_int + i as libc::c_int,
				obj,
			) != 0
			{
				return;
			}
			i += 1;
			i;
		}
		i = -(n as libc::c_int) as libc::c_short;
		while i as libc::c_int <= n as libc::c_int {
			if try_to_cough(
				row as libc::c_int + i as libc::c_int,
				col as libc::c_int - n as libc::c_int,
				obj,
			) != 0
			{
				return;
			}
			if try_to_cough(
				row as libc::c_int + i as libc::c_int,
				col as libc::c_int + n as libc::c_int,
				obj,
			) != 0
			{
				return;
			}
			i += 1;
			i;
		}
		n += 1;
		n;
	}
	free_object(obj);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn seek_gold(mut monster: *mut object) -> bool {
	let mut i: libc::c_short = 0;
	let mut j: libc::c_short = 0;
	let mut rn: libc::c_short = 0;
	let mut s: libc::c_short = 0;
	rn = get_room_number((*monster).row as libc::c_int, (*monster).col as libc::c_int)
		as libc::c_short;
	if (rn as libc::c_int) < 0 as libc::c_int {
		return false;
	}
	i = ((*rooms.as_mut_ptr().offset(rn as isize)).top_row as libc::c_int
		+ 1 as libc::c_int) as libc::c_short;
	while (i as libc::c_int)
		< (*rooms.as_mut_ptr().offset(rn as isize)).bottom_row as libc::c_int
	{
		j = ((*rooms.as_mut_ptr().offset(rn as isize)).left_col as libc::c_int
			+ 1 as libc::c_int) as libc::c_short;
		while (j as libc::c_int)
			< (*rooms.as_mut_ptr().offset(rn as isize)).right_col as libc::c_int
		{
			if gold_at(i as libc::c_int, j as libc::c_int) != 0
				&& dungeon[i as usize][j as usize] as libc::c_int
				& 0o2 as libc::c_int as libc::c_ushort as libc::c_int == 0
			{
				(*monster).m_flags |= 0o400 as libc::c_long as libc::c_ulong;
				s = mon_can_go(monster, i as libc::c_int, j as libc::c_int)
					as libc::c_short;
				(*monster).m_flags &= !(0o400 as libc::c_long) as libc::c_ulong;
				if s != 0 {
					move_mon_to(monster, i as libc::c_int, j as libc::c_int);
					(*monster).m_flags |= 0o10 as libc::c_long as libc::c_ulong;
					(*monster).m_flags
						&= !(0o20 as libc::c_long | 0o1000000 as libc::c_long)
						as libc::c_ulong;
					return true;
				}
				(*monster).m_flags &= !(0o1000000 as libc::c_long) as libc::c_ulong;
				(*monster).m_flags |= 0o400 as libc::c_long as libc::c_ulong;
				mv_monster(monster, i as libc::c_int, j as libc::c_int);
				(*monster).m_flags &= !(0o400 as libc::c_long) as libc::c_ulong;
				(*monster).m_flags |= 0o1000000 as libc::c_long as libc::c_ulong;
				return true;
			}
			j += 1;
			j;
		}
		i += 1;
		i;
	}
	return false;
}

pub fn check_gold_seeker(monster: &mut object) {
	monster.m_flags.seeks_gold = false;
}

#[no_mangle]
pub unsafe extern "C" fn check_imitator(mut monster: *mut object) -> bool {
	if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
		wake_up(monster);
		if blind == 0 {
			if wmove(
				stdscr,
				(*monster).row as libc::c_int,
				(*monster).col as libc::c_int,
			) == -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				addch(get_dungeon_char((*monster).row as usize, (*monster).col as usize) as chtype);
			};
			check_message();
			let msg = format!("wait, that's a {}!", mon_name(monster));
			message(&msg, 1 as libc::c_int);
		}
		return true;
	}
	return false;
}

#[no_mangle]
pub unsafe extern "C" fn imitating(
	mut row: libc::c_short,
	mut col: libc::c_short,
) -> libc::c_int {
	if dungeon[row as usize][col as usize] as libc::c_int
		& 0o2 as libc::c_int as libc::c_ushort as libc::c_int != 0
	{
		let mut monster: *mut object = 0 as *mut object;
		monster = object_at(&mut level_monsters, row, col);
		if !monster.is_null() {
			if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
				return 1 as libc::c_int;
			}
		}
	}
	return 0 as libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn m_confuse(mut monster: *mut object) -> bool {
	let mut msg: [libc::c_char; 80] = [0; 80];
	if rogue_can_see((*monster).row as libc::c_int, (*monster).col as libc::c_int) == 0 {
		return false;
	}
	if rand_percent(45 as libc::c_int) != 0 {
		(*monster).m_flags &= !(0o10000000 as libc::c_long) as libc::c_ulong;
		return false;
	}
	if rand_percent(55 as libc::c_int) != 0 {
		(*monster).m_flags &= !(0o10000000 as libc::c_long) as libc::c_ulong;
		sprintf(
			msg.as_mut_ptr(),
			b"the gaze of the %s has confused you\0" as *const u8 as *const libc::c_char,
			mon_name(monster),
		);
		message(msg.as_mut_ptr(), 1 as libc::c_int);
		confuse();
		return true;
	}
	return false;
}

#[no_mangle]
pub unsafe extern "C" fn flame_broil(mut monster: *mut object) -> bool {
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	if mon_sees(monster, rogue.row as libc::c_int, rogue.col as libc::c_int) == 0 || coin_toss() {
		return false;
	}
	row = (rogue.row as libc::c_int - (*monster).row as libc::c_int) as libc::c_short;
	col = (rogue.col as libc::c_int - (*monster).col as libc::c_int) as libc::c_short;
	if (row as libc::c_int) < 0 as libc::c_int {
		row = -(row as libc::c_int) as libc::c_short;
	}
	if (col as libc::c_int) < 0 as libc::c_int {
		col = -(col as libc::c_int) as libc::c_short;
	}
	if row as libc::c_int != 0 as libc::c_int && col as libc::c_int != 0 as libc::c_int
		&& row as libc::c_int != col as libc::c_int
		|| (row as libc::c_int > 7 as libc::c_int
		|| col as libc::c_int > 7 as libc::c_int)
	{
		return false;
	}
	if blind == 0
		&& rogue_is_around((*monster).row as libc::c_int, (*monster).col as libc::c_int)
		== 0
	{
		row = (*monster).row;
		col = (*monster).col;
		get_closer(
			&mut row,
			&mut col,
			rogue.row as libc::c_int,
			rogue.col as libc::c_int,
		);
		wattrset(
			stdscr,
			((1 as libc::c_uint) << 8 as libc::c_int + 8 as libc::c_int) as libc::c_int,
		);
		loop {
			if wmove(stdscr, row as libc::c_int, col as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				addch('~' as i32 as chtype);
			};
			refresh();
			get_closer(
				&mut row,
				&mut col,
				rogue.row as libc::c_int,
				rogue.col as libc::c_int,
			);
			if !(row as libc::c_int != rogue.row as libc::c_int
				|| col as libc::c_int != rogue.col as libc::c_int)
			{
				break;
			}
		}
		wattrset(
			stdscr,
			(1 as libc::c_uint).wrapping_sub(1 as libc::c_uint) as libc::c_int,
		);
		row = (*monster).row;
		col = (*monster).col;
		get_closer(
			&mut row,
			&mut col,
			rogue.row as libc::c_int,
			rogue.col as libc::c_int,
		);
		loop {
			if wmove(stdscr, row as libc::c_int, col as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int);
			} else {
				addch(get_dungeon_char(row as usize, col as usize) as chtype);
			};
			refresh();
			get_closer(
				&mut row,
				&mut col,
				rogue.row as libc::c_int,
				rogue.col as libc::c_int,
			);
			if !(row as libc::c_int != rogue.row as libc::c_int
				|| col as libc::c_int != rogue.col as libc::c_int)
			{
				break;
			}
		}
	}
	mon_hit(monster, flame_name, 1 as libc::c_int);
	return true;
}
