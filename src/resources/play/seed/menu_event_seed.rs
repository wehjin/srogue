use crate::resources::play::event::RunEvent;
use crate::resources::play::state::RunState;
use std::fmt::{Debug, Formatter};


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MenuInput {
	Close,
	Item(char),
}

pub struct MenuEventSeed(String, Box<dyn FnOnce(RunState, MenuInput) -> RunEvent + 'static>);

impl Debug for MenuEventSeed {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("MenuEventSeed({})", self.0))
	}
}

impl MenuEventSeed {
	pub fn new(name: impl AsRef<str>, into_event: impl FnOnce(RunState, MenuInput) -> RunEvent + 'static) -> Self {
		Self(name.as_ref().to_string(), Box::new(into_event))
	}
	pub fn create_event(self, state: RunState, menu_input: MenuInput) -> RunEvent {
		self.1(state, menu_input)
	}
}