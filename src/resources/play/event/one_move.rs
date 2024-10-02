use crate::hit::rogue_hit;
use crate::init::Dungeon;
use crate::inventory::get_obj_desc;
use crate::motion::{MoveDirection, MoveResult};
use crate::odds::R_TELE_PERCENT;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::wake::{wake_room, WakeType};
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::message::Message;
use crate::resources::play::event::pick_up::{PickUpRegMove, PickupType};
use crate::resources::play::event::reg_move::RegMove;
use crate::resources::play::event::state_action::{redirect, StateAction};
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use crate::resources::rogue::spot::RogueSpot;

#[derive(Debug, Clone)]
pub struct OneMove(pub RunState, pub MoveDirection);

impl StateAction for OneMove {
	fn into_event(self) -> RunEvent {
		RunEvent::OneMove(self)
	}

	fn dispatch(self, ctx: &mut RunContext) -> RunStep {
		let OneMove(state, direction) = self;
		walk(state, direction, true, ctx)
	}
}


fn walk(state: RunState, direction: MoveDirection, allow_pickup: bool, ctx: &mut RunContext) -> RunStep {
	let mut next_state = State::Start { state, direction, allow_pickup };
	loop {
		let state = step(next_state, ctx);
		if let State::End(step) = state {
			return step;
		}
		next_state = state;
	}
}

fn step(state: State, ctx: &mut RunContext) -> State {
	match state {
		State::Start { mut state, direction, allow_pickup } => {
			state.move_result = None;
			state.diary.clear_message_lines();
			// Where are we now?
			let rogue_spot = (state.rogue_row(), state.rogue_col());
			// Where are we going?
			let to_spot = get_destination_spot(direction, rogue_spot.0, rogue_spot.1, &mut state);
			let to_spot_is_navigable = state.rogue_can_move(to_spot.0, to_spot.1);
			if !to_spot_is_navigable {
				state.move_result = Some(MoveResult::MoveFailed);
				State::End(RunStep::Effect(state, RunEffect::AwaitMove))
			} else {
				State::CheckStuck { state, to_spot, rogue_spot, allow_pickup }
			}
		}
		State::CheckStuck { mut state, to_spot, rogue_spot, allow_pickup } => {
			// What if we're stuck in place?
			let no_monster_at_spot = !state.has_monster_at(to_spot.0, to_spot.1);
			let in_bear_trap = state.as_health().bear_trap > 0;
			let being_held = state.as_health().being_held;
			if being_held && no_monster_at_spot {
				state.move_result = Some(MoveResult::MoveFailed);
				// Wait for the player's next move after reporting being held.
				let after_report = |state| RunStep::Effect(state, RunEffect::AwaitMove);
				let report = "you are being held";
				let next_step = redirect(Message::new(state, report, true, after_report));
				State::End(next_step)
			} else if in_bear_trap && no_monster_at_spot {
				state.move_result = Some(MoveResult::MoveFailed);
				// Do a regular move after reporting the bear trap so that the bear trap counts down.
				let after_report = |state| redirect(RegMove(state));
				let report = "you are still stuck in the bear trap";
				let next_step = redirect(Message::new(state, report, true, after_report));
				State::End(next_step)
			} else {
				State::CheckTeleport { state, to_spot, rogue_spot, allow_pickup }
			}
		}
		State::CheckTeleport { mut state, to_spot, rogue_spot, allow_pickup } => {
			// What if we're wearing a teleport ring?
			if state.as_ring_effects().has_teleport() && state.roll_chance(R_TELE_PERCENT) {
				state.move_result = Some(MoveResult::StoppedOnSomething);
				// TODO tele(game);
				State::End(RunStep::Effect(state, RunEffect::AwaitMove))
			} else {
				State::CheckMonster { state, to_spot, rogue_spot, allow_pickup }
			}
		}
		State::CheckMonster { mut state, to_spot, rogue_spot, allow_pickup } => {
			// What if there is a monster is where we want to go?
			let monster_in_spot = state.has_monster_at(to_spot.0, to_spot.1);
			if monster_in_spot {
				state.move_result = Some(MoveResult::MoveFailed);
				let mon_id = state.get_monster_at(to_spot.0, to_spot.1).unwrap();
				state = rogue_hit(state, mon_id, false, ctx);
				let next_step = redirect(RegMove(state));
				State::End(next_step)
			} else {
				State::AdjustLighting { state, to_spot, rogue_spot, allow_pickup }
			}
		}
		State::AdjustLighting { mut state, to_spot, rogue_spot, allow_pickup } => {
			// The lighting in the level changes as we move.
			if state.is_any_door_at(to_spot.0, to_spot.1) {
				// What if we're moving to a door?
				match state.level.rogue.spot {
					RogueSpot::None => {}
					RogueSpot::Passage(_) => {
						// tunnel to door
						let door = state.level.get_door_at(LevelSpot::from(to_spot)).unwrap();
						state.level.light_room(door.room_id);
						let (level, rng) = wake_room(WakeType::EnterVault(door.room_id), state.level, state.rng);
						state.level = level;
						state.rng = rng;
					}
					RogueSpot::Vault(_, _) => {
						// vault to door
						state.level.light_tunnel_spot(LevelSpot::from(to_spot));
					}
				}
			} else if state.is_any_door_at(rogue_spot.0, rogue_spot.1) && state.is_any_tunnel_at(to_spot.0, to_spot.1) {
				// door to tunnel
				let door = state.level.get_door_at(LevelSpot::from(rogue_spot)).unwrap();
				state.level.light_tunnel_spot(LevelSpot::from(to_spot));
				let (level, rng) = wake_room(WakeType::ExitVault(door.room_id, LevelSpot::from(rogue_spot)), state.level, state.rng);
				state.level = level;
				state.rng = rng;
				// TODO darken_room()
			} else if state.is_any_tunnel_at(to_spot.0, to_spot.1) {
				// tunnel to tunnel
				state.level.light_tunnel_spot(LevelSpot::from(to_spot));
			}
			State::MoveRogue { state, to_spot, allow_pickup }
		}
		State::MoveRogue { mut state, to_spot, allow_pickup } => {
			// Move the rogue.
			state.set_rogue_row_col(to_spot.0, to_spot.1);
			State::PickupObjects { state, spot: to_spot, allow_pickup }
		}
		State::PickupObjects { state, spot, allow_pickup } => {
			// Pick up objects.
			let has_object = state.level.try_object(LevelSpot::from(spot)).is_some();
			if has_object {
				// [pickup_object] will update [move_result].
				let step = pickup_object(spot.0, spot.1, allow_pickup, state, ctx);
				State::End(step)
			} else {
				State::CheckTraps { state, spot }
			}
		}
		State::CheckTraps { mut state, spot: (row, col) } => {
			// Check for traps.
			let is_door = state.is_any_door_at(row, col);
			let is_stairs = state.level.features.feature_at(LevelSpot::from_i64(row, col)).is_stairs();
			let is_trap = state.is_any_trap_at(row, col);
			if is_door || is_stairs || is_trap {
				let not_levitating = state.as_health().levitate.is_inactive();
				if is_trap && not_levitating {
					// TODO trap_player(row as usize, col as usize, game);
				}
				state.move_result = Some(MoveResult::StoppedOnSomething);
				State::End(redirect(RegMove(state)))
			} else {
				// RegMove will update [move_result].
				State::End(redirect(RegMove(state)))
			}
		}
		State::End(_) => { panic!("Do not step the end state!") }
	}
}

pub enum State {
	Start { state: RunState, direction: MoveDirection, allow_pickup: bool },
	CheckStuck { state: RunState, to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	CheckTeleport { state: RunState, to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	CheckMonster { state: RunState, to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	AdjustLighting { state: RunState, to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	MoveRogue { state: RunState, to_spot: (i64, i64), allow_pickup: bool },
	PickupObjects { state: RunState, spot: (i64, i64), allow_pickup: bool },
	CheckTraps { state: RunState, spot: (i64, i64) },
	End(RunStep),
}

fn get_destination_spot(direction: MoveDirection, from_row: i64, from_col: i64, state: &mut RunState) -> (i64, i64) {
	let confused = state.as_health().confused.is_active();
	let confused_direction = if !confused { direction } else { MoveDirection::random(state.rng()) };
	confused_direction.apply(from_row, from_col)
}

fn pickup_object(row: i64, col: i64, allow_pickup: bool, mut state: RunState, ctx: &mut RunContext) -> RunStep {
	if allow_pickup {
		// Pick up the object and complete the move.
		let spot = LevelSpot::from_i64(row, col);
		PickUpRegMove(state, PickupType::AfterMove(spot)).dispatch(ctx)
	} else {
		// Leave the object alone but stop moving and report we're on top of it. Then complete the move.
		state.move_result = Some(MoveResult::StoppedOnSomething);
		let after_report = move |state| redirect(RegMove(state));
		let report = moved_onto_message(row, col, &state);
		Message::new(state, report, true, after_report).dispatch(ctx)
	}
}

pub fn moved_onto_message(row: i64, col: i64, state: &RunState) -> String {
	let obj = state.level.try_object(LevelSpot::from_i64(row, col)).unwrap();
	let obj_desc = get_obj_desc(obj, &state);
	let desc = format!("moved onto {}", obj_desc);
	desc
}
