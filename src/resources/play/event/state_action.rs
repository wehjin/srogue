use crate::resources::play::context::RunContext;
use crate::resources::play::event::{RunEvent, RunStep};

pub trait StateAction {
	fn into_event(self) -> RunEvent;
	fn dispatch(self, _ctx: &mut RunContext) -> RunStep;

	fn into_redirect(self) -> RunStep
	where
		Self: Sized,
	{
		RunStep::Redirect(self.into_event())
	}
}

pub fn redirect<T: StateAction>(action: T) -> RunStep {
	RunStep::Redirect(action.into_event())
}
