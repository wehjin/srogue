use crate::resources::play::context::RunContext;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::{RunEvent, RunEventVariant, RunStep};
use crate::resources::play::state::RunState;

#[derive(Debug)]
pub enum GameEvent {
	RegMoveTask(RegMoveEvent),
}
impl RunEventVariant for GameEvent {
	fn into_run_event(self, state: RunState) -> RunEvent {
		RunEvent::Game(state, self)
	}
}
impl Dispatch for GameEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			GameEvent::RegMoveTask(task) => task.dispatch(state, ctx)
		}
	}
}

pub trait GameEventVariant: Dispatch {
	fn into_game_event(self) -> GameEvent;
	fn into_redirect(self, state: RunState) -> RunStep
	where
		Self: Sized,
	{
		self.into_game_event().into_redirect(state)
	}
}

pub trait Dispatch {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep;
}