use crate::resources::play::event::state_action::StateAction;
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
	pub fn run_await_exit(&mut self, event: RunEvent) -> RunState {
		let exit_state = dispatch(event, self);
		exit_state
	}

	pub fn run_action_await_exit(&mut self, action: impl StateAction) -> RunState {
		let action_event = action.into_event();
		let exit_state = self.run_await_exit(action_event);
		exit_state
	}
}