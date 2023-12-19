#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn wclear(_: *mut WINDOW) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
	fn fread(
		_: *mut libc::c_void,
		_: libc::c_ulong,
		_: libc::c_ulong,
		_: *mut FILE,
	) -> libc::c_ulong;
	fn fwrite(
		_: *const libc::c_void,
		_: libc::c_ulong,
		_: libc::c_ulong,
		_: *mut FILE,
	) -> libc::c_ulong;
	fn fclose(_: *mut FILE) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut rooms: [room; 0];
	static mut traps: [trap; 0];
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	static mut id_scrolls: [id; 0];
	static mut id_potions: [id; 0];
	static mut id_wands: [id; 0];
	static mut id_rings: [id; 0];
	static mut level_monsters: object;
	fn strcpy(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn md_getenv() -> *mut libc::c_char;
	fn md_df() -> libc::c_char;
	fn alloc_object() -> *mut object;
	fn xxx() -> libc::c_long;
	static mut detect_monster: libc::c_char;
	static mut cur_level: libc::c_short;
	static mut max_level: libc::c_short;
	static mut hunger_str: [libc::c_char; 0];
	static mut party_room: libc::c_short;
	static mut party_counter: libc::c_short;
	static mut foods: libc::c_short;
	static mut is_wood: [libc::c_char; 0];
	static mut cur_room: libc::c_short;
	static mut being_held: libc::c_char;
	static mut bear_trap: libc::c_short;
	static mut halluc: libc::c_short;
	static mut blind: libc::c_short;
	static mut confused: libc::c_short;
	static mut levitate: libc::c_short;
	static mut haste_self: libc::c_short;
	static mut see_invisible: libc::c_char;
	static mut wizard: libc::c_char;
	static mut m_moves: libc::c_short;
	static mut msg_cleared: libc::c_char;
	fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
	fn strlen(_: *const libc::c_char) -> libc::c_ulong;
}

use crate::prelude::*;


pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
pub type fpos_t = __darwin_off_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
	pub _base: *mut libc::c_uchar,
	pub _size: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: libc::c_int,
	pub _w: libc::c_int,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
	pub _lbfsize: libc::c_int,
	pub _cookie: *mut libc::c_void,
	pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
	pub _read: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*mut libc::c_char,
			libc::c_int,
		) -> libc::c_int,
	>,
	pub _seek: Option::<
		unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
	>,
	pub _write: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*const libc::c_char,
			libc::c_int,
		) -> libc::c_int,
	>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: libc::c_int,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: libc::c_int,
	pub _offset: fpos_t,
}

pub type FILE = __sFILE;

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
pub struct id {
	pub value: libc::c_short,
	pub title: [libc::c_char; 128],
	pub real: [libc::c_char; 128],
	pub id_status: libc::c_ushort,
}


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
pub struct tr {
	pub trap_type: libc::c_short,
	pub trap_row: libc::c_short,
	pub trap_col: libc::c_short,
}

pub type trap = tr;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct rogue_time {
	pub year: libc::c_short,
	pub month: libc::c_short,
	pub day: libc::c_short,
	pub hour: libc::c_short,
	pub minute: libc::c_short,
	pub second: libc::c_short,
}

#[no_mangle]
pub static mut write_failed: libc::c_short = 0 as libc::c_int as libc::c_short;

#[no_mangle]
pub unsafe extern "C" fn save_game() -> libc::c_int {
	let mut fname: [libc::c_char; 64] = [0; 64];
	if get_input_line(
		b"file name?\0" as *const u8 as *const libc::c_char,
		save_file,
		fname.as_mut_ptr(),
		b"game not saved\0" as *const u8 as *const libc::c_char,
		0 as libc::c_int,
		1 as libc::c_int,
	) == 0
	{
		return;
	}
	check_message();
	message(fname.as_mut_ptr(), 0 as libc::c_int);
	save_into_file(fname.as_mut_ptr());
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn restore(fname: &str) -> libc::c_int {
	let mut fp: *mut FILE = 0 as *mut FILE;
	let mut saved_time: rogue_time = rogue_time {
		year: 0,
		month: 0,
		day: 0,
		hour: 0,
		minute: 0,
		second: 0,
	};
	let mut mod_time: rogue_time = rogue_time {
		year: 0,
		month: 0,
		day: 0,
		hour: 0,
		minute: 0,
		second: 0,
	};
	let mut buf: [libc::c_char; 4] = [0; 4];
	let mut tbuf: [libc::c_char; 40] = [0; 40];
	let mut new_file_id: libc::c_int = 0;
	let mut saved_file_id: libc::c_int = 0;
	new_file_id = md_get_file_id(fname);
	if new_file_id == -(1 as libc::c_int)
		|| {
		fp = fopen(fname, b"r\0" as *const u8 as *const libc::c_char);
		fp.is_null()
	}
	{
		clean_up(b"cannot open file\0" as *const u8 as *const libc::c_char);
	}
	if md_link_count(fname) > 1 as libc::c_int {
		clean_up(b"file has link\0" as *const u8 as *const libc::c_char);
	}
	xxx(1 as libc::c_int);
	r_read(
		fp,
		&mut detect_monster as *mut libc::c_char,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut cur_level as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut max_level as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	read_string(hunger_str.as_mut_ptr(), fp);
	strcpy(tbuf.as_mut_ptr(), login_name.as_mut_ptr());
	read_string(login_name.as_mut_ptr(), fp);
	if strcmp(tbuf.as_mut_ptr(), login_name.as_mut_ptr()) != 0 {
		clean_up(
			b"you're not the original player\0" as *const u8 as *const libc::c_char,
		);
	}
	r_read(
		fp,
		&mut party_room as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut party_counter as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	read_pack(&mut level_monsters, fp, 0 as libc::c_int);
	read_pack(&mut level_objects, fp, 0 as libc::c_int);
	r_read(
		fp,
		&mut saved_file_id as *mut libc::c_int as *mut libc::c_char,
		::core::mem::size_of::<libc::c_int>() as libc::c_ulong,
	);
	if new_file_id != saved_file_id {
		clean_up(
			b"sorry, saved game is not in the same file\0" as *const u8
				as *const libc::c_char,
		);
	}
	rw_dungeon(fp, 0 as libc::c_int);
	r_read(
		fp,
		&mut foods as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut rogue as *mut fighter as *mut libc::c_char,
		::core::mem::size_of::<fighter>() as libc::c_ulong,
	);
	read_pack(&mut rogue.pack, fp, 1 as libc::c_int);
	rw_id_alloc(id_potions.as_mut_ptr(), fp, 14 as libc::c_int, 0 as libc::c_int);
	rw_id_alloc(id_scrolls.as_mut_ptr(), fp, 12 as libc::c_int, 0 as libc::c_int);
	rw_id_alloc(id_wands.as_mut_ptr(), fp, 10 as libc::c_int, 0 as libc::c_int);
	rw_id_alloc(id_rings.as_mut_ptr(), fp, 11 as libc::c_int, 0 as libc::c_int);
	r_read(
		fp,
		traps.as_mut_ptr() as *mut libc::c_char,
		(10 as libc::c_int as libc::c_ulong)
			.wrapping_mul(::core::mem::size_of::<trap>() as libc::c_ulong),
	);
	r_read(
		fp,
		is_wood.as_mut_ptr(),
		(10 as libc::c_int as libc::c_ulong)
			.wrapping_mul(::core::mem::size_of::<libc::c_char>() as libc::c_ulong),
	);
	r_read(
		fp,
		&mut cur_room as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	rw_rooms(fp, 0 as libc::c_int);
	r_read(
		fp,
		&mut being_held as *mut libc::c_char,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut bear_trap as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut halluc as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut blind as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut confused as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut levitate as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut haste_self as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut see_invisible as *mut libc::c_char,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut detect_monster as *mut libc::c_char,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut wizard as *mut libc::c_char,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut score_only as *mut libc::c_char,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut m_moves as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	r_read(
		fp,
		&mut saved_time as *mut rogue_time as *mut libc::c_char,
		::core::mem::size_of::<rogue_time>() as libc::c_ulong,
	);
	if fread(
		buf.as_mut_ptr() as *mut libc::c_void,
		::core::mem::size_of::<libc::c_char>() as libc::c_ulong,
		1 as libc::c_int as libc::c_ulong,
		fp,
	) > 0 as libc::c_int as libc::c_ulong
	{
		wclear(stdscr);
		clean_up(b"extra characters in file\0" as *const u8 as *const libc::c_char);
	}
	md_gfmt(fname, &mut mod_time);
	if has_been_touched(&mut saved_time, &mut mod_time) != 0 {
		wclear(stdscr);
		clean_up(b"sorry, file has been touched\0" as *const u8 as *const libc::c_char);
	}
	if wizard == 0 && md_df(fname) == 0 {
		clean_up(b"cannot delete file\0" as *const u8 as *const libc::c_char);
	}
	msg_cleared = 0 as libc::c_int as libc::c_char;
	ring_stats(0 as libc::c_int);
	fclose(fp);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn rw_id(
	mut id_table: *mut id,
	mut fp: *mut FILE,
	mut n: libc::c_int,
	mut wr: libc::c_char,
) -> libc::c_int {
	let mut i: libc::c_short = 0;
	i = 0 as libc::c_int as libc::c_short;
	while (i as libc::c_int) < n {
		if wr != 0 {
			r_write(
				fp,
				&mut (*id_table.offset(i as isize)).value as *mut libc::c_short
					as *mut libc::c_char,
				::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
			);
			r_write(
				fp,
				&mut (*id_table.offset(i as isize)).id_status as *mut libc::c_ushort
					as *mut libc::c_char,
				::core::mem::size_of::<libc::c_ushort>() as libc::c_ulong,
			);
			write_string(((*id_table.offset(i as isize)).title).as_mut_ptr(), fp);
		} else {
			r_read(
				fp,
				&mut (*id_table.offset(i as isize)).value as *mut libc::c_short
					as *mut libc::c_char,
				::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
			);
			r_read(
				fp,
				&mut (*id_table.offset(i as isize)).id_status as *mut libc::c_ushort
					as *mut libc::c_char,
				::core::mem::size_of::<libc::c_ushort>() as libc::c_ulong,
			);
			read_string(((*id_table.offset(i as isize)).title).as_mut_ptr(), fp);
		}
		i += 1;
		i;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn has_been_touched(
	mut saved_time: *mut rogue_time,
	mut mod_time: *mut rogue_time,
) -> libc::c_char {
	if ((*saved_time).year as libc::c_int) < (*mod_time).year as libc::c_int {
		return 1 as libc::c_int as libc::c_char;
	} else if (*saved_time).year as libc::c_int > (*mod_time).year as libc::c_int {
		return 0 as libc::c_int as libc::c_char;
	}
	if ((*saved_time).month as libc::c_int) < (*mod_time).month as libc::c_int {
		return 1 as libc::c_int as libc::c_char;
	} else if (*saved_time).month as libc::c_int > (*mod_time).month as libc::c_int {
		return 0 as libc::c_int as libc::c_char;
	}
	if ((*saved_time).day as libc::c_int) < (*mod_time).day as libc::c_int {
		return 1 as libc::c_int as libc::c_char;
	} else if (*saved_time).day as libc::c_int > (*mod_time).day as libc::c_int {
		return 0 as libc::c_int as libc::c_char;
	}
	if ((*saved_time).hour as libc::c_int) < (*mod_time).hour as libc::c_int {
		return 1 as libc::c_int as libc::c_char;
	} else if (*saved_time).hour as libc::c_int > (*mod_time).hour as libc::c_int {
		return 0 as libc::c_int as libc::c_char;
	}
	if ((*saved_time).minute as libc::c_int) < (*mod_time).minute as libc::c_int {
		return 1 as libc::c_int as libc::c_char;
	} else if (*saved_time).minute as libc::c_int > (*mod_time).minute as libc::c_int {
		return 0 as libc::c_int as libc::c_char;
	}
	if ((*saved_time).second as libc::c_int) < (*mod_time).second as libc::c_int {
		return 1 as libc::c_int as libc::c_char;
	}
	return 0 as libc::c_int as libc::c_char;
}
