use crate::resources::play::event::RunEvent;
use crate::resources::play::state::RunState;
use crate::resources::play::{dispatch, TextConsole};

pub struct RunContext {
	pub console: Box<dyn TextConsole>,
}
impl RunContext { // Constructor
	pub fn new(console: impl TextConsole + 'static) -> Self {
		Self {
			console: Box::new(console),
		}
	}
}
impl RunContext { // Accessors
	pub fn console(&mut self) -> &mut impl TextConsole {
		&mut self.console
	}
}
impl RunContext { // Utilities
	pub fn dispatch(&mut self, event: RunEvent) -> RunState {
		let out_state = dispatch(event, self);
		out_state
	}
}