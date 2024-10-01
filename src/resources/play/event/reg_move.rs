use crate::motion::{reg_move, MoveResult, RogueEnergy};
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use rand::Rng;

#[derive(Debug)]
pub struct RegMove(pub RunState, pub Option<MoveResult>);

impl StateAction for RegMove {
	fn into_event(self) -> RunEvent {
		RunEvent::RegisterMove(self)
	}

	fn dispatch<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		let Self(state, move_result) = self;
		match reg_move(state, ctx) {
			(RogueEnergy::Starved, mut state) => {
				// TODO Might need to do something like killed_by here instead.
				state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::StoppedOnSomething));
				RunStep::Exit(state)
			}
			(RogueEnergy::Fainted, mut state) => {
				state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::StoppedOnSomething));
				RunStep::Effect(state, RunEffect::AwaitMove)
			}
			(RogueEnergy::Normal, mut state) => {
				if state.as_health().confused.is_active() {
					state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::StoppedOnSomething));
					RunStep::Effect(state, RunEffect::AwaitMove)
				} else {
					state.level.rogue.move_result = Some(move_result.unwrap_or(MoveResult::Moved));
					RunStep::Effect(state, RunEffect::AwaitMove)
				}
			}
		}
	}
}
