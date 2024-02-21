use serde::{Deserialize, Serialize};

use crate::init::onintr;
use crate::pack::wait_for_ack;
use crate::render_system::backend;

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
		backend::set_full_row("", DIALOG_ROW);
		backend::push_screen();
		self.msg_cleared = true;
	}
	pub fn message(&mut self, msg: &str, _intrpt: i64) {
		// if !save_is_interactive {
		// 	return;
		// }
		self.cant_int = true;
		if !self.msg_cleared {
			backend::set_str(MORE, (DIALOG_ROW, self.msg_written.len()).into());
			backend::push_screen();
			wait_for_ack();
			self.clear_message();
		}
		backend::set_str(msg, (DIALOG_ROW, 0).into());
		backend::set_char_at_cursor(' ');
		backend::push_screen();
		self.msg_written = msg.to_string();
		self.msg_cleared = false;
		self.cant_int = false;
		if self.did_int {
			self.did_int = false;
			onintr();
		}
	}
	pub fn re_message(&mut self) {
		if !self.msg_written.is_empty() {
			let string = self.msg_written.to_string();
			self.message(string.as_str(), 0);
		}
	}
}

const MORE: &'static str = "-more-";


