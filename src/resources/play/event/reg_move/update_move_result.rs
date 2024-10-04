use crate::motion::MoveResult;
use crate::resources::avatar::Avatar;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

pub fn update_move_result(mut state: RunState, old_energy: RogueEnergy) -> RunState {
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
