use crate::init::Dungeon;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::stage1_start_reg_move::Stage1StartRegMove;
use crate::resources::play::event::reg_move::stage2_check_hunger::Stage2CheckHunger;
use crate::resources::play::event::reg_move::stage3_move_monsters::{get_monster_ids_for_movement, mv_one_mon, update_wanderers};
use crate::resources::play::event::reg_move::stage4_update_health::Stage4UpdateHealth;
use crate::resources::play::event::reg_move::stage5_update_move_result::Stage5UpdateMoveResult;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
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
	CheckHunger(Stage2CheckHunger),
	Stage3MoveMonsters { old_energy: RogueEnergy, mon_ids: Option<Vec<u64>> },
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
			Self::CheckHunger(task) => task.dispatch(state, ctx),
			Self::Stage3MoveMonsters { old_energy, mon_ids } => {
				match mon_ids {
					None => {
						let mon_ids = get_monster_ids_for_movement(&state);
						Self::Stage3MoveMonsters { old_energy, mon_ids: Some(mon_ids) }.into_redirect(state)
					}
					Some(mut mon_ids) => {
						match mon_ids.pop() {
							None => {
								let state = update_wanderers(state);
								Stage4UpdateHealth::new(old_energy).into_redirect(state)
							}
							Some(mon_id) => {
								let state = mv_one_mon(mon_id, state, ctx);
								if state.cleaned_up().is_some() {
									state.into_exit()
								} else {
									Self::Stage3MoveMonsters { old_energy, mon_ids: Some(mon_ids) }.into_redirect(state)
								}
							}
						}
					}
				}
			}
			Self::UpdateHealth(task) => task.dispatch(state, ctx),
			Self::UpdateMoveResult(task) => task.dispatch(state, ctx),
		}
	}
}