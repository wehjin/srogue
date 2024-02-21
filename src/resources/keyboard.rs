use crate::message;
use crate::render_system::backend;

pub(crate) const CTRL_R_CHAR: char = '\u{12}';
pub(crate) const CANCEL: char = '\u{1b}';
pub(crate) const LIST: char = '*';
pub(crate) const X_CHAR: char = 'X';

pub fn rgetchar() -> char {
	loop {
		let input = backend::read_input_char();
		match input {
			CTRL_R_CHAR => {
				backend::reload_screen();
			}
			X_CHAR => {
				message::save_screen();
			}
			_ => {
				return input;
			}
		}
	}
}
