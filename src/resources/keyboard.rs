use crate::message;
use crate::render_system::backend;

pub(crate) const BACKSPACE_CHAR: char = '\u{8}';
pub(crate) const CANCEL_CHAR: char = '\u{1b}';
pub(crate) const CTRL_B: char = '\x02';
pub(crate) const CTRL_H: char = '\x08';
pub(crate) const CTRL_I: char = '\x09';
pub(crate) const CTRL_J: char = '\x0a';
pub(crate) const CTRL_K: char = '\x0b';
pub(crate) const CTRL_L: char = '\x0c';
pub(crate) const CTRL_N: char = '\x0e';
pub(crate) const CTRL_P: char = '\x10';
pub(crate) const CTRL_R_CHAR: char = '\u{12}';
pub(crate) const CTRL_S: char = '\x13';
pub(crate) const CTRL_U: char = '\x15';
pub(crate) const CTRL_W: char = '\u{17}';
pub(crate) const CTRL_Y: char = '\x19';

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
