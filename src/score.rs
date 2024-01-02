#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{RwLock};
use libc::{sprintf, strcat, strcpy, strlen};
use ncurses::{addch, clear, mvaddstr, refresh, standend, standout, waddnstr};
use settings::score_only;
use crate::prelude::*;
use crate::{settings, turn_into_games, turn_into_user};
use crate::settings::{login_name, nick_name, SETTINGS};

pub const SCORE_FILE: &'static str = "/usr/games/rogue.scores";

#[no_mangle]
pub unsafe extern "C" fn killed_by(
	mut monster: *mut object,
	mut other: libc::c_short,
) -> i64 {
	let mut buf: [libc::c_char; 80] = [0; 80];
	md_ignore_signals();
	if other as i64 != 4 as i64 {
		rogue
			.gold = rogue.gold * 9 as i64 as libc::c_long
			/ 10 as i64 as libc::c_long;
	}
	if other != 0 {
		match other as i64 {
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
				.offset(((*monster).ichar as i64 - 'A' as i32) as isize))
				.offset(0 as i64 as isize) as i64,
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
				.offset(((*monster).ichar as i64 - 'A' as i32) as isize),
		);
	}
	strcat(buf.as_mut_ptr(), b" with \0" as *const u8 as *const libc::c_char);
	sprintf(
		buf.as_mut_ptr().offset(strlen(buf.as_mut_ptr()) as isize),
		b"%ld gold\0" as *const u8 as *const libc::c_char,
		rogue.gold,
	);
	if other == 0 && SETTINGS.get().show_skull as i64 != 0 {
		ncurses::wclear(ncurses::stdscr());
		if ncurses::wmove(ncurses::stdscr(), 4 as i64, 32 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"__---------__\0" as *const u8 as *const libc::c_char,
				-(1),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 5 as i64, 30 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"_~             ~_\0" as *const u8 as *const libc::c_char,
				-(1),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 6 as i64, 29 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"/                 \\\0" as *const u8 as *const libc::c_char,
				-(1),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 7 as i64, 28 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"~                   ~\0" as *const u8 as *const libc::c_char,
				-(1),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 8 as i64, 27 as i64) == -(1) {
			-(1);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"/                     \\\0" as *const u8 as *const libc::c_char,
				-(1),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 9 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"|    XXXX     XXXX    |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 10 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"|    XXXX     XXXX    |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 11 as libc::c_int, 27 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"|    XXX       XXX    |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 12 as libc::c_int, 28 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"\\         @         /\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 13 as libc::c_int, 29 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"--\\     @@@     /--\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 14 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"| |    @@@    | |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 15 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"| |           | |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 16 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"| vvVvvvvvvvVvv |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 17 as libc::c_int, 30 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"|  ^^^^^^^^^^^  |\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 18 as libc::c_int, 31 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
				b"\\_           _/\0" as *const u8 as *const libc::c_char,
				-(1 as libc::c_int),
			);
		};
		if ncurses::wmove(ncurses::stdscr(), 19 as libc::c_int, 33 as libc::c_int) == -(1 as libc::c_int) {
			-(1 as libc::c_int);
		} else {
			waddnstr(
				ncurses::stdscr(),
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
	ncurses::wclear(ncurses::stdscr());
	if ncurses::wmove(ncurses::stdscr(), 10 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b"@   @  @@@   @   @      @  @  @   @@@   @   @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 11 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b" @ @  @   @  @   @      @  @  @  @   @  @@  @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 12 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b"  @   @   @  @   @      @  @  @  @   @  @ @ @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 13 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b"  @   @   @  @   @      @  @  @  @   @  @  @@\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 14 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b"  @    @@@    @@@        @@ @@    @@@   @   @   @\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 17 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b"Congratulations,  you have  been admitted  to  the\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 18 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
			b"Fighters' Guild.   You return home,  sell all your\0" as *const u8
				as *const libc::c_char,
			-(1 as libc::c_int),
		);
	};
	if ncurses::wmove(ncurses::stdscr(), 19 as libc::c_int, 11 as libc::c_int) == -(1 as libc::c_int) {
		-(1 as libc::c_int);
	} else {
		waddnstr(
			ncurses::stdscr(),
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
				as usize] = (if ncurses::wmove(ncurses::stdscr(), 0 as libc::c_int, i as libc::c_int)
				== -(1 as libc::c_int)
			{
				-(1 as libc::c_int) as ncurses::chtype
			} else {
				ncurses::winch(ncurses::stdscr())
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
				if ncurses::wmove(ncurses::stdscr(), 0 as libc::c_int, i as libc::c_int)
					== -(1 as libc::c_int)
				{
					-(1 as libc::c_int);
				} else {
					addch(buf[i as usize] as ncurses::chtype);
				};
				i += 1;
				i;
			}
			msg_cleared = mc;
			ncurses::wmove(ncurses::stdscr(), orow as libc::c_int, ocol as libc::c_int);
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
	clean_up("sorry, score file is out of order");
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
				if rogue.gold < s as i32 {
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
			if rogue.gold >= s as i32 {
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
	clean_up("\n");
}

pub fn is_vowel(ch: char) -> bool {
	match ch {
		'a' | 'e' | 'i' | 'o' | 'u' => true,
		_ => false
	}
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
