use crate::init::Dungeon;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::check_hunger::CheckHungerEvent;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::move_monsters::MoveMonstersEvent;
use crate::resources::play::event::reg_move::update_health::update_health;
use crate::resources::play::event::reg_move::update_move_result::update_move_result;
use crate::resources::play::event::reg_move::update_wanderers::update_wanderers;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

pub mod update_wanderers;
pub mod update_health;
pub mod update_move_result;

impl GameEventVariant for RegMoveEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::RegMove(self) }
}

#[derive(Debug)]
pub enum RegMoveEvent {
	Stage1Start,
	Stage2CheckHunger { old_energy: RogueEnergy },
	Stage3MoveMonsters { old_energy: RogueEnergy },
	Stage4UpdateHealth { old_energy: RogueEnergy },
	Stage5UpdateMoveResult { old_energy: RogueEnergy },
}

impl RegMoveEvent {
	pub fn new() -> Self { Self::Stage1Start }
}

impl Dispatch for RegMoveEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		match self {
			Self::Stage1Start => {
				let old_energy = state.rogue_energy();
				if state.is_max_depth() || state.as_fighter().moves_left <= RogueEnergy::MAX_HUNGRY {
					Self::Stage2CheckHunger { old_energy }.into_redirect(state)
				} else {
					Self::Stage3MoveMonsters { old_energy }.into_redirect(state)
				}
			}
			Self::Stage2CheckHunger { old_energy } => {
				// Update rogue's energy level and perform any energy-level effects.
				let after_check = move |state: RunState| {
					match RogueEnergy::Starved == state.rogue_energy() {
						true => Self::Stage5UpdateMoveResult { old_energy }.into_run_event(state),
						false => Self::Stage3MoveMonsters { old_energy }.into_run_event(state),
					}
				};
				CheckHungerEvent::new(after_check).into_redirect(state)
			}
			Self::Stage3MoveMonsters { old_energy } => {
				// Move existing monsters.
				let after_move = move |state| {
					// After moving existing monsters, update wandering monsters.
					let state = update_wanderers(state);
					Self::Stage4UpdateHealth { old_energy }.into_redirect(state)
				};
				MoveMonstersEvent::new(after_move).into_redirect(state)
			}
			Self::Stage4UpdateHealth { old_energy } => {
				// Update rogue's health statuses.
				let state = update_health(state);
				Self::Stage5UpdateMoveResult { old_energy }.into_redirect(state)
			}
			Self::Stage5UpdateMoveResult { old_energy } => {
				// Record the move result and wait for the player's next move.
				state = update_move_result(state, old_energy);
				if RogueEnergy::Starved == state.rogue_energy() {
					state.into_exit()
				} else {
					state.into_effect(RunEffect::AwaitMove)
				}
			}
		}
	}
}