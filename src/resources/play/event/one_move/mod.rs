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
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::MessageEvent;
use crate::resources::play::event::one_move::OneMoveEvent::{Stage1Start, Stage2CheckStuck, Stage3CheckTeleport, Stage4AUpgradeRogue, Stage4CheckMonster, Stage5AdjustLighting, Stage6MoveRogue, Stage7PickupObjects, Stage8CheckStoppedAndTraps};
use crate::resources::play::event::pick_up::{PickUpRegMove, PickupType};
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::state_action::{redirect, StateAction};
use crate::resources::play::event::upgrade_rogue::UpgradeRogueEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::spot::RogueSpot;

#[derive(Debug)]
pub enum OneMoveEvent {
	Stage1Start { direction: MoveDirection, allow_pickup: bool },
	Stage2CheckStuck { to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	Stage3CheckTeleport { to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	Stage4CheckMonster { to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	Stage4AUpgradeRogue,
	Stage5AdjustLighting { to_spot: (i64, i64), rogue_spot: (i64, i64), allow_pickup: bool },
	Stage6MoveRogue { to_spot: (i64, i64), allow_pickup: bool },
	Stage7PickupObjects { spot: (i64, i64), allow_pickup: bool },
	Stage8CheckStoppedAndTraps { spot: (i64, i64) },
}

impl OneMoveEvent {
	pub fn new(direction: MoveDirection, allow_pickup: bool) -> Self {
		Stage1Start { direction, allow_pickup }
	}
}

impl Dispatch for OneMoveEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		match self {
			Stage1Start { direction, allow_pickup } => {
				state.move_result = None;
				state.diary.clear_message_lines();
				// Where are we?
				let rogue_spot = (state.rogue_row(), state.rogue_col());
				// Where are we going?
				let to_spot = get_destination_spot(direction, rogue_spot.0, rogue_spot.1, &mut state);
				// Can we get there?
				let navigable = state.rogue_can_move(to_spot.0, to_spot.1);
				if !navigable {
					// No. Done.
					state.move_result = Some(MoveResult::MoveFailed);
					state.into_effect(RunEffect::AwaitMove)
				} else {
					// Yes. See if we can move.
					Stage2CheckStuck { to_spot, rogue_spot, allow_pickup }.into_redirect(state)
				}
			}
			Stage2CheckStuck { to_spot, rogue_spot, allow_pickup } => {
				// What if we're stuck in place?
				let no_monster_at_spot = !state.has_monster_at(to_spot.0, to_spot.1);
				let in_bear_trap = state.as_health().bear_trap > 0;
				let being_held = state.as_health().being_held;
				if being_held && no_monster_at_spot {
					// Report held status and wait for next move.
					state.move_result = Some(MoveResult::MoveFailed);
					let after_report = |state| RunStep::Effect(state, RunEffect::AwaitMove);
					let report = "you are being held";
					redirect(MessageEvent::new(state, report, true, after_report))
				} else if in_bear_trap && no_monster_at_spot {
					// Report bear trap and do a regular move so that the bear trap counts down.
					state.move_result = Some(MoveResult::MoveFailed);
					let after_report = |state| RegMoveEvent::new().into_redirect(state);
					let report = "you are still stuck in the bear trap";
					redirect(MessageEvent::new(state, report, true, after_report))
				} else {
					// On to the next stage.
					Stage3CheckTeleport { to_spot, rogue_spot, allow_pickup }.into_redirect(state)
				}
			}
			Stage3CheckTeleport { to_spot, rogue_spot, allow_pickup } => {
				// What if we're wearing a teleport ring?
				let wearing_teleport_ring = state.as_ring_effects().has_teleport();
				if wearing_teleport_ring && state.roll_chance(R_TELE_PERCENT) {
					// Teleport.
					state.move_result = Some(MoveResult::StoppedOnSomething);
					// TODO tele(game);
					state.into_effect(RunEffect::AwaitMove)
				} else {
					// No teleport. On to the next stage.
					Stage4CheckMonster { to_spot, rogue_spot, allow_pickup }.into_redirect(state)
				}
			}
			Stage4CheckMonster { to_spot, rogue_spot, allow_pickup } => {
				// What if there is a monster is where we want to go?
				let monster_in_spot = state.has_monster_at(to_spot.0, to_spot.1);
				if monster_in_spot {
					// Monster. Strike monster and finish the move.
					state.move_result = Some(MoveResult::MoveFailed);
					let mon_id = state.get_monster_at(to_spot.0, to_spot.1).unwrap();
					state = rogue_hit(state, mon_id, false);
					if let Some(report) = state.diary.next_message_line.take() {
						// If there is something to report, report it then finish the move.
						let after_report = |state| Stage4AUpgradeRogue.into_redirect(state);
						MessageEvent::new(state, report, true, after_report).into_redirect()
					} else {
						// Finish the move.
						Stage4AUpgradeRogue.into_redirect(state)
					}
				} else {
					// No monster. Go to next stage.
					Stage5AdjustLighting { to_spot, rogue_spot, allow_pickup }.into_redirect(state)
				}
			}
			Stage4AUpgradeRogue => {
				let after_upgrade = |state| RegMoveEvent::new().into_redirect(state);
				UpgradeRogueEvent::new(after_upgrade).into_redirect(state)
			}
			Stage5AdjustLighting { to_spot, rogue_spot, allow_pickup } => {
				// Adjust lighting.
				if state.is_any_door_at(to_spot.0, to_spot.1) {
					// x to door
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
				// On to next stage.
				Stage6MoveRogue { to_spot, allow_pickup }.into_redirect(state)
			}
			Stage6MoveRogue { to_spot, allow_pickup } => {
				// Move the rogue then pick-up stage.
				state.set_rogue_row_col(to_spot.0, to_spot.1);
				Stage7PickupObjects { spot: to_spot, allow_pickup }.into_redirect(state)
			}
			Stage7PickupObjects { spot, allow_pickup } => {
				// Pick up objects.
				let has_object = state.level.try_object(LevelSpot::from(spot)).is_some();
				if has_object {
					let (row, col) = spot;
					if allow_pickup {
						// Pick up the object and complete the move.
						let spot = LevelSpot::from_i64(row, col);
						PickUpRegMove(state, PickupType::AfterMove(spot)).into_redirect()
					} else {
						// Leave the object alone but stop moving and report we're on top of it. Then complete the move.
						state.move_result = Some(MoveResult::StoppedOnSomething);
						let after_report = move |state| RegMoveEvent::new().into_redirect(state);
						let report = moved_onto_message(row, col, &state);
						MessageEvent::new(state, report, true, after_report).into_redirect()
					}
				} else {
					Stage8CheckStoppedAndTraps { spot }.into_redirect(state)
				}
			}
			Stage8CheckStoppedAndTraps { spot: (row, col) } => {
				// Are we on something interesting?
				let is_door = state.is_any_door_at(row, col);
				let is_stairs = state.level.features.feature_at(LevelSpot::from_i64(row, col)).is_stairs();
				let is_trap = state.is_any_trap_at(row, col);
				if is_door || is_stairs || is_trap {
					// Have we hit a trap?
					let not_levitating = state.as_health().levitate.is_inactive();
					if is_trap && not_levitating {
						// TODO trap_player(row as usize, col as usize, game);
					}
					// Interesting spot. Stop to smell the roses.
					state.move_result = Some(MoveResult::StoppedOnSomething);
				} else {
					// RegMove will decide the value of [move_result].
				}
				RegMoveEvent::new().into_redirect(state)
			}
		}
	}
}

impl GameEventVariant for OneMoveEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::OneMove(self) }
}

fn get_destination_spot(direction: MoveDirection, from_row: i64, from_col: i64, state: &mut RunState) -> (i64, i64) {
	let confused = state.as_health().confused.is_active();
	let confused_direction = if !confused { direction } else { MoveDirection::random(state.rng()) };
	confused_direction.apply(from_row, from_col)
}

pub fn moved_onto_message(row: i64, col: i64, state: &RunState) -> String {
	let obj = state.level.try_object(LevelSpot::from_i64(row, col)).unwrap();
	let obj_desc = get_obj_desc(obj, &state);
	let desc = format!("moved onto {}", obj_desc);
	desc
}
