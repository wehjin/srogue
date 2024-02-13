use crate::level::constants::{DCOLS, DROWS};
use crate::machdep::{md_control_keybord};

pub enum ConsoleError {
	ScreenTooSmall { min_rows: usize, min_cols: usize }
}

pub struct Console {
	stopped: bool,
}

pub fn start() -> Result<Console, ConsoleError> {
	ncurses::initscr();
	if ncurses::LINES() < DROWS as i32 || ncurses::COLS() < DCOLS as i32 {
		ncurses::endwin();
		return Err(ConsoleError::ScreenTooSmall { min_rows: DROWS, min_cols: DCOLS });
	}
	ncurses::cbreak();
	ncurses::noecho();
	ncurses::nonl();
	md_control_keybord(0);
	return Ok(Console { stopped: false });
}

impl Drop for Console {
	fn drop(&mut self) {
		assert_eq!(self.stopped, false);
		self.stopped = true;
		// Disable for now since it erases errors messages. We should discriminate
		// between normal and error exits.
		// {
		// 	ncurses::wmove(ncurses::stdscr(), (DROWS - 1) as i32, 0);
		// 	ncurses::refresh();
		// }
		ncurses::endwin();
		md_control_keybord(1);
	}
}
