use crate::message;
use crate::render_system::backend;
use libc::{c_uchar, STDIN_FILENO};
use std::sync::RwLock;
use termios::os::target::VDSUSP;
use termios::{tcsetattr, Termios, TCSANOW, VERASE, VSTART, VSTOP, VSUSP};

pub(crate) const BACKSPACE_CHAR: char = '\u{8}';
pub(crate) const CANCEL_CHAR: char = '\u{1b}';
pub(crate) const CTRL_A: char = '\x01';
pub(crate) const CTRL_B: char = '\x02';
pub(crate) const CTRL_C: char = '\x03';
pub(crate) const CTRL_H: char = '\x08';
pub(crate) const CTRL_I: char = '\x09';
pub(crate) const CTRL_J: char = '\x0a';
pub(crate) const CTRL_K: char = '\x0b';
pub(crate) const CTRL_L: char = '\x0c';
pub(crate) const CTRL_M: char = '\x0d';
pub(crate) const CTRL_N: char = '\x0e';
pub(crate) const CTRL_O: char = '\x0f';
pub(crate) const CTRL_P: char = '\x10';
pub(crate) const CTRL_R_CHAR: char = '\u{12}';
pub(crate) const CTRL_S: char = '\x13';
pub(crate) const CTRL_T: char = '\x14';
pub(crate) const CTRL_U: char = '\x15';
pub(crate) const CTRL_W: char = '\u{17}';
pub(crate) const CTRL_Y: char = '\x19';

#[derive(Debug, Copy, Clone)]
enum TtyState {
	Normal,
	Rogue { normal_ios: Termios },
}
static TTY_STATE: RwLock<TtyState> = RwLock::new(TtyState::Normal);

/// This routine was formerly named md_control_keyboard and is much like
/// md_cbreak_no_echo_nonl() in the original.  It sets up the
/// keyboard for appropriate input.  Specifically, it prevents the tty driver
/// from stealing characters.  For example, ^Y is needed as a command
/// character, but the tty driver intercepts it for another purpose.  Any
/// such behavior should be stopped.  This routine could be avoided if
/// we used RAW mode instead of CBREAK.  But RAW mode does not allow the
/// generation of keyboard signals, which the program uses.
pub fn enter_rogue_mode() {

	// md_control_keyboard ignores SIGTSTP here. Maybe do that here as well when we deal with signals.
	// signal(SIGTSTP,SIG_IGN);

	let state = *TTY_STATE.read().expect("TTY read lock failed");
	match state {
		TtyState::Normal => {
			let normal_ios = Termios::from_fd(STDIN_FILENO).expect("Could not read normal_ios");
			let mut rogue_ios = normal_ios;
			rogue_ios.c_cc[VERASE] = (-1isize) as c_uchar;
			rogue_ios.c_cc[VSTART] = (-1isize) as c_uchar;
			rogue_ios.c_cc[VSTOP] = (-1isize) as c_uchar;
			rogue_ios.c_cc[VSUSP] = (-1isize) as c_uchar;
			rogue_ios.c_cc[VDSUSP] = (-1isize) as c_uchar;
			tcsetattr(STDIN_FILENO, TCSANOW, &rogue_ios).expect("Could not set rogue_ios");
			let mut write = TTY_STATE.write().expect("TTY write lock failed");
			*write = TtyState::Rogue { normal_ios };
		}
		TtyState::Rogue { .. } => {}
	}
}

/// Formerly named md_control_keyboard
pub fn exit_rogue_mode() {

	// md_control_keyboard ignores SIGTSTP here. Maybe do that here as well when we deal with signals.
	// signal(SIGTSTP,SIG_IGN);

	let state = *TTY_STATE.read().expect("TTY read lock failed");
	match state {
		TtyState::Normal => {}
		TtyState::Rogue { normal_ios: down_ios } => {
			tcsetattr(STDIN_FILENO, TCSANOW, &down_ios).expect("Could not write normal_ios");
			let mut write = TTY_STATE.write().expect("TTY write lock failed");
			*write = TtyState::Normal;
		}
	}
}

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
