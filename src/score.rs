#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{RwLock};
use ncurses::{addch, clear, mvaddstr, refresh, standend, standout, waddnstr};
use settings::{score_only, show_skull};
use crate::prelude::*;
use crate::{settings, turn_into_games, turn_into_user};
use crate::prelude::ending::Ending;
use crate::settings::{login_name, nick_name};

pub const SCORE_FILE: &'static str = "/usr/games/rogue.scores";

pub unsafe fn killed_by(ending: Ending) {
	md_ignore_signals();
	if ending != Ending::Quit {
		rogue.gold = ((rogue.gold as f64 * 9.0) / 10.0) as isize;
	}
	let mut how_ended = match ending {
		Ending::Monster(monster) => {
			let name = mon_real_name(&monster);
			let article = if is_vowel(name.chars().nth(0).unwrap()) { "an" } else { "a" };
			&format!("Killed by {} {}", article, name)
		}
		Ending::Hypothermia => "died of hypothermia",
		Ending::Starvation => "died of starvation",
		Ending::PoisonDart => "killed by a dart",
		Ending::Quit => "quit",
		Ending::Win => ""
	}.to_string();
	how_ended += &format!(" with {} gold", rogue.gold);
	if ending.is_monster() && show_skull() {
		clear();
		mvaddstr(4, 32, "__---------__");
		mvaddstr(5, 30, "_~             ~_");
		mvaddstr(6, 29, "/                 \\");
		mvaddstr(7, 28, "~                   ~");
		mvaddstr(8, 27, "/                     \\");
		mvaddstr(9, 27, "|    XXXX     XXXX    |");
		mvaddstr(10, 27, "|    XXXX     XXXX    |");
		mvaddstr(11, 27, "|    XXX       XXX    |");
		mvaddstr(12, 28, "\\         @         /");
		mvaddstr(13, 29, "--\\     @@@     /--");
		mvaddstr(14, 30, "| |    @@@    | |");
		mvaddstr(15, 30, "| |           | |");
		mvaddstr(16, 30, "| vvVvvvvvvvVvv |");
		mvaddstr(17, 30, "|  ^^^^^^^^^^^  |");
		mvaddstr(18, 31, "\\_           _/");
		mvaddstr(19, 33, "~---------~");
		let skull_name = if let Some(nick_name) = nick_name() {
			nick_name
		} else {
			login_name()
		};
		center(21, skull_name);
		center(22, &how_ended);
	} else {
		message(&how_ended, 0);
	}
	message("", 0);
	put_scores(Some(ending));
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
	killed_by(Ending::Quit);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn put_scores(ending: Option<Ending>) {
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
			insert_score(scores, n_names, nick_name(), rank, ne, ending);
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

pub fn center(row: i64, msg: &str) {
	let margin = (DCOLS - msg.len()) / 2;
	mvaddstr(row as i32, margin as i32, msg);
}

pub unsafe fn sf_error() {
	message("", 1);
	clean_up("sorry, score file is out of order");
}
