use crate::resources::play::event::RunEvent;
use crate::resources::play::state::RunState;
use crate::resources::play::{dispatch, TextConsole};
use rand::Rng;

pub struct RunContext<R: Rng> {
	pub rng: Box<R>,
	pub console: Box<dyn TextConsole>,
}

impl<R: Rng> RunContext<R> {
	pub fn rng(&mut self) -> &mut R {
		&mut self.rng
	}
	pub fn console(&mut self) -> &mut impl TextConsole {
		&mut self.console
	}
	pub fn dispatch(&mut self, event: RunEvent) -> RunState {
		let out_state = dispatch(event, self);
		out_state
	}

	pub fn new(rng: R, console: impl TextConsole + 'static) -> Self {
		Self {
			rng: Box::new(rng),
			console: Box::new(console),
		}
	}
}