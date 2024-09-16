use crate::level::constants::{DCOLS, DROWS};
use crate::render_system::backend;
use crate::resources::keyboard;

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
	keyboard::enter_rogue_mode();
	Ok(Console { stopped: false })
}


impl Drop for Console {
	fn drop(&mut self) {
		assert_eq!(self.stopped, false);
		self.stopped = true;
		backend::tear_down();
		keyboard::exit_rogue_mode();
	}
}
