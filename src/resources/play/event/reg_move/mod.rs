use crate::init::Dungeon;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::move_monsters::MoveMonstersEvent;
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::stage1_start_reg_move::Stage1StartRegMove;
use crate::resources::play::event::reg_move::stage2_check_hunger::check_hunger;
use crate::resources::play::event::reg_move::stage3_move_monsters::update_wanderers;
use crate::resources::play::event::reg_move::stage4_update_health::Stage4UpdateHealth;
use crate::resources::play::event::reg_move::stage5_update_move_result::Stage5UpdateMoveResult;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::EventSeed;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

pub mod stage;
pub mod stage2_check_hunger;
pub mod stage3_move_monsters;
pub mod stage1_start_reg_move;
pub mod stage4_update_health;
pub mod stage5_update_move_result;

#[derive(Debug)]
pub enum RegMoveEvent {
	StartRegMove(Stage1StartRegMove),
	Stage2CheckHunger { old_energy: RogueEnergy },
	Stage3MoveMonsters { old_energy: RogueEnergy },
	UpdateHealth(Stage4UpdateHealth),
	UpdateMoveResult(Stage5UpdateMoveResult),
}

impl RegMoveEvent {
	pub fn new() -> Self {
		Stage1StartRegMove.into_reg_move_event()
	}
}

impl GameEventVariant for RegMoveEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::RegMove(self) }
}

impl Dispatch for RegMoveEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			Self::StartRegMove(task) => task.dispatch(state, ctx),
			Self::Stage2CheckHunger { old_energy } => {
				// Check the rogue's energy state.
				let state = check_hunger(state, ctx);
				match RogueEnergy::Starved == state.rogue_energy() {
					true => Stage5UpdateMoveResult::new(old_energy).into_redirect(state),
					false => RegMoveEvent::Stage3MoveMonsters { old_energy }.into_redirect(state),
				}
			}
			Self::Stage3MoveMonsters { old_energy } => {
				// Move existing monsters then update wandering monsters.
				let after_move = EventSeed::new(move |state| {
					let state = update_wanderers(state);
					Stage4UpdateHealth::new(old_energy).into_run_event(state)
				});
				MoveMonstersEvent { mon_ids: None, after_move }.into_redirect(state)
			}
			Self::UpdateHealth(task) => task.dispatch(state, ctx),
			Self::UpdateMoveResult(task) => task.dispatch(state, ctx),
		}
	}
}