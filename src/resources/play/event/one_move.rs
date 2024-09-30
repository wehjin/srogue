use crate::init::Dungeon;
use crate::inventory::get_obj_desc;
use crate::motion::{reg_move, MoveDirection, MoveResult, RegMoveResult};
use crate::odds::R_TELE_PERCENT;
use crate::pack::{pick_up, PickUpResult};
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::wake::{wake_room, WakeType};
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::message::{print_and_redirect, MessageEvent};
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use crate::resources::rogue::spot::RogueSpot;
use rand::Rng;

#[derive(Debug)]
pub struct OneMoveEvent(pub RunState, pub MoveDirection);

impl OneMoveEvent {
	pub fn into_step<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		let OneMoveEvent(state, direction) = self;
		let step = one_move_rogue(direction, true, state, ctx);
		step
	}
}

fn one_move_rogue<R: Rng>(direction: MoveDirection, pickup: bool, mut game: RunState, ctx: &mut RunContext<R>) -> RunStep {
	game.level.rogue.move_result = None;
	game.diary.clear_message_lines();
	{
		// Where are we now?
		let rogue_row = game.rogue_row();
		let rogue_col = game.rogue_col();

		// Where are we going?
		let (to_row, to_col) = {
			let confused = game.as_health().confused.is_active();
			let confused_direction = if !confused { direction } else { MoveDirection::random(ctx.rng()) };
			confused_direction.apply(rogue_row, rogue_col)
		};
		// Is the spot navigable?
		let to_spot_is_navigable = game.rogue_can_move(to_row, to_col);
		if !to_spot_is_navigable {
			game.level.rogue.move_result = Some(MoveResult::MoveFailed);
			return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
		}
		// What if we're stuck in place?
		{
			let begin_held = game.as_health().being_held;
			let in_bear_trap = game.as_health().bear_trap > 0;
			if begin_held || in_bear_trap {
				let monster_in_spot = game.has_monster_at(to_row, to_col);
				if !monster_in_spot {
					return if begin_held {
						game.level.rogue.move_result = Some(MoveResult::MoveFailed);
						let message = "you are being held";
						let state = MessageEvent::dispatch(game, message, true, ctx);
						RunStep::Effect(state, RunEffect::AwaitPlayerMove)
					} else {
						game.level.rogue.move_result = Some(MoveResult::MoveFailed);
						let message = "you are still stuck in the bear trap";
						let mut state = MessageEvent::dispatch(game, message, false, ctx);
						// Do a regular move here so that the bear trap counts down.
						reg_move(&mut state);
						RunStep::Effect(state, RunEffect::AwaitPlayerMove)
					};
				}
			}
		}
		// What if we're wearing a teleport ring?
		if game.as_ring_effects().has_teleport() && ctx.roll_chance(R_TELE_PERCENT) {
			game.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
			// TODO tele(game);
			return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
		}
		// What if there is a monster is where we want to go?
		let monster_in_spot = game.has_monster_at(to_row, to_col);
		if monster_in_spot {
			let _mon_id = game.get_monster_at(to_row, to_col).unwrap();
			game.level.rogue.move_result = Some(MoveResult::MoveFailed);
			// TODO rogue_hit(mon_id, false, game);
			reg_move(&mut game);
			return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
		}
		// The lighting in the level changes as we move.
		// What if we're moving to a door?
		if game.is_any_door_at(to_row, to_col) {
			match game.level.rogue.spot {
				RogueSpot::None => {}
				RogueSpot::Passage(_) => {
					// tunnel to door
					let door = game.level.get_door_at(LevelSpot::from_i64(to_row, to_col)).unwrap();
					game.level.light_room(door.room_id);
					wake_room(WakeType::EnterVault(door.room_id), &mut game.level, ctx.rng());
				}
				RogueSpot::Vault(_, _) => {
					// vault to door
					game.level.light_tunnel_spot(LevelSpot::from_i64(to_row, to_col));
				}
			}
		} else if game.is_any_door_at(rogue_row, rogue_col) && game.is_any_tunnel_at(to_row, to_col) {
			// door to tunnel
			let door = game.level.get_door_at(LevelSpot::from_i64(rogue_row, rogue_col)).unwrap();
			game.level.light_tunnel_spot(LevelSpot::from_i64(to_row, to_col));
			wake_room(WakeType::ExitVault(door.room_id, LevelSpot::from_i64(rogue_row, rogue_col)), &mut game.level, ctx.rng());
			// TODO darken_room()
		} else if game.is_any_tunnel_at(to_row, to_col) {
			// tunnel to tunnel
			game.level.light_tunnel_spot(LevelSpot::from_i64(to_row, to_col));
		}

		// Move the rogue.
		game.set_rogue_row_col(to_row, to_col);
	}

	// We have moved.
	let row = game.rogue_row();
	let col = game.rogue_col();
	let has_object = game.level.try_object(LevelSpot::from_i64(row, col)).is_some();
	if has_object {
		return if !pickup {
			stopped_on_something_with_message(&moved_on_message(row, col, &game), game)
		} else {
			if game.as_health().levitate.is_active() {
				game.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
				RunStep::Effect(game, RunEffect::AwaitPlayerMove)
			} else {
				match pick_up(row, col, game, ctx) {
					(PickUpResult::TurnedToDust, state) => {
						keep_moving_unless_hungry_or_confused(state)
					}
					(PickUpResult::AddedToGold(obj), mut state) => {
						state.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
						let msg = get_obj_desc(&obj, &state);
						print_and_redirect(state, &msg, true, |state| RunEvent::RegisterMove(state),
						)
					}
					(PickUpResult::AddedToPack { added_id, .. }, mut state) => {
						state.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
						let msg = state.get_rogue_obj_desc(added_id);
						print_and_redirect(state, &msg, true, |state| RunEvent::RegisterMove(state))
					}
					(PickUpResult::PackTooFull, mut state) => {
						state.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
						stopped_on_something_with_message(&moved_on_message(row, col, &state), state)
					}
				}
			}
		};
	}
	if game.is_any_door_at(row, col) || game.level.features.feature_at(LevelSpot::from_i64(row, col)).is_stairs() || game.is_any_trap_at(row, col) {
		game.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
		if game.as_health().levitate.is_inactive() && game.is_any_trap_at(row, col) {
			// TODO trap_player(row as usize, col as usize, game);
		}
		reg_move(&mut game);
		return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
	}
	keep_moving_unless_hungry_or_confused(game)
}

fn stopped_on_something_with_message(desc: &str, game: RunState) -> RunStep {
	print_and_redirect(game, desc, true, |state| RunEvent::RegisterMove(state))
}

fn keep_moving_unless_hungry_or_confused(mut game: RunState) -> RunStep {
	match reg_move(&mut game) {
		RegMoveResult::Starved => {
			// TODO Might need to do something like killed_by here instead.
			RunStep::Exit(game)
		}
		RegMoveResult::Fainted => {
			game.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
			RunStep::Effect(game, RunEffect::AwaitPlayerMove)
		}
		RegMoveResult::Alert => if game.as_health().confused.is_active() {
			game.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
			RunStep::Effect(game, RunEffect::AwaitPlayerMove)
		} else {
			game.level.rogue.move_result = Some(MoveResult::Moved);
			RunStep::Effect(game, RunEffect::AwaitPlayerMove)
		},
	}
}

fn moved_on_message(row: i64, col: i64, game: &RunState) -> String {
	let obj = game.level.try_object(LevelSpot::from_i64(row, col)).unwrap();
	let obj_desc = get_obj_desc(obj, &game);
	let desc = format!("moved onto {}", obj_desc);
	desc
}
