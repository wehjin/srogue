#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;

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
	fn fclose(_: *mut FILE) -> i64;
	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn md_getenv() -> *mut libc::c_char;
	fn alloc_object() -> *mut object;
	static mut msg_cleared: libc::c_char;
	fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> i64;
}

use std::env;
use std::fs::File;
use std::io::Write;
use libc::strcpy;
use crate::prelude::*;
use crate::settings::{login_name, save_file, score_only};


pub type __int64_t = libc::c_longlong;
pub type __darwin_off_t = __int64_t;
pub type fpos_t = __darwin_off_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
	pub _base: *mut libc::c_uchar,
	pub _size: i64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
	pub _p: *mut libc::c_uchar,
	pub _r: i64,
	pub _w: i64,
	pub _flags: libc::c_short,
	pub _file: libc::c_short,
	pub _bf: __sbuf,
	pub _lbfsize: i64,
	pub _cookie: *mut libc::c_void,
	pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> i64>,
	pub _read: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*mut libc::c_char,
			i64,
		) -> i64,
	>,
	pub _seek: Option::<
		unsafe extern "C" fn(*mut libc::c_void, fpos_t, i64) -> fpos_t,
	>,
	pub _write: Option::<
		unsafe extern "C" fn(
			*mut libc::c_void,
			*const libc::c_char,
			i64,
		) -> i64,
	>,
	pub _ub: __sbuf,
	pub _extra: *mut __sFILEX,
	pub _ur: i64,
	pub _ubuf: [libc::c_uchar; 3],
	pub _nbuf: [libc::c_uchar; 1],
	pub _lb: __sbuf,
	pub _blksize: i64,
	pub _offset: fpos_t,
}

pub type FILE = __sFILE;


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
pub unsafe extern "C" fn save_game() {
	let file_name = get_input_line(
		"file name?",
		save_file().map(|f| &f as &str),
		Some("game not saved"),
		false,
		true,
	);
	if file_name.is_empty() {
		return;
	}
	check_message();
	message(&file_name, 0);
	save_into_file(&file_name);
}

mod data;

pub unsafe fn save_into_file(sfile: &str) {
	let sfile = expand_tilde(&sfile);
	let file = File::create(&sfile);
	let mut file = match file {
		Err(_) => {
			message("problem accessing the save file", 0);
			return;
		}
		Ok(file) => {
			file
		}
	};
	let file_id = md_get_file_id(&sfile);
	if file_id == -1 {
		message("problem accessing the save file", 0);
		return;
	}
	md_ignore_signals();
	xxx(true);
	let save_data = data::from_statics(file_id as i32);
	let json = serde_json::to_string_pretty(&save_data).expect("serialize data");
	let buf = json.as_bytes();
	let write_failed = if let Err(_) = file.write(buf) {
		message("write() failed, don't know why", 0);
		sound_bell();
		true
	} else {
		false
	};
	drop(file);
	if write_failed {
		md_df(&sfile);
	} else {
		clean_up("");
	}
}

fn expand_tilde(file: &str) -> String {
	if file.starts_with('~') {
		if let Ok(home) = env::var("HOME") {
			format!("{}{}", home, &file[1..])
		} else {
			file.to_string()
		}
	} else {
		file.to_string()
	}
}

#[no_mangle]
pub unsafe extern "C" fn restore(fname: &str) -> i64 {
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
	let mut new_file_id: i64 = 0;
	let mut saved_file_id: i64 = 0;
	new_file_id = md_get_file_id(fname);
	if new_file_id == -(1)
		|| {
		fp = fopen(fname, b"r\0" as *const u8 as *const libc::c_char);
		fp.is_null()
	}
	{
		clean_up("cannot open file");
	}
	if md_link_count(fname) > 1 {
		clean_up("file has link");
	}
	xxx(1);
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
	strcpy(tbuf.as_mut_ptr(), &login_name());
	read_string(&login_name(), fp);
	if strcmp(tbuf.as_mut_ptr(), &login_name()) != 0 {
		clean_up("you're not the original player");
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
	read_pack(&mut level_monsters, fp, 0 as i64);
	read_pack(&mut level_objects, fp, 0 as i64);
	r_read(
		fp,
		&mut saved_file_id as *mut i64 as *mut libc::c_char,
		::core::mem::size_of::<i64>() as libc::c_ulong,
	);
	if new_file_id != saved_file_id {
		clean_up("sorry, saved game is not in the same file");
	}
	rw_dungeon(fp, 0 as i64);
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
	read_pack(&mut rogue.pack, fp, 1);
	rw_id_alloc(id_potions.as_mut_ptr(), fp, 14 as i64, 0 as i64);
	rw_id_alloc(id_scrolls.as_mut_ptr(), fp, 12 as i64, 0 as i64);
	rw_id_alloc(id_wands.as_mut_ptr(), fp, 10 as i64, 0 as i64);
	rw_id_alloc(id_rings.as_mut_ptr(), fp, 11, 0 as i64);
	r_read(
		fp,
		traps.as_mut_ptr() as *mut libc::c_char,
		(10 as i64 as libc::c_ulong)
			.wrapping_mul(::core::mem::size_of::<trap>() as libc::c_ulong),
	);
	r_read(
		fp,
		is_wood.as_mut_ptr(),
		(10 as i64 as libc::c_ulong)
			.wrapping_mul(::core::mem::size_of::<libc::c_char>() as libc::c_ulong),
	);
	r_read(
		fp,
		&mut cur_room as *mut libc::c_short as *mut libc::c_char,
		::core::mem::size_of::<libc::c_short>() as libc::c_ulong,
	);
	rw_rooms(fp, 0 as i64);
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
		score_only(),
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
		1 as libc::c_ulong,
		fp,
	) > 0 as i64 as libc::c_ulong
	{
		ncurses::wclear(ncurses::stdscr());
		clean_up("extra characters in file");
	}
	let mod_time = md_gfmt(fname);
	if has_been_touched(&mut saved_time, &mut mod_time) != 0 {
		ncurses::wclear(ncurses::stdscr());
		clean_up("sorry, file has been touched");
	}
	if wizard == 0 && md_df(fname) == 0 {
		clean_up("cannot delete file");
	}
	msg_cleared = 0 as i64 as libc::c_char;
	ring_stats(0 as i64);
	fclose(fp);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn has_been_touched(
	mut saved_time: *mut rogue_time,
	mut mod_time: *mut rogue_time,
) -> libc::c_char {
	if ((*saved_time).year as i64) < (*mod_time).year as i64 {
		return 1 as libc::c_char;
	} else if (*saved_time).year as i64 > (*mod_time).year as i64 {
		return 0 as i64 as libc::c_char;
	}
	if ((*saved_time).month as i64) < (*mod_time).month as i64 {
		return 1 as libc::c_char;
	} else if (*saved_time).month as i64 > (*mod_time).month as i64 {
		return 0 as i64 as libc::c_char;
	}
	if ((*saved_time).day as i64) < (*mod_time).day as i64 {
		return 1 as libc::c_char;
	} else if (*saved_time).day as i64 > (*mod_time).day as i64 {
		return 0 as i64 as libc::c_char;
	}
	if ((*saved_time).hour as i64) < (*mod_time).hour as i64 {
		return 1 as libc::c_char;
	} else if (*saved_time).hour as i64 > (*mod_time).hour as i64 {
		return 0 as i64 as libc::c_char;
	}
	if ((*saved_time).minute as i64) < (*mod_time).minute as i64 {
		return 1 as libc::c_char;
	} else if (*saved_time).minute as i64 > (*mod_time).minute as i64 {
		return 0 as i64 as libc::c_char;
	}
	if ((*saved_time).second as i64) < (*mod_time).second as i64 {
		return 1 as libc::c_char;
	}
	return 0 as i64 as libc::c_char;
}
