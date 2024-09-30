use crate::resources::play::event::RunEvent;
use crate::resources::play::state::RunState;
use std::fmt::{Debug, Formatter};

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
	pub fn into_event(self, state: RunState) -> RunEvent {
		self.0(state)
	}
}