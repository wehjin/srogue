use std::fmt::{Debug, Formatter};
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;

pub struct EventSeed(Box<dyn FnOnce(RunState) -> RunEvent + 'static>);

impl Debug for EventSeed {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("EventSeed")
	}
}

impl EventSeed {
	pub fn new(into_event: impl FnOnce(RunState) -> RunEvent + 'static) -> Self {
		Self(Box::new(into_event))
	}
	pub fn create_event(self, state: RunState) -> RunEvent {
		self.0(state)
	}
	pub fn into_redirect(self, state: RunState) -> RunStep {
		RunStep::Redirect(self.create_event(state))
	}
}