use ncurses::chtype;
use serde::{Deserialize, Serialize};

use crate::init::onintr;
use crate::pack::wait_for_ack;
use crate::prelude::MIN_ROW;

pub(crate) const DIALOG_ROW: usize = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerDialog {
	msg_written: String,
	msg_cleared: bool,
	cant_int: bool,
	did_int: bool,
}

impl Default for PlayerDialog {
	fn default() -> Self {
		Self {
			msg_written: String::new(),
			msg_cleared: true,
			cant_int: false,
			did_int: false,
		}
	}
}

impl PlayerDialog {
	pub fn reset(&mut self) {
		self.msg_written = String::new();
		self.msg_cleared = true;
	}
	pub fn message_cleared(&self) -> bool {
		self.msg_cleared
	}
	pub fn set_message_cleared(&mut self, value: bool) {
		self.msg_cleared = value;
	}
	pub fn clear_message(&mut self) {
		if self.msg_cleared {
			return;
		}
		ncurses::mv((MIN_ROW - 1) as i32, 0);
		ncurses::clrtoeol();
		ncurses::refresh();
		self.msg_cleared = true;
	}
	pub fn message(&mut self, msg: &str, _intrpt: i64) {
		// if !save_is_interactive {
		// 	return;
		// }
		self.cant_int = true;
		if !self.msg_cleared {
			ncurses::mvaddstr((MIN_ROW - 1) as i32, self.msg_written.len() as i32, MORE);
			ncurses::refresh();
			wait_for_ack();
			self.clear_message();
		}
		ncurses::mvaddstr((MIN_ROW - 1) as i32, 0, msg);
		ncurses::addch(chtype::from(' '));
		ncurses::refresh();
		self.msg_written = msg.to_string();
		self.msg_cleared = false;
		self.cant_int = false;
		if self.did_int {
			self.did_int = false;
			onintr();
		}
	}
	pub fn remessage(&mut self) {
		if !self.msg_written.is_empty() {
			let string = self.msg_written.to_string();
			self.message(string.as_str(), 0);
		}
	}
}

const MORE: &'static str = "-more-";


