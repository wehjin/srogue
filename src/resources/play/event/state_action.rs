use crate::resources::play::context::RunContext;
use crate::resources::play::event::{RunEvent, RunStep};

pub trait StateAction {
	fn into_event(self) -> RunEvent;
	fn dispatch(self, _ctx: &mut RunContext) -> RunStep;
}

pub fn redirect<T: StateAction>(action: T) -> RunStep {
	RunStep::Redirect(action.into_event())
}
