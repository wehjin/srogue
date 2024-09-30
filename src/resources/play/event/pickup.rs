use crate::init::Dungeon;
use crate::inventory::get_obj_desc;
use crate::motion::MoveResult;
use crate::objects::NoteStatus::{Identified, Unidentified};
use crate::pack::{PickUpResult, MAX_PACK_COUNT};
use crate::resources::avatar::Avatar;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::message::{print_and_do, Message};
use crate::resources::play::event::one_move::moved_onto_message;
use crate::resources::play::event::reg_move::RegMove;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use crate::scrolls::ScrollKind::ScareMonster;
use rand::Rng;

#[derive(Debug)]
pub enum PickupType {
	AfterMove(LevelSpot),
}

#[derive(Debug)]
pub struct Pickup(pub RunState, pub PickupType);

impl StateAction for Pickup {
	fn into_event(self) -> RunEvent {
		RunEvent::Pickup(self)
	}
	fn dispatch<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		let Pickup(mut state, pickup_type) = self;

		match pickup_type {
			PickupType::AfterMove(spot) => {
				let (row, col) = spot.into();
				match state.as_health().levitate.is_active() {
					true => {
						state.level.rogue.move_result = Some(MoveResult::StoppedOnSomething);
						RunStep::Effect(state, RunEffect::AwaitPlayerMove)
					}
					false => match pick_up(row, col, state, ctx) {
						(PickUpResult::TurnedToDust, state) => {
							register_move(state, ctx)
						}
						(PickUpResult::AddedToGold(obj), state) => {
							let object_desc = get_obj_desc(&obj, &state);
							let move_result = Some(MoveResult::StoppedOnSomething);
							print_message_register_move(object_desc, move_result, state)
						}
						(PickUpResult::AddedToPack { added_id, .. }, state) => {
							let object_desc_with_item_handle = state.get_rogue_obj_desc(added_id);
							let move_result = Some(MoveResult::StoppedOnSomething);
							print_message_register_move(object_desc_with_item_handle, move_result, state)
						}
						(PickUpResult::PackTooFull, state) => {
							let moved_onto_message = moved_onto_message(row, col, &state);
							let move_result = Some(MoveResult::StoppedOnSomething);
							print_message_register_move(moved_onto_message, move_result, state)
						}
					},
				}
			}
		}
	}
}

fn register_move<R: Rng>(state: RunState, ctx: &mut RunContext<R>) -> RunStep {
	RegMove(state, None).dispatch(ctx)
}

fn print_message_register_move(message: impl AsRef<str>, move_result: Option<MoveResult>, state: RunState) -> RunStep {
	print_and_do(state, message.as_ref(), true, RegMove::delay_state(move_result))
}

fn pick_up<R: Rng>(row: i64, col: i64, mut game: RunState, ctx: &mut RunContext<R>) -> (PickUpResult, RunState) {
	let obj = game.try_object_at(row, col).unwrap();
	if obj.is_used_scare_monster_scroll() {
		let mut state = Message::dispatch_new(game, "the scroll turns to dust as you pick it up", false, ctx);
		state.level.remove_object(LevelSpot::from_i64(row, col));
		if state.as_notes().scrolls[ScareMonster.to_index()].status == Unidentified {
			let notes = state.as_notes_mut();
			notes.scrolls[ScareMonster.to_index()].status = Identified
		}
		return (PickUpResult::TurnedToDust, state);
	}
	if let Some(quantity) = obj.gold_quantity() {
		game.as_fighter_mut().gold += quantity;
		let removed = game.level.remove_object(LevelSpot::from_i64(row, col)).unwrap();
		game.as_diary_mut().set_stats_changed(true);
		return (PickUpResult::AddedToGold(removed), game);
	}
	if game.pack_weight_with_new_object(Some(obj)) >= MAX_PACK_COUNT {
		game.interrupt_and_slurp();
		let state = Message::dispatch_new(game, "pack too full", true, ctx);
		return (PickUpResult::PackTooFull, state);
	}
	let removed = game.level.remove_object(LevelSpot::from_i64(row, col)).unwrap();
	let added_id = game.combine_or_add_item_to_pack(removed);
	let added_kind = {
		let obj = game.as_fighter_mut().pack.object_mut(added_id).unwrap();
		obj.picked_up = 1;
		obj.which_kind
	};
	(PickUpResult::AddedToPack { added_id, added_kind: added_kind as usize }, game)
}