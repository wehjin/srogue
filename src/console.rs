use crate::level::constants::{DCOLS, DROWS};
use crate::machdep::md_control_keybord;
use crate::render_system::backend;

pub enum ConsoleError {
	ScreenTooSmall { min_rows: usize, min_cols: usize }
}

pub struct Console {
	stopped: bool,
}

pub fn start() -> Result<Console, ConsoleError> {
	backend::set_up();
	let (rows, cols) = backend::rows_cols();
	if rows < DROWS || cols < DCOLS {
		backend::tear_down();
		return Err(ConsoleError::ScreenTooSmall { min_rows: DROWS, min_cols: DCOLS });
	}
	md_control_keybord(0);
	return Ok(Console { stopped: false });
}


impl Drop for Console {
	fn drop(&mut self) {
		assert_eq!(self.stopped, false);
		self.stopped = true;
		backend::tear_down();
		md_control_keybord(1);
	}
}
