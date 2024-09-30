use crate::motion::{reg_move, MoveResult, RogueEnergy};
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use rand::Rng;

pub trait StepEvent: Into<RunEvent> {
	fn step<R: Rng>(self, _ctx: &mut RunContext<R>) -> RunStep;
}

#[derive(Debug)]
pub struct RegMoveEvent(pub RunState, pub Option<MoveResult>);

impl StepEvent for RegMoveEvent {
	fn step<R: Rng>(self, _ctx: &mut RunContext<R>) -> RunStep {
		let Self(mut state, move_result) = self;
		match reg_move(&mut state) {
			RogueEnergy::Starved => {
				// TODO Might need to do something like killed_by here instead.
				state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::StoppedOnSomething));
				RunStep::Exit(state)
			}
			RogueEnergy::Fainted => {
				state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::StoppedOnSomething));
				RunStep::Effect(state, RunEffect::AwaitPlayerMove)
			}
			RogueEnergy::Normal => if state.as_health().confused.is_active() {
				state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::StoppedOnSomething));
				RunStep::Effect(state, RunEffect::AwaitPlayerMove)
			} else {
				state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::Moved));
				RunStep::Effect(state, RunEffect::AwaitPlayerMove)
			},
		}
	}
}

impl Into<RunEvent> for RegMoveEvent {
	fn into(self) -> RunEvent {
		RunEvent::RegisterMove(self)
	}
}
