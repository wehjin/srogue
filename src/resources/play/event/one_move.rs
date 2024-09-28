use crate::init::Dungeon;
use crate::motion::MoveDirection;
use crate::odds::R_TELE_PERCENT;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::wake::{wake_room, WakeType};
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::message::MessageEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::spot::RogueSpot;
use rand::Rng;

#[derive(Debug)]
pub struct OneMoveEvent(pub RunState, pub MoveDirection);

impl OneMoveEvent {
	pub fn into_step<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		let OneMoveEvent(state, direction) = self;
		one_move_rogue(direction, state, ctx)
	}
}

fn one_move_rogue<R: Rng>(direction: MoveDirection, mut game: RunState, ctx: &mut RunContext<R>) -> RunStep {
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
		// TODO Mark move as failed in the diary.
		return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
	}
	// What if we're stuck in place?
	{
		let begin_held = game.as_health().being_held;
		let in_bear_trap = game.as_health().bear_trap > 0;
		if begin_held || in_bear_trap {
			let monster_in_spot = game.has_monster_at(to_row, to_col);
			if !monster_in_spot {
				if begin_held {
					let message = "you are being held";
					let state = MessageEvent::dispatch(game, message, true, ctx);
					// TODO Mark move as failed in the diary.
					return RunStep::Effect(state, RunEffect::AwaitPlayerMove);
				} else {
					let message = "you are still stuck in the bear trap";
					let state = MessageEvent::dispatch(game, message, false, ctx);
					// Do a regular move here so that the bear trap counts down.
					// TODO (void) reg_move();
					// TODO Mark move as failed in the diary.
					return RunStep::Effect(state, RunEffect::AwaitPlayerMove);
				}
			}
		}
	}
	// What if we're wearing a teleport ring?
	if game.as_ring_effects().has_teleport() && ctx.roll_chance(R_TELE_PERCENT) {
		// TODO tele(game);
		// TODO Mark move as stopped-on-something in the diary.
		return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
	}
	// What if there is a monster is where we want to go?
	let monster_in_spot = game.has_monster_at(to_row, to_col);
	if monster_in_spot {
		let mon_id = game.get_monster_at(to_row, to_col).unwrap();
		// TODO rogue_hit(mon_id, false, game);
		// TODO reg_move(game);
		// TODO Mark move as failed in the diary.
		return RunStep::Effect(game, RunEffect::AwaitPlayerMove);
	}
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
	game.set_rogue_row_col(to_row, to_col);

	// TODO Take care of everything after moving the rogue (like picking up objects, hitting traps, fainting).
	RunStep::Effect(game, RunEffect::AwaitPlayerMove)
}