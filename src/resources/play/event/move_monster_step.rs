use crate::init::Dungeon;
use crate::level::constants::{DCOLS, DROWS};
use crate::monster::mon_can_go_and_reach;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::dice::roll_chance;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::monster_hit::MonsterHitEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::step_seed::StepSeed;
use crate::resources::play::state::RunState;
use crate::spec_hit::{flame_broil, m_confuse, seek_gold, GoldSearch};
use crate::{monster, odds};
use rand::prelude::SliceRandom;

impl GameEventVariant for MoveMonsterStepEvent {
	fn into_game_event(self) -> GameEvent {
		GameEvent::MoveMonsterStep(self)
	}
}

#[derive(Debug)]
pub enum MoveMonsterStepEvent {
	Start { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	AwakeCheckMoved { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NotMovedCheckFlit { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NoFlitCheckStationary { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NotStationaryCheckFreezing { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NotFreezingCheckConfuses { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NoConfusesCheckHit { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NoHitCheckFlames { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NoFlamesCheckGold { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
	NoGoldCheckTarget { mon_id: u64, to: LevelSpot, any_direction: bool, after_move: StepSeed },
}

impl MoveMonsterStepEvent {
	pub fn new(mon_id: u64, to: LevelSpot, any_direction: bool, after_move: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self::Start {
			mon_id,
			to,
			any_direction,
			after_move: StepSeed::new("move-monster", after_move),
		}
	}
}

impl Dispatch for MoveMonsterStepEvent {
	fn dispatch(self, mut state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			Self::Start { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.asleep {
					if state.as_monster(mon_id).m_flags.napping {
						state.as_monster_mut(mon_id).do_nap();
					} else {
						let wakens_when_rogue_is_near = state.as_monster(mon_id).m_flags.wakens;
						let rogue_is_near = {
							let mon_spot = state.as_monster(mon_id).spot;
							state.rogue_is_near(mon_spot.row, mon_spot.col)
						};
						if wakens_when_rogue_is_near
							&& rogue_is_near
							&& roll_chance(state.as_ring_effects().apply_stealthy(odds::WAKE_PERCENT), state.rng()) {
							state.as_monster_mut(mon_id).wake_up();
						}
					}
					after_move.into_step(state)
				} else {
					Self::AwakeCheckMoved { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::AwakeCheckMoved { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.already_moved {
					let monster = state.as_monster_mut(mon_id);
					monster.m_flags.already_moved = false;
					after_move.into_step(state)
				} else {
					Self::NotMovedCheckFlit { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::NotMovedCheckFlit { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.flits
					&& monster::flit(mon_id, any_direction, &mut state) {
					after_move.into_step(state)
				} else {
					Self::NoFlitCheckStationary { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::NoFlitCheckStationary { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.stationary
					&& !mon_can_go_and_reach(mon_id, state.rogue_row(), state.rogue_col(), any_direction, &mut state) {
					after_move.into_step(state)
				} else {
					Self::NotStationaryCheckFreezing { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::NotStationaryCheckFreezing { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.freezing_rogue {
					after_move.into_step(state)
				} else {
					Self::NotFreezingCheckConfuses { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::NotFreezingCheckConfuses { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.confuses
					&& m_confuse(mon_id, &mut state) {
					after_move.into_step(state)
				} else {
					Self::NoConfusesCheckHit { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::NoConfusesCheckHit { mon_id, to, any_direction, after_move } => {
				if mon_can_go_and_reach(mon_id, state.rogue_row(), state.rogue_col(), any_direction, &mut state) {
					let after_hit = move |state| after_move.into_step(state);
					MonsterHitEvent::new(mon_id, None, after_hit).into_redirect(state)
				} else {
					Self::NoHitCheckFlames { mon_id, to, any_direction, after_move }.into_redirect(state)
				}
			}
			Self::NoHitCheckFlames { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.flames {
					let (broiled, after_broil) = flame_broil(state, mon_id, ctx);
					if broiled {
						return after_move.into_step(after_broil);
					} else {
						state = after_broil;
					}
				}
				Self::NoFlamesCheckGold { mon_id, to, any_direction, after_move }.into_redirect(state)
			}
			Self::NoFlamesCheckGold { mon_id, to, any_direction, after_move } => {
				if state.as_monster(mon_id).m_flags.seeks_gold {
					match seek_gold(mon_id, &mut state) {
						GoldSearch::Failed => {}
						GoldSearch::FoundAndReached => {
							return after_move.into_step(state);
						}
						GoldSearch::FoundButUnreachable(gold_spot) => {
							let flags = state.as_monster_flags_mut(mon_id);
							flags.seeks_gold = false;
							// Move towards the gold instead.
							return MoveMonsterStepEvent::new(
								mon_id,
								gold_spot,
								true,
								move |mut state| {
									let flags = state.as_monster_flags_mut(mon_id);
									flags.seeks_gold = true;
									after_move.into_step(state)
								},
							).into_redirect(state);
						}
					}
				}
				Self::NoGoldCheckTarget { mon_id, to, any_direction, after_move }.into_redirect(state)
			}
			Self::NoGoldCheckTarget { mon_id, to, any_direction, after_move } => {
				state.as_monster_mut(mon_id).clear_target_spot_if_reached();
				let target_spot = state.as_monster(mon_id).target_spot_or(to.into());
				let monster_spot = state.as_monster(mon_id).spot;
				let row = monster_spot.next_closest_row(target_spot.row);
				if state.is_any_door_at(row, monster_spot.col)
					&& monster::mtry(mon_id, row, monster_spot.col, any_direction, &mut state) {
					return after_move.into_step(state);
				}
				let col = monster_spot.next_closest_col(target_spot.col);
				if state.is_any_door_at(monster_spot.row, col)
					&& monster::mtry(mon_id, monster_spot.row, col, any_direction, &mut state) {
					return after_move.into_step(state);
				}
				if monster::mtry(mon_id, row, col, any_direction, &mut state) {
					return after_move.into_step(state);
				}
				let mut indices = (0..=5).collect::<Vec<_>>();
				indices.shuffle(state.rng());
				for kind in indices {
					match kind {
						0 => if monster::mtry(mon_id, row, monster_spot.col - 1, any_direction, &mut state) { break; }
						1 => if monster::mtry(mon_id, row, monster_spot.col, any_direction, &mut state) { break; }
						2 => if monster::mtry(mon_id, row, monster_spot.col + 1, any_direction, &mut state) { break; }
						3 => if monster::mtry(mon_id, monster_spot.row - 1, col, any_direction, &mut state) { break; }
						4 => if monster::mtry(mon_id, monster_spot.row, col, any_direction, &mut state) { break; }
						5 => if monster::mtry(mon_id, monster_spot.row + 1, col, any_direction, &mut state) { break; }
						_ => unreachable!("0 <= n  <= 5")
					}
				}
				let new_monster_spot = state.as_monster(mon_id).spot;
				if new_monster_spot == monster_spot {
					state.as_monster_mut(mon_id).stuck_counter.log_row_col(monster_spot.row, monster_spot.col);
					if state.as_monster(mon_id).stuck_counter.count > 4 {
						// Stuck too many times.
						let no_target = state.as_monster_mut(mon_id).target_spot.is_none();
						let cant_see_rogue = !state.monster_sees_rogue(mon_id);
						if no_target && cant_see_rogue {
							let row = state.roll_range(1..=(DROWS as i64 - 2));
							let col = state.roll_range(0..=(DCOLS as i64 - 1));
							state.as_monster_mut(mon_id).set_target_spot(row, col);
						} else {
							state.as_monster_mut(mon_id).clear_target_reset_stuck();
						}
					}
				}
				after_move.into_step(state)
			}
		}
	}
}
