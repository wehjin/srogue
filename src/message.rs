use crate::init::GameState;
use crate::prelude::*;
use crate::render_system::backend;
use crate::resources::dialog::DIALOG_ROW;
use crate::resources::keyboard;
use crate::resources::keyboard::{BACKSPACE_CHAR, CANCEL_CHAR};

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
		backend::set_str(insert, (0, n + 1).into());
		line.extend(insert.chars());
		backend::move_cursor((0, n + line.len() + 1).into());
		backend::push_screen();
	}
	let mut ch: char;
	loop {
		ch = keyboard::rgetchar() as u8 as char;
		if ch == '\r' || ch == '\n' || ch == CANCEL_CHAR {
			break;
		}
		if ch >= ' ' && ch <= '~' && line.len() < MAX_TITLE_LENGTH {
			if ch != ' ' || line.len() > 0 {
				line.push(ch);
				if do_echo {
					backend::set_char_at_cursor(ch);
				}
			}
		}
		if ch == BACKSPACE_CHAR && line.len() > 0 {
			if do_echo {
				backend::set_char(' ', (DIALOG_ROW, line.len() + n).into());
				backend::move_cursor((DIALOG_ROW, line.len() + n).into());
			}
			line.pop();
		}
		backend::push_screen();
	}
	game.dialog.clear_message();
	if add_blank {
		line.push(' ');
	} else {
		while let Some(' ') = line.last() {
			line.pop();
		}
	}
	if ch == CANCEL_CHAR || line.is_empty() || (line.len() == 1 && add_blank) {
		if let Some(msg) = if_cancelled {
			game.dialog.message(msg, 0);
		}
		"".to_string()
	} else {
		line.iter().collect()
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
