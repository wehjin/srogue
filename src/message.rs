#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::c_int;
use ncurses::{addch, chtype, clrtoeol, curscr, mvaddstr, wrefresh};

use crate::components::hunger::HungerLevel;
use crate::init::GameState;
use crate::level::constants::DROWS;
use crate::objects::get_armor_class;
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::stat_const::{STAT_ARMOR, STAT_EXP, STAT_GOLD, STAT_HP, STAT_HUNGER, STAT_LABEL, STAT_LEVEL, STAT_STRENGTH};

pub const CANCEL: char = '\u{1b}';
pub const LIST: char = '*';

pub fn get_input_line<T: AsRef<str>>(
	prompt: &str,
	insert: Option<T>,
	if_cancelled: Option<&str>,
	add_blank: bool,
	do_echo: bool,
	game: &mut GameState,
) -> String {
	game.dialog.message(prompt, 0);

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
	game.dialog.clear_message();
	if add_blank {
		line.push(' ');
	} else {
		while let Some(' ') = line.last() {
			line.pop();
		}
	}
	if ch == CANCEL || line.is_empty() || (line.len() == 1 && add_blank) {
		if let Some(msg) = if_cancelled {
			game.dialog.message(msg, 0);
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

pub fn print_stats(stat_mask: usize, player: &mut Player) {
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
		let strength = player.buffed_strength();
		let s = format!("{}({})", strength, player.rogue.str_max);
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
		let hunger_str = if player.hunger == HungerLevel::Normal { "" } else { player.hunger.as_str() };
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
