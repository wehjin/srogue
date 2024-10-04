use crate::init::Dungeon;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEventVariant};
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

#[derive(Debug)]
pub(super) struct Stage1StartRegMove;
impl RegMoveStage for Stage1StartRegMove {
	fn into_reg_move_event(self) -> RegMoveEvent { RegMoveEvent::StartRegMove(self) }
}

impl Dispatch for Stage1StartRegMove {
	fn dispatch(self, state: RunState, _ctx: &mut RunContext) -> RunStep {
		let old_energy = state.rogue_energy();
		if state.is_max_depth() || state.as_fighter().moves_left <= RogueEnergy::MAX_HUNGRY {
			RegMoveEvent::Stage2CheckHunger { old_energy }.into_redirect(state)
		} else {
			RegMoveEvent::Stage3MoveMonsters { old_energy }.into_redirect(state)
		}
	}
}
