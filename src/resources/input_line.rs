use crate::prelude::MAX_TITLE_LENGTH;
use crate::render_system::backend;
use crate::resources::diary::DIALOG_ROW;
use crate::resources::diary::Diary;
use crate::resources::keyboard::{BACKSPACE_CHAR, CANCEL_CHAR};
use crate::resources::{diary, keyboard};

pub fn get_input_line<T: AsRef<str>>(prompt: &str, insert: Option<T>, if_cancelled: Option<&str>, add_blank: bool, do_echo: bool, diary: &mut Diary) -> String {
	diary::show_prompt(&format!("{prompt} "), diary);
	if let Some(answer) = get_answer(prompt.len() + 1, do_echo, insert) {
		match add_blank {
			true => format!("{answer} "),
			false => answer
		}
	} else {
		if let Some(msg) = if_cancelled {
			diary.add_entry(msg);
		}
		"".to_string()
	}
}

fn get_answer<T: AsRef<str>>(start_col: usize, do_echo: bool, insert: Option<T>) -> Option<String> {
	let mut answer = insert.map(|s| s.as_ref().to_string()).unwrap_or("".to_string());
	backend::set_str(&answer, (DIALOG_ROW, start_col).into());
	backend::push_screen();
	loop {
		let ch = keyboard::rgetchar();
		if ch == '\r' || ch == '\n' || ch == CANCEL_CHAR {
			answer = answer.trim().to_string();
			return if ch != CANCEL_CHAR && answer.len() > 0 {
				Some(answer)
			} else {
				None
			};
		}
		if !ch.is_control() && answer.len() < MAX_TITLE_LENGTH {
			if ch != ' ' || answer.len() > 0 {
				answer.push(ch);
				if do_echo {
					backend::set_char_at_cursor(ch);
				}
			}
		}
		if ch == BACKSPACE_CHAR && answer.len() > 0 {
			answer.pop();
			if do_echo {
				let erase_index = start_col + answer.len();
				backend::set_char(' ', (DIALOG_ROW, erase_index).into());
				backend::move_cursor((DIALOG_ROW, erase_index).into());
			}
		}
		backend::push_screen();
	}
}
