use std::{fs, io, thread};
use std::error::Error;
use std::os::macos::fs::MetadataExt;
use std::time::Duration;

use chrono::{Datelike, DateTime, Timelike, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RogueTime {
	pub year: i32,
	pub month: u32,
	pub day: u32,
	pub hour: u32,
	pub minute: u32,
	pub second: u32,
}

impl RogueTime {
	pub fn add_seconds(&self, seconds: u64) -> Self {
		let utc: DateTime<Utc> = Utc.with_ymd_and_hms(self.year, self.month, self.day, self.hour, self.minute, self.second).unwrap();
		Self::from(utc + Duration::from_secs(seconds))
	}
}

impl From<DateTime<Utc>> for RogueTime {
	fn from(value: DateTime<Utc>) -> Self {
		RogueTime {
			year: value.year(),
			month: value.month(),
			day: value.day(),
			hour: value.hour(),
			minute: value.minute(),
			second: value.second(),
		}
	}
}

/* md_slurp:
 *
 * This routine throws away all keyboard input that has not
 * yet been read.  It is used to get rid of input that the user may have
 * typed-ahead.
 *
 * This function is not necessary, so it may be stubbed.  The might cause
 * message-line output to flash by because the game has continued to read
 * input without waiting for the user to read the message.  Not such a
 * big deal.
 */
pub fn md_slurp() {}

pub fn md_control_keybord(_mode: libc::c_short) {
	// Stubbing this out allows tty driver so steal some commands like ^Y.
	// See machdep.c for more details
}

//
// fn sig_on_intr(_: c_int) { onintr(); }
//
// // fn sig_on_quit(_: c_int) {
// // 	byebye(true, unimplemented!("Acquire max_level for quit"));
// // }
//
// fn sig_on_hup(_: c_int) {
// 	unimplemented!("Acquire game state for interrupt");
// 	// save_is_interactive = false;
// 	// crate::save::save_into_file(ERROR_FILE, game);
// 	// clean_up("");
// }


pub fn md_heed_signals() {
	// signal(SIGINT, sig_on_intr as fn(c_int) as *mut c_void as sighandler_t);
	// signal(SIGQUIT, sig_on_quit as fn(c_int) as *mut c_void as sighandler_t);
	// signal(SIGHUP, sig_on_hup as fn(c_int) as *mut c_void as sighandler_t);
}

pub fn md_ignore_signals() {
	// signal(SIGQUIT, SIG_IGN);
	// signal(SIGINT, SIG_IGN);
	// signal(SIGHUP, SIG_IGN);
	// signal(SIGTSTP, SIG_IGN);
}

pub fn md_get_file_id(file_path: &str) -> io::Result<u64> {
	fs::metadata(file_path).map(|it| it.st_ino())
}

pub fn md_link_count(file_path: &str) -> io::Result<u64> {
	fs::metadata(file_path).map(|it| it.st_nlink())
}

pub fn get_current_time() -> RogueTime {
	let utc_now = Utc::now();
	RogueTime::from(utc_now)
}

pub fn get_file_modification_time(file_name: &str) -> Result<RogueTime, Box<dyn Error>> {
	let metadata = fs::metadata(file_name)?;
	let system_time = metadata.modified()?;
	let utc = DateTime::<Utc>::from(system_time);
	Ok(RogueTime::from(utc))
}

pub fn delete_file(file_name: &str) -> bool {
	let result = fs::remove_file(file_name);
	result.is_ok()
}


pub fn get_login_name() -> Option<String> {
	let username = whoami::username();
	if username.is_empty() {
		None
	} else {
		Some(username)
	}
}


pub fn md_sleep(nsecs: i64) {
	thread::sleep(Duration::from_nanos(nsecs as u64));
}
