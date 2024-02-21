use crate::message;
use crate::render_system::backend;

pub(crate) const BACKSPACE_CHAR: char = '\u{8}';
pub(crate) const CANCEL_CHAR: char = '\u{1b}';
pub(crate) const CTRL_R_CHAR: char = '\u{12}';
pub(crate) const CTRL_W_CHAR: char = '\u{17}';

pub fn rgetchar() -> char {
	loop {
		let input = backend::read_input_char();
		match input {
			CTRL_R_CHAR => {
				backend::reload_screen();
			}
			'X' => {
				message::save_screen();
			}
			_ => {
				return input;
			}
		}
	}
}
