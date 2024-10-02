use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::stage1_start_reg_move::Stage1StartRegMove;
use crate::resources::play::event::reg_move::stage2_check_hunger::Stage2CheckHunger;
use crate::resources::play::event::reg_move::stage3_move_monsters::Stage3MoveMonsters;
use crate::resources::play::event::reg_move::stage4_update_health::Stage4UpdateHealth;
use crate::resources::play::event::reg_move::stage5_update_move_result::Stage5UpdateMoveResult;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;

pub mod stage;
pub mod stage2_check_hunger;
pub mod stage3_move_monsters;
pub mod stage1_start_reg_move;
pub mod stage4_update_health;
pub mod stage5_update_move_result;

#[derive(Debug)]
pub enum RegMoveEvent {
	StartRegMove(Stage1StartRegMove),
	CheckHunger(Stage2CheckHunger),
	MoveMonsters(Stage3MoveMonsters),
	UpdateHealth(Stage4UpdateHealth),
	UpdateMoveResult(Stage5UpdateMoveResult),
}

impl RegMoveEvent {
	pub fn new() -> Self {
		Stage1StartRegMove.into_reg_move_event()
	}
}

impl GameEventVariant for RegMoveEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::RegMoveTask(self) }
}

impl Dispatch for RegMoveEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			RegMoveEvent::StartRegMove(task) => task.dispatch(state, ctx),
			RegMoveEvent::CheckHunger(task) => task.dispatch(state, ctx),
			RegMoveEvent::MoveMonsters(task) => task.dispatch(state, ctx),
			RegMoveEvent::UpdateHealth(task) => task.dispatch(state, ctx),
			RegMoveEvent::UpdateMoveResult(task) => task.dispatch(state, ctx),
		}
	}
}