use crate::resources::play::context::RunContext;
use crate::resources::play::event::{RunEvent, RunStep};
use rand::Rng;

pub trait StateAction {
	fn into_event(self) -> RunEvent;
	fn dispatch<R: Rng>(self, _ctx: &mut RunContext<R>) -> RunStep;
}
