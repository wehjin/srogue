use crate::resources::play::event::game::{Dispatch, GameEventVariant};
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;

pub(super) trait RegMoveStage: Dispatch {
	fn into_reg_move_event(self) -> RegMoveEvent;
	fn into_run_event(self, state: RunState) -> RunEvent
	where
		Self: Sized,
	{
		self.into_reg_move_event().into_run_event(state)
	}
	fn into_redirect(self, state: RunState) -> RunStep
	where
		Self: Sized,
	{
		self.into_reg_move_event().into_redirect(state)
	}
}
