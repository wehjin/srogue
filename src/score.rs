#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type __sFILEX;
	pub type ldat;

	fn waddnstr(_: *mut WINDOW, _: *const libc::c_char, _: libc::c_int) -> libc::c_int;
	fn wattrset(_: *mut WINDOW, _: libc::c_int) -> libc::c_int;
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
	fn rewind(_: *mut FILE);

	fn fclose(_: *mut FILE) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut id_scrolls: [id; 0];
	static mut id_potions: [id; 0];
	static mut id_wands: [id; 0];
	static mut id_rings: [id; 0];
	static mut id_weapons: [id; 0];
	static mut id_armors: [id; 0];
	fn strncpy(
		_: *mut libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> *mut libc::c_char;
	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	static mut m_names: [*mut libc::c_char; 0];
	static mut max_level: libc::c_short;
	static mut msg_cleared: libc::c_char;
	static mut byebye_string: *mut libc::c_char;
	fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
}

use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{RwLock};
use libc::{sprintf, strcpy, strlen};
use ncurses::{addch, clear, mvaddstr, refresh, standend, standout};
use settings::score_only;
use crate::prelude::*;
use crate::{settings, turn_into_games, turn_into_user};
use crate::settings::{login_name, nick_name, SETTINGS};

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

pub const SCORE_FILE: &'static str = "/usr/games/rogue.scores";

#[no_mangle]
pub unsafe extern "C" fn killed_by(
	mut monster: *mut object,
	mut other: libc::c_short,
) -> libc::c_int {
	let mut buf: [libc::c_char; 80] = [0; 80];
	md_ignore_signals();
	if other as libc::c_int != 4 as libc::c_int {
		rogue
			.gold = rogue.gold * 9 as libc::c_int as libc::c_long
			/ 10 as libc::c_int as libc::c_long;
	}
	if other != 0 {
		match other as libc::c_int {
			1 => {
				strcpy(buf.as_mut_ptr(), b"died of hypothermia\0" as *const u8 as *const libc::c_char);
			}
			2 => {
				strcpy(buf.as_mut_ptr(), b"died of starvation\0" as *const u8 as *const libc::c_char);
			}
			3 => {
				strcpy(buf.as_mut_ptr(), b"killed by a dart\0" as *const u8 as *const libc::c_char);
			}
			4 => {
				strcpy(buf.as_mut_ptr(), b"quit\0" as *const u8 as *const libc::c_char);
			}
			_ => {}
		}
	} else {
		strcpy(buf.as_mut_ptr(), b"Killed by \0" as *const u8 as *const libc::c_char);
		if is_vowel(
			*(*m_names
				.as_mut_ptr()
				.offset(((*monster).ichar as libc::c_int - 'A' as i32) as isize))
				.offset(0 as libc::c_int as isize) as libc::c_int,
		) != 0
		{
			strcat(buf.as_mut_ptr(), b"an \0" as *const u8 as *const libc::c_char);
		} else {
			strcat(buf.as_mut_ptr(), b"a \0" as *const u8 as *const libc::c_char);
		}
		strcat(
			buf.as_mut_ptr(),
			*m_names
				.as_mut_ptr()
				.offset(((*monster).ichar as libc::c_int - 'A' as i32) as isize),
		);
	}
	strcat(buf.as_mut_ptr(), b" with \0" as *const u8 as *const libc::c_char);
	sprintf(
		buf.as_mut_ptr().offset(strlen(buf.as_mut_ptr()) as isize),
		b"%ld gold\0" as *const u8 as *const libc::c_char,
		rogue.gold,
	);
	if other == 0 && SETTINGS.get().show_skull as libc::c_int != 0 {
		wclear(stdscr);
		if wmove(stdscr, 4 as libc::c_int, 32 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"__---------__\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 5 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"_~             ~_\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 6 as libc::c_int, 29 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"/                 \\\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 7 as libc::c_int, 28 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"~                   ~\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 8 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"/                     \\\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 9 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"|    XXXX     XXXX    |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 10 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"|    XXXX     XXXX    |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 11 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"|    XXX       XXX    |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 12 as libc::c_int, 28 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"\\         @         /\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 13 as libc::c_int, 29 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"--\\     @@@     /--\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 14 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"| |    @@@    | |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 15 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"| |           | |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 16 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"| vvVvvvvvvvVvv |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 17 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"|  ^^^^^^^^^^^  |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 18 as libc::c_int, 31 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"\\_           _/\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if wmove(stdscr, 19 as libc::c_int, 33 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				stdscr,
				b"~---------~\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		center(
			21 as libc::c_int,
			if let Some(name) = &nick_name() { name.as_str() } else { &login_name() },
		);
		center(22 as libc::c_int, buf.as_mut_ptr());
	} else {
		message(buf.as_mut_ptr(), 0 as libc::c_int);
	}
	message("", 0 as libc::c_int);
	let monster = if monster.is_null() {
		None
	} else {
		Some(&*monster)
	};
	put_scores(monster, other);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn win() -> libc::c_int {
	unwield(rogue.weapon);
	unwear(rogue.armor);
	un_put_on(rogue.left_ring);
	un_put_on(rogue.right_ring);
	wclear(stdscr);
	if wmove(stdscr, 10 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"@   @  @@@   @   @      @  @  @   @@@   @   @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 11 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b" @ @  @   @  @   @      @  @  @  @   @  @@  @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 12 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"  @   @   @  @   @      @  @  @  @   @  @ @ @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 13 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"  @   @   @  @   @      @  @  @  @   @  @  @@\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 14 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"  @    @@@    @@@        @@ @@    @@@   @   @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 17 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"Congratulations,  you have  been admitted  to  the\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 18 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"Fighters' Guild.   You return home,  sell all your\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if wmove(stdscr, 19 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			stdscr,
			b"treasures at great profit and retire into comfort.\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	message(b"\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
	message(b"\0" as *const u8 as *const libc::c_char, 0 as libc::c_int);
	id_all();
	sell_pack();
	put_scores(0 as *mut object, 5 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn quit(mut from_intrpt: libc::c_char) {
	let mut buf: [libc::c_char; 128] = [0; 128];
	let mut i: libc::c_short = 0;
	let mut orow: libc::c_short = 0;
	let mut ocol: libc::c_short = 0;
	let mut mc: libc::c_char = 0;
	md_ignore_signals();
	if from_intrpt != 0 {
		orow = rogue.row;
		ocol = rogue.col;
		mc = msg_cleared;
		i = 0 as libc::c_int as libc::c_short;
		while (i as libc::c_int) < 80 as libc::c_int {
			buf[i
				as usize] = (if wmove(stdscr, 0 as libc::c_int, i as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int) as chtype
			} else {
				winch(stdscr)
			}) as libc::c_char;
			i += 1;
			i;
		}
	}
	check_message();
	message("really quit?", 1 as libc::c_int);
	if rgetchar() != 'y' as i32 {
		md_heed_signals();
		check_message();
		if from_intrpt != 0 {
			i = 0 as libc::c_int as libc::c_short;
			while (i as libc::c_int) < 80 as libc::c_int {
				if wmove(stdscr, 0 as libc::c_int, i as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					addch(buf[i as usize] as chtype);
				};
				i += 1;
				i;
			}
			msg_cleared = mc;
			wmove(stdscr, orow as libc::c_int, ocol as libc::c_int);
			ncurses::refresh();
		}
		return;
	}
	if from_intrpt != 0 {
		clean_up(byebye_string);
	}
	check_message();
	killed_by(0 as *mut object, 4);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn sf_error() {
	message("", 1);
	clean_up(b"sorry, score file is out of order\0" as *const u8 as *const libc::c_char);
}

pub unsafe fn put_scores(monster: Option<&object>, other: i16) {
	turn_into_games();

	let mut file = File::options().read(true).write(true).open(SCORE_FILE).unwrap_or_else(|_| {
		match File::options().write(true).open(SCORE_FILE) {
			Ok(file) => file,
			Err(_) => {
				message("cannot read/write/create score file", 0);
				sf_error();
				unreachable!("sf_error")
			}
		}
	});
	turn_into_user();
	xxx(true);

	let mut scores: [[u8; 80]; 10];
	let mut n_names: [[u8; 30]; 10];
	let mut ne = 0;
	let mut view_only = score_only();
	let mut found_player = None;
	for i in 0..10 {
		let score = &mut scores[i];
		let read_result = file.read(score).map_err(|_| ())
			.and_then(|n| if n == 0 {
				Ok(())
			} else if n != scores.len() {
				Err(())
			} else {
				xxxx(score, 80);
				let name = &mut n_names[i];
				file.read(name).map_err(|_| ()).and_then(|n| if n < name.len() {
					Err(())
				} else {
					xxxx(name, 30);
					Ok(())
				})
			});
		match read_result {
			Ok(_) => {}
			Err(_) => {
				sf_error();
				unreachable!()
			}
		}
		ne += 1;
		if !view_only {
			if !name_cmp(score[15..].as_ptr(), settings::login_name().as_ptr()) {
				let trimmed_score = {
					let mut x = 5;
					while score[x] == ' ' as u8 { x += 1; }
					&score[x..]
				};
				let s = lget_number(trimmed_score);
				if rogue.gold < s as i64 {
					view_only = true;
				} else {
					found_player = Some(i);
				}
			}
		}
	}
	if let Some(found_player) = found_player {
		ne -= 1;
		for i in found_player..ne {
			scores[i + 1].clone_into(&mut scores[i]);
			n_names[i + 1].clone_into(&mut n_names[i]);
		}
	}
	let mut rank = 10;
	if !view_only {
		for i in 0..ne {
			let score = &scores[i];
			let trimmed_score = {
				let mut x = 5;
				while score[x] == ' ' as u8 { x += 1; }
				&score[x..]
			};
			let s = lget_number(trimmed_score);
			if rogue.gold >= s as i64 {
				rank = i;
				break;
			}
		}
		if ne == 0 {
			rank = 0;
		} else if (ne < 10) && (rank == 10) {
			rank = ne;
		}
		if rank < 10 {
			insert_score(scores, n_names, nick_name(), rank, ne, monster, other);
			if ne < 10 {
				ne += 1;
			}
		}
		file.rewind().expect("rewind file");
	}

	clear();
	mvaddstr(3, 30, "Top  Ten  Rogueists");
	mvaddstr(8, 0, "Rank   Score   Name");

	md_ignore_signals();

	xxx(true);
	for i in 0..ne {
		let score = &mut scores[i];
		let name = &mut n_names[i];
		if i == rank {
			standout();
		}
		if i == 9 {
			score[0] = '1' as u8;
			score[1] = '0' as u8;
		} else {
			score[0] = ' ' as u8;
			score[1] = i + '1';
		}

		let buf: String = nickize(score, name);
		mvaddstr((i + 10) as i32, 0, buf.as_str());
		if rank < 10 {
			xxxx(score, 80);
			file.write(score).expect("write score");
			xxxx(name, 30);
			file.write(name).expect("write name");
		}
		if i == rank {
			standend();
		}
	}
	refresh();
	drop(file);
	message("", 0);
	clean_up(b"\n\0" as *const u8 as *const libc::c_char);
}

pub fn xxxx<const N: usize>(buf: &mut [u8; N], n: usize) {
	for i in 0..n {
		/* It does not matter if accuracy is lost during this assignment */
		let c = xxx(false) as u8;
		buf[i] ^= c;
	}
}

pub fn xxx(st: bool) -> isize {
	static FS: RwLock<(isize, isize)> = RwLock::new((0, 0));
	if st {
		let write = FS.write().unwrap();
		*write = (37, 7);
		0
	} else {
		let read = FS.read().unwrap();
		let (f, s) = *read;
		let r = (f * s + 9337) % 8887;
		FS.set((s, r));
		r
	}
}
