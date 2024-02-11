#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments)]

use libc::{c_int};
use ncurses::{addch, chtype, clrtoeol, curscr, mvaddstr, wrefresh};
use crate::level::constants::DROWS;
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::stat_const::{STAT_ARMOR, STAT_EXP, STAT_GOLD, STAT_HP, STAT_HUNGER, STAT_LABEL, STAT_LEVEL, STAT_STRENGTH};

pub static mut msg_written: String = String::new();
pub static mut msg_cleared: bool = true;
pub static mut hunger_str: String = String::new();

pub unsafe fn message(msg: &str, intrpt: i64) {
	if !save_is_interactive {
		return;
	}
	if intrpt != 0 {
		interrupted = true;
		md_slurp();
	}
	cant_int = true;

	if !msg_cleared {
		mvaddstr((MIN_ROW - 1) as i32, msg_written.len() as i32, MORE);
		ncurses::refresh();
		wait_for_ack();
		check_message();
	}
	mvaddstr((MIN_ROW - 1) as i32, 0, msg);
	addch(chtype::from(' '));
	ncurses::refresh();
	msg_written = msg.to_string();
	msg_cleared = false;
	cant_int = false;
	if did_int {
		did_int = false;
		onintr();
	}
}

pub unsafe extern "C" fn remessage() {
	if !msg_written.is_empty() {
		message(&msg_written, 0);
	}
}

pub unsafe fn check_message() {
	if msg_cleared {
		return;
	}
	ncurses::mv((MIN_ROW - 1) as i32, 0);
	clrtoeol();
	ncurses::refresh();
	msg_cleared = true;
}

pub const CANCEL: char = '\u{1b}';
pub const LIST: char = '*';

pub unsafe fn get_input_line<T: AsRef<str>>(prompt: &str, insert: Option<T>, if_cancelled: Option<&str>, add_blank: bool, do_echo: bool) -> String {
	message(prompt, 0);

	let mut line: Vec<char> = Vec::new();
	let n = prompt.len();
	if let Some(insert) = insert {
		let insert = insert.as_ref();
		mvaddstr(0, (n + 1) as i32, insert);
		line.extend(insert.chars());
		ncurses::mv(0, (n + line.len() + 1) as i32);
		ncurses::refresh();
	}
	let mut ch: char;
	loop {
		ch = rgetchar() as u8 as char;
		if ch == '\r' || ch == '\n' || ch == CANCEL {
			break;
		}
		if ch >= ' ' && ch <= '~' && line.len() < MAX_TITLE_LENGTH {
			if ch != ' ' || line.len() > 0 {
				line.push(ch);
				if do_echo {
					addch(ch as chtype);
				}
			}
		}
		const BACKSPACE: char = '\u{8}';
		if ch == BACKSPACE && line.len() > 0 {
			if do_echo {
				ncurses::mvaddch(0, (line.len() + n) as i32, ' ' as chtype);
				ncurses::mv((MIN_ROW - 1) as i32, (line.len() + n) as i32);
			}
			line.pop();
		}
		ncurses::refresh();
	}
	check_message();
	if add_blank {
		line.push(' ');
	} else {
		while let Some(' ') = line.last() {
			line.pop();
		}
	}
	if ch == CANCEL || line.is_empty() || (line.len() == 1 && add_blank) {
		if let Some(msg) = if_cancelled {
			message(msg, 0);
		}
		"".to_string()
	} else {
		line.iter().collect()
	}
}

const X_CHAR: c_int = 'X' as c_int;
const CTRL_R_CHAR: c_int = 0o022 as c_int;

pub fn rgetchar() -> char {
	loop {
		let ch = { unsafe { libc::getchar() } };
		match ch {
			CTRL_R_CHAR => {
				wrefresh(curscr());
			}
			X_CHAR => {
				save_screen();
			}
			_ => {
				return ch as u8 as char;
			}
		}
	}
}

pub unsafe fn print_stats(stat_mask: usize, player: &mut Player) {
	const STATS_ROW: i32 = DROWS as i32 - 1;
	let label = if stat_mask & STAT_LABEL != 0 { true } else { false };

	if stat_mask & STAT_LEVEL != 0 {
		if label {
			mvaddstr(STATS_ROW, 0, "Level: ");
		}
		let s = format!("{}", player.cur_depth);
		mvaddstr(STATS_ROW, 7, &s);
		pad(&s, 2);
	}
	if stat_mask & STAT_GOLD != 0 {
		if label {
			player.maintain_max_gold();
			mvaddstr(STATS_ROW, 10, "Gold: ");
		}
		let s = format!("{}", player.gold());
		mvaddstr(STATS_ROW, 16, &s);
		pad(&s, 6);
	}
	if stat_mask & STAT_HP != 0 {
		if label {
			mvaddstr(STATS_ROW, 23, "Hp: ");
			if player.rogue.hp_max > 800 {
				player.rogue.hp_current -= player.rogue.hp_max - 800;
				player.rogue.hp_max = 800;
			}
		}
		let s = format!("{}({})", player.rogue.hp_current, player.rogue.hp_max);
		mvaddstr(STATS_ROW, 27, &s);
		pad(&s, 8);
	}
	if stat_mask & STAT_STRENGTH != 0 {
		if label {
			mvaddstr(STATS_ROW, 36, "Str: ");
		}
		if player.rogue.str_max > MAX_STRENGTH {
			player.rogue.str_current -= player.rogue.str_max - MAX_STRENGTH;
			player.rogue.str_max = MAX_STRENGTH;
		}
		let s = format!("{}({})", player.rogue.str_current + add_strength, player.rogue.str_max);
		mvaddstr(STATS_ROW, 41, &s);
		pad(&s, 6);
	}
	if stat_mask & STAT_ARMOR != 0 {
		if label {
			mvaddstr(STATS_ROW, 48, "Arm: ");
		}
		player.maintain_armor_max_enchant();
		let s = format!("{}", get_armor_class(player.armor()));
		mvaddstr(STATS_ROW, 53, &s);
		pad(&s, 2);
	}
	if stat_mask & STAT_EXP != 0 {
		if label {
			mvaddstr(STATS_ROW, 56, "Exp: ");
		}
		/*  Max exp taken care of in add_exp() */
		let s = format!("{}/{}", player.rogue.exp, player.rogue.exp_points);
		mvaddstr(STATS_ROW, 61, &s);
		pad(&s, 11);
	}
	if stat_mask & STAT_HUNGER != 0 {
		mvaddstr(STATS_ROW, 73, &hunger_str);
		clrtoeol();
	}
	ncurses::refresh();
}

fn pad(s: &str, n: usize) {
	for _ in s.len()..n {
		addch(' ' as chtype);
	}
}

pub fn save_screen() {
	// TODO
	// FILE *fp;
	// short i, j, row, col;
	// char buf[DCOLS+2];
	// boolean found_non_blank;
	//
	//
	// if ((fp = fopen("player.rogue.screen", "w")) != NULL) {
	// 	for (i = 0; i < DROWS; i++) {
	// 		found_non_blank = 0;
	// 		for (j = (DCOLS - 1); j >= 0; j--) {
	// 			buf[j] = mvinch(i, j);
	// 			if (!found_non_blank) {
	// 				if ((buf[j] != ' ') || (j == 0)) {
	// 					buf[j + ((j == 0) ? 0 : 1)] = 0;
	// 					found_non_blank = 1;
	// 				}
	// 			}
	// 		}
	// 		fputs(buf, fp);
	// 		putc('\n', fp);
	// 	}
	// 	fclose(fp);
	// } else {
	// 	sound_bell();
	// }
}

pub fn sound_bell() {
	// TODO
	// putchar(7);
	// fflush(stdout);
}

#[no_mangle]
pub unsafe extern "C" fn is_digit(ch: libc::c_short) -> libc::c_char {
	return (ch as i64 >= '0' as i64 && ch as i64 <= '9' as i64) as libc::c_char;
}
