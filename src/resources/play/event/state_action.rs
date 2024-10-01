use crate::resources::play::context::RunContext;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;

pub trait StateAction {
	fn into_event(self) -> RunEvent;
	fn dispatch(self, _ctx: &mut RunContext) -> RunStep;


	fn run(self, ctx: &mut RunContext) -> RunState
	where
		Self: Sized,
	{
		let event = self.into_event();
		let exit_state = ctx.dispatch(event);
		exit_state
	}
}

pub fn redirect<T: StateAction>(action: T) -> RunStep {
	RunStep::Redirect(action.into_event())
}
