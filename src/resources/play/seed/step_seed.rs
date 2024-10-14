use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use std::fmt::{Debug, Formatter};

pub struct StepSeed(String, Box<dyn FnOnce(RunState) -> RunStep + 'static>);

impl Debug for StepSeed {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("StepSeed({})", self.0))
	}
}

impl StepSeed {
	pub fn new(name: impl AsRef<str>, seed: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self(name.as_ref().to_string(), Box::new(seed))
	}

	pub fn into_step(self, state: RunState) -> RunStep {
		self.1(state)
	}
}