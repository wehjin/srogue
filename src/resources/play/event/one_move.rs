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
use crate::resources::play::event::pickup::{Pickup, PickupType};
use crate::resources::play::event::reg_move::RegMove;
use crate::resources::play::event::state_action::{redirect, StateAction};
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use crate::resources::rogue::spot::RogueSpot;
use rand::Rng;

#[derive(Debug)]
pub struct OneMove(pub RunState, pub MoveDirection);

impl StateAction for OneMove {
	fn into_event(self) -> RunEvent {
		RunEvent::OneMove(self)
	}

	fn dispatch<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		let OneMove(state, direction) = self;
		let step = one_move_rogue(direction, true, state, ctx);
		step
	}
}

fn one_move_rogue<R: Rng>(direction: MoveDirection, allow_pickup: bool, mut state: RunState, ctx: &mut RunContext<R>) -> RunStep {
	state.level.rogue.move_result = None;
	state.diary.clear_message_lines();
	{
		// Where are we now?
		let rogue_row = state.rogue_row();
		let rogue_col = state.rogue_col();

		// Where are we going?
		let (to_row, to_col) = {
			let confused = state.as_health().confused.is_active();
			let confused_direction = if !confused { direction } else { MoveDirection::random(ctx.rng()) };
			confused_direction.apply(rogue_row, rogue_col)
		};
		// Is the spot navigable?
		let to_spot_is_navigable = state.rogue_can_move(to_row, to_col);
		if !to_spot_is_navigable {
			state.level.rogue.move_result = Some(MoveResult::MoveFailed);
			return RunStep::Effect(state, RunEffect::AwaitMove);
		}
		// What if we're stuck in place?
		{
			let health = state.as_health();
			let in_bear_trap = health.bear_trap > 0;
			let being_held = health.being_held;
			if being_held || in_bear_trap {
				let monster_in_spot = state.has_monster_at(to_row, to_col);
				if !monster_in_spot {
					return if being_held {
						state.level.rogue.move_result = Some(MoveResult::MoveFailed);
						let message = "you are being held";
						Message::new(state, message, true, |state| {
							RunStep::Effect(state, RunEffect::AwaitMove)
						}).dispatch(ctx)
					} else {
						state.level.rogue.move_result = Some(MoveResult::MoveFailed);
						let message = "you are still stuck in the bear trap";
						Message::new(state, message, true, |state| {
							// Do a regular move here so that the bear trap counts down.
							redirect(RegMove(state, Some(MoveResult::MoveFailed)))
						}).dispatch(ctx)
					};
				}
			}
		}
		// What if we're wearing a teleport ring?
		if state.as_ring_effects().has_teleport() && ctx.roll_chance(R_TELE_PERCENT) {
			state.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
			// TODO tele(game);
			return RunStep::Effect(state, RunEffect::AwaitMove);
		}
		// What if there is a monster is where we want to go?
		let monster_in_spot = state.has_monster_at(to_row, to_col);
		if monster_in_spot {
			let _mon_id = state.get_monster_at(to_row, to_col).unwrap();
			state.level.rogue.move_result = Some(MoveResult::MoveFailed);
			// TODO rogue_hit(mon_id, false, game);
			return RegMove(state, Some(MoveResult::MoveFailed)).dispatch(ctx);
		}
		// The lighting in the level changes as we move.
		// What if we're moving to a door?
		if state.is_any_door_at(to_row, to_col) {
			match state.level.rogue.spot {
				RogueSpot::None => {}
				RogueSpot::Passage(_) => {
					// tunnel to door
					let door = state.level.get_door_at(LevelSpot::from_i64(to_row, to_col)).unwrap();
					state.level.light_room(door.room_id);
					wake_room(WakeType::EnterVault(door.room_id), &mut state.level, ctx.rng());
				}
				RogueSpot::Vault(_, _) => {
					// vault to door
					state.level.light_tunnel_spot(LevelSpot::from_i64(to_row, to_col));
				}
			}
		} else if state.is_any_door_at(rogue_row, rogue_col) && state.is_any_tunnel_at(to_row, to_col) {
			// door to tunnel
			let door = state.level.get_door_at(LevelSpot::from_i64(rogue_row, rogue_col)).unwrap();
			state.level.light_tunnel_spot(LevelSpot::from_i64(to_row, to_col));
			wake_room(WakeType::ExitVault(door.room_id, LevelSpot::from_i64(rogue_row, rogue_col)), &mut state.level, ctx.rng());
			// TODO darken_room()
		} else if state.is_any_tunnel_at(to_row, to_col) {
			// tunnel to tunnel
			state.level.light_tunnel_spot(LevelSpot::from_i64(to_row, to_col));
		}

		// Move the rogue.
		state.set_rogue_row_col(to_row, to_col);
	}

	// We have moved.
	let row = state.rogue_row();
	let col = state.rogue_col();
	let has_object = state.level.try_object(LevelSpot::from_i64(row, col)).is_some();
	if has_object {
		return pickup_object(row, col, allow_pickup, state, ctx);
	}
	if state.is_any_door_at(row, col) || state.level.features.feature_at(LevelSpot::from_i64(row, col)).is_stairs() || state.is_any_trap_at(row, col) {
		if state.as_health().levitate.is_inactive() && state.is_any_trap_at(row, col) {
			// TODO trap_player(row as usize, col as usize, game);
		}
		return RegMove(state, Some(MoveResult::StoppedOnSomething)).dispatch(ctx);
	}
	RegMove(state, None).dispatch(ctx)
}

fn pickup_object<R: Rng>(row: i64, col: i64, allow_pickup: bool, state: RunState, ctx: &mut RunContext<R>) -> RunStep {
	if allow_pickup {
		let spot = LevelSpot::from_i64(row, col);
		Pickup(state, PickupType::AfterMove(spot)).dispatch(ctx)
	} else {
		let message = moved_onto_message(row, col, &state);
		let post_step = move |state| redirect(RegMove(state, Some(MoveResult::StoppedOnSomething)));
		Message::new(state, message, true, post_step).dispatch(ctx)
	}
}

pub fn moved_onto_message(row: i64, col: i64, state: &RunState) -> String {
	let obj = state.level.try_object(LevelSpot::from_i64(row, col)).unwrap();
	let obj_desc = get_obj_desc(obj, &state);
	let desc = format!("moved onto {}", obj_desc);
	desc
}
