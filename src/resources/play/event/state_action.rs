use crate::resources::play::context::RunContext;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use rand::Rng;

pub trait StateAction {
	fn into_event(self) -> RunEvent;
	fn dispatch<R: Rng>(self, _ctx: &mut RunContext<R>) -> RunStep;


	fn run<R: Rng>(self, ctx: &mut RunContext<R>) -> RunState
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
