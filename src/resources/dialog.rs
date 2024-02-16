use ncurses::chtype;
use serde::{Deserialize, Serialize};

use crate::init::{cant_int, did_int, onintr, save_is_interactive};
use crate::machdep::md_slurp;
use crate::pack::wait_for_ack;
use crate::play::interrupted;
use crate::prelude::MIN_ROW;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerDialog {
	msg_written: String,
	msg_cleared: bool,
}

impl Default for PlayerDialog {
	fn default() -> Self { Self { msg_written: String::new(), msg_cleared: true } }
}

impl PlayerDialog {
	pub fn reset(&mut self) {
		self.msg_written = String::new();
		self.msg_cleared = true;
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
	pub unsafe fn message(&mut self, msg: &str, intrpt: i64) {
		if !save_is_interactive {
			return;
		}
		if intrpt != 0 {
			interrupted = true;
			md_slurp();
		}
		cant_int = true;
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
		cant_int = false;
		if did_int {
			did_int = false;
			onintr();
		}
	}
}

const MORE: &'static str = "-more-";


