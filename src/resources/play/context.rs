use crate::resources::dice::roll_chance;
use crate::resources::play::event::RunEvent;
use crate::resources::play::state::RunState;
use crate::resources::play::{dispatch, TextConsole};
use rand::Rng;

pub struct RunContext<R: Rng> {
	// TODO Move rng into RunState so that we can split dispatch into step and walk.
	pub rng: Box<R>,
	pub console: Box<dyn TextConsole>,
}
impl<R: Rng> RunContext<R> { // Constructor
	pub fn new(rng: R, console: impl TextConsole + 'static) -> Self {
		Self {
			rng: Box::new(rng),
			console: Box::new(console),
		}
	}
}
impl<R: Rng> RunContext<R> { // Accessors
	pub fn rng(&mut self) -> &mut R {
		&mut self.rng
	}
	pub fn console(&mut self) -> &mut impl TextConsole {
		&mut self.console
	}
}
impl<R: Rng> RunContext<R> { // Utilities
	pub fn roll_chance(&mut self, chance: usize) -> bool {
		roll_chance(chance, self.rng())
	}
	pub fn dispatch(&mut self, event: RunEvent) -> RunState {
		let out_state = dispatch(event, self);
		out_state
	}
}