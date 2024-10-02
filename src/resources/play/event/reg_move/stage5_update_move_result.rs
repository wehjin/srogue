use crate::motion::MoveResult;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::game::Dispatch;
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

#[derive(Debug)]
pub(super) struct Stage5UpdateMoveResult {
	old_energy: RogueEnergy,
}
impl Stage5UpdateMoveResult {
	pub fn new(old_energy: RogueEnergy) -> Self {
		Self { old_energy }
	}
}

impl RegMoveStage for Stage5UpdateMoveResult {
	fn into_reg_move_event(self) -> RegMoveEvent { RegMoveEvent::UpdateMoveResult(self) }
}

impl Dispatch for Stage5UpdateMoveResult {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		let Stage5UpdateMoveResult { old_energy } = self;
		state = update_move_result(state, old_energy);
		if RogueEnergy::Starved == state.rogue_energy() {
			state.into_exit()
		} else {
			state.into_effect(RunEffect::AwaitMove)
		}
	}
}

fn update_move_result(mut state: RunState, old_energy: RogueEnergy) -> RunState {
	let energy = state.rogue_energy();
	state.move_result = state.move_result.or_else(|| {
		let energy_changed = energy != old_energy;
		let starved = energy == RogueEnergy::Starved;
		let confused = state.as_health().confused.is_active();
		let interrupted = state.diary.interrupted;
		if energy_changed || starved || confused || interrupted {
			Some(MoveResult::StoppedOnSomething)
		} else {
			Some(MoveResult::Moved)
		}
	});
	state
}
