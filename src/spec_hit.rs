#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;

	fn wattrset(_: *mut WINDOW, _: i64) -> i64;

	fn mon_sees() -> libc::c_char;
	fn alloc_object() -> *mut object;
	fn gr_object() -> *mut object;
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


#[no_mangle]
pub static mut less_hp: libc::c_short = 0 as i64 as libc::c_short;
#[no_mangle]
pub static mut flame_name: *mut libc::c_char = b"flame\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

pub static mut being_held: bool = false;

#[no_mangle]
pub unsafe extern "C" fn special_hit(mut monster: *mut object) -> i64 {
	if (*monster).m_flags & 0o1000 as libc::c_long as libc::c_ulong != 0
		&& rand_percent(66 as i64) != 0
	{
		return;
	}
	if (*monster).m_flags & 0o2000 as libc::c_long as libc::c_ulong != 0 {
		rust(monster);
	}
	if (*monster).m_flags & 0o4000 as libc::c_long as libc::c_ulong != 0 && levitate == 0
	{
		being_held = 1 as libc::c_char;
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
pub unsafe extern "C" fn cough_up(mut monster: *mut object) -> i64 {
	let mut obj: *mut object = 0 as *mut object;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut i: libc::c_short = 0;
	let mut n: libc::c_short = 0;
	if (cur_level as i64) < max_level as i64 {
		return;
	}
	if (*monster).m_flags & 0o20000 as libc::c_long as libc::c_ulong != 0 {
		obj = alloc_object();
		(*obj).what_is = 0o20 as i64 as libc::c_ushort;
		(*obj)
			.quantity = get_rand(
			cur_level as i64 * 15 as i64,
			cur_level as i64 * 30 as i64,
		) as libc::c_short;
	} else {
		if rand_percent((*monster).which_kind as i64) == 0 {
			return;
		}
		obj = gr_object();
	}
	row = (*monster).row;
	col = (*monster).col;
	n = 0 as i64 as libc::c_short;
	while n as i64 <= 5 as i64 {
		i = -(n as i64) as libc::c_short;
		while i as i64 <= n as i64 {
			if try_to_cough(
				row as i64 + n as i64,
				col as i64 + i as i64,
				obj,
			) != 0
			{
				return;
			}
			if try_to_cough(
				row as i64 - n as i64,
				col as i64 + i as i64,
				obj,
			) != 0
			{
				return;
			}
			i += 1;
			i;
		}
		i = -(n as i64) as libc::c_short;
		while i as i64 <= n as i64 {
			if try_to_cough(
				row as i64 + i as i64,
				col as i64 - n as i64,
				obj,
			) != 0
			{
				return;
			}
			if try_to_cough(
				row as i64 + i as i64,
				col as i64 + n as i64,
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
	rn = get_room_number((*monster).row as i64, (*monster).col as i64)
		as libc::c_short;
	if (rn as i64) < 0 as i64 {
		return false;
	}
	i = ((*rooms.as_mut_ptr().offset(rn as isize)).top_row as i64
		+ 1) as libc::c_short;
	while (i as i64)
		< (*rooms.as_mut_ptr().offset(rn as isize)).bottom_row as i64
	{
		j = ((*rooms.as_mut_ptr().offset(rn as isize)).left_col as i64
			+ 1) as libc::c_short;
		while (j as i64)
			< (*rooms.as_mut_ptr().offset(rn as isize)).right_col as i64
		{
			if gold_at(i as i64, j as i64) != 0
				&& dungeon[i as usize][j as usize] as i64
				& 0o2 as i64 as libc::c_ushort as i64 == 0
			{
				(*monster).m_flags |= 0o400 as libc::c_long as libc::c_ulong;
				s = mon_can_go(monster, i as i64, j as i64)
					as libc::c_short;
				(*monster).m_flags &= !(0o400 as libc::c_long) as libc::c_ulong;
				if s != 0 {
					move_mon_to(monster, i as i64, j as i64);
					(*monster).m_flags |= 0o10 as libc::c_long as libc::c_ulong;
					(*monster).m_flags
						&= !(0o20 as libc::c_long | 0o1000000 as libc::c_long)
						as libc::c_ulong;
					return true;
				}
				(*monster).m_flags &= !(0o1000000 as libc::c_long) as libc::c_ulong;
				(*monster).m_flags |= 0o400 as libc::c_long as libc::c_ulong;
				mv_monster(monster, i as i64, j as i64);
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
			if ncurses::wmove(
				ncurses::stdscr(),
				(*monster).row as i64,
				(*monster).col as i64,
			) == -(1)
			{
				-(1);
			} else {
				addch(get_dungeon_char((*monster).row as usize, (*monster).col as usize) as ncurses::chtype);
			};
			check_message();
			let msg = format!("wait, that's a {}!", mon_name(monster));
			message(&msg, 1);
		}
		return true;
	}
	return false;
}

#[no_mangle]
pub unsafe extern "C" fn imitating(
	mut row: libc::c_short,
	mut col: libc::c_short,
) -> i64 {
	if dungeon[row as usize][col as usize] as i64
		& 0o2 as i64 as libc::c_ushort as i64 != 0
	{
		let mut monster: *mut object = 0 as *mut object;
		monster = object_at(&mut level_monsters, row, col);
		if !monster.is_null() {
			if (*monster).m_flags & 0o20000000 as libc::c_long as libc::c_ulong != 0 {
				return 1;
			}
		}
	}
	return 0 as i64;
}

#[no_mangle]
pub unsafe extern "C" fn m_confuse(mut monster: *mut object) -> bool {
	let mut msg: [libc::c_char; 80] = [0; 80];
	if rogue_can_see((*monster).row as i64, (*monster).col as i64) == 0 {
		return false;
	}
	if rand_percent(45 as i64) != 0 {
		(*monster).m_flags &= !(0o10000000 as libc::c_long) as libc::c_ulong;
		return false;
	}
	if rand_percent(55 as i64) != 0 {
		(*monster).m_flags &= !(0o10000000 as libc::c_long) as libc::c_ulong;
		sprintf(
			msg.as_mut_ptr(),
			b"the gaze of the %s has confused you\0" as *const u8 as *const libc::c_char,
			mon_name(monster),
		);
		message(msg.as_mut_ptr(), 1);
		confuse();
		return true;
	}
	return false;
}

#[no_mangle]
pub unsafe extern "C" fn flame_broil(mut monster: *mut object) -> bool {
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	if mon_sees(monster, rogue.row as i64, rogue.col as i64) == 0 || coin_toss() {
		return false;
	}
	row = (rogue.row as i64 - (*monster).row as i64) as libc::c_short;
	col = (rogue.col as i64 - (*monster).col as i64) as libc::c_short;
	if (row as i64) < 0 as i64 {
		row = -(row as i64) as libc::c_short;
	}
	if (col as i64) < 0 as i64 {
		col = -(col as i64) as libc::c_short;
	}
	if row as i64 != 0 as i64 && col as i64 != 0 as i64
		&& row as i64 != col as i64
		|| (row as i64 > 7 as i64
		|| col as i64 > 7 as i64)
	{
		return false;
	}
	if blind == 0
		&& rogue_is_around((*monster).row as i64, (*monster).col as i64)
		== 0
	{
		row = (*monster).row;
		col = (*monster).col;
		get_closer(
			&mut row,
			&mut col,
			rogue.row as i64,
			rogue.col as i64,
		);
		wattrset(
			ncurses::stdscr(),
			((1 as libc::c_uint) << 8 as i64 + 8 as i64) as i64,
		);
		loop {
			if ncurses::wmove(ncurses::stdscr(), row as i64, col as i64)
				== -(1)
			{
				-(1);
			} else {
				addch('~' as i32 as ncurses::chtype);
			};
			refresh();
			get_closer(
				&mut row,
				&mut col,
				rogue.row as i64,
				rogue.col as i64,
			);
			if !(row as i64 != rogue.row as i64
				|| col as i64 != rogue.col as i64)
			{
				break;
			}
		}
		wattrset(
			ncurses::stdscr(),
			(1 as libc::c_uint).wrapping_sub(1 as libc::c_uint) as i64,
		);
		row = (*monster).row;
		col = (*monster).col;
		get_closer(
			&mut row,
			&mut col,
			rogue.row as i64,
			rogue.col as i64,
		);
		loop {
			if ncurses::wmove(ncurses::stdscr(), row as i64, col as i64)
				== -(1)
			{
				-(1);
			} else {
				addch(get_dungeon_char(row as usize, col as usize) as ncurses::chtype);
			};
			refresh();
			get_closer(
				&mut row,
				&mut col,
				rogue.row as i64,
				rogue.col as i64,
			);
			if !(row as i64 != rogue.row as i64
				|| col as i64 != rogue.col as i64)
			{
				break;
			}
		}
	}
	mon_hit(monster, flame_name, 1);
	return true;
}
