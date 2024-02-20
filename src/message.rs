use libc::c_int;
use ncurses::{addch, chtype, curscr, mvaddstr, wrefresh};

use crate::init::GameState;
use crate::prelude::*;

pub const CANCEL: char = '\u{1b}';
pub const LIST: char = '*';

const X_CHAR: c_int = 'X' as c_int;

const CTRL_R_CHAR: c_int = 0o022 as c_int;

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

pub fn rgetchar() -> char {
	loop {
		let ch = ncurses::getch();
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
