use crate::init::Dungeon;
use crate::monster;
use crate::monster::{mon_can_go_and_reach, move_confused};
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::step_seed::StepSeed;
use crate::resources::play::state::RunState;

impl GameEventVariant for MoveMonstersEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::MoveMonsters(self) }
}

#[derive(Debug)]
pub struct MoveMonstersEvent {
	mon_ids: Option<Vec<u64>>,
	after_move: StepSeed,
}

impl MoveMonstersEvent {
	pub fn new(after_move: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self { mon_ids: None, after_move: StepSeed::new("move-monster", after_move) }
	}
}

impl Dispatch for MoveMonstersEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		let Self { mon_ids, after_move } = self;
		match mon_ids {
			None => {
				let mon_ids = get_monster_ids_for_movement(&state);
				Self { mon_ids: Some(mon_ids), after_move }.into_redirect(state)
			}
			Some(mut mon_ids) => match state.cleaned_up().is_some() {
				true => state.into_exit(),
				false => match mon_ids.pop() {
					None => after_move.into_step(state),
					Some(mon_id) => {
						match state.as_monster(mon_id).is_defeated() {
							true => Self { mon_ids: Some(mon_ids), after_move }.into_redirect(state),
							false => {
								let after_mon_move = |state| Self { mon_ids: Some(mon_ids), after_move }.into_redirect(state);
								MoveMonsterFullEvent::new(mon_id, after_mon_move).into_redirect(state)
							}
						}
					}
				},
			}
		}
	}
}

fn get_monster_ids_for_movement(state: &RunState) -> Vec<u64> {
	if state.cleaned_up().is_some() || state.as_health().haste_self.is_half_active() {
		vec![]
	} else {
		state.monster_ids()
	}
}

impl GameEventVariant for MoveMonsterFullEvent {
	fn into_game_event(self) -> GameEvent {
		GameEvent::MoveMonsterFull(self)
	}
}
#[derive(Debug)]
pub enum MoveMonsterFullEvent {
	Start { mon_id: u64, after_move: StepSeed },
	Step { count: usize, mon_id: u64, after_move: StepSeed },
	ConfusedStep { count: usize, mon_id: u64, after_move: StepSeed },
	AirStep { count: usize, mon_id: u64, after_move: StepSeed },
	GroundStep { count: usize, mon_id: u64, after_move: StepSeed },
	End { after_move: StepSeed },
}
impl MoveMonsterFullEvent {
	pub fn new(mon_id: u64, after_move: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self::Start { mon_id, after_move: StepSeed::new("move-monster-ful", after_move) }
	}
}

impl Dispatch for MoveMonsterFullEvent {
	fn dispatch(self, mut state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			Self::Start { mon_id, after_move } => {
				let count = if state.as_monster(mon_id).is_hasted() {
					2
				} else if state.as_monster(mon_id).is_slowed() {
					state.as_monster_mut(mon_id).flip_slowed_toggle();
					if state.as_monster(mon_id).is_slowed_toggle() { 0 } else { 1 }
				} else {
					1
				};
				Self::Step { count, mon_id, after_move }.into_redirect(state)
			}
			Self::Step { count, mon_id, after_move } => {
				if count == 0 {
					Self::End { after_move }.into_redirect(state)
				} else {
					Self::ConfusedStep { count, mon_id, after_move }.into_redirect(state)
				}
			}
			Self::ConfusedStep { count, mon_id, after_move } => {
				if state.as_monster(mon_id).is_confused() && move_confused(mon_id, false, &mut state) {
					Self::Step { count: count - 1, mon_id, after_move }.into_redirect(state)
				} else {
					Self::AirStep { count, mon_id, after_move }.into_redirect(state)
				}
			}
			Self::AirStep { count, mon_id, after_move } => {
				let rogue_row = state.rogue_row();
				let rogue_col = state.rogue_col();
				let flies = state.as_monster(mon_id).flies();
				let not_napping = !state.as_monster(mon_id).is_napping();
				let cant_reach_rogue = !mon_can_go_and_reach(mon_id, rogue_row, rogue_col, false, &state);
				if flies && not_napping && cant_reach_rogue {
					// If monster flies and cannot reach the rogue in one step, then the monster is allowed
					// to attack from two steps away.
					state = monster::mv_monster(state, mon_id, rogue_row, rogue_col, false, ctx);
					let still_cant_reach_rogue = !mon_can_go_and_reach(mon_id, rogue_row, rogue_col, false, &state);
					if still_cant_reach_rogue {
						Self::Step { count: count - 1, mon_id, after_move }.into_redirect(state)
					} else {
						Self::GroundStep { count, mon_id, after_move }.into_redirect(state)
					}
				} else {
					Self::GroundStep { count, mon_id, after_move }.into_redirect(state)
				}
			}
			Self::GroundStep { count, mon_id, after_move } => {
				let rogue_row = state.rogue_row();
				let rogue_col = state.rogue_col();
				// TODO Convert mv_monster into an event.
				let state = monster::mv_monster(state, mon_id, rogue_row, rogue_col, false, ctx);
				let new_count = match state.get_monster(mon_id) {
					None => 0,
					Some(monster) => if monster.is_defeated() { 0 } else { count - 1 }
				};
				Self::Step { count: new_count, mon_id, after_move }.into_redirect(state)
			}
			Self::End { after_move } => after_move.into_step(state),
		}
	}
}
