use crate::resources::avatar::Avatar;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::Message;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::menu_event_seed::MenuInput;
use crate::resources::play::state::RunState;

impl GameEventVariant for DropItemEvent {
	fn into_game_event(self) -> GameEvent {
		GameEvent::DropItem(self)
	}
}

#[derive(Debug)]
pub enum DropItemEvent {
	S1Start,
	S2Search(LevelSpot, MenuInput),
}

impl DropItemEvent {
	pub fn new() -> Self {
		Self::S1Start
	}
}

impl Dispatch for DropItemEvent {
	fn dispatch(self, mut state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			DropItemEvent::S1Start => {
				state.diary.clear_message_lines();
				match check_drop(&state) {
					CheckDrop::NowhereToDrop => {
						let after_report = |state: RunState| state.into_effect(RunEffect::AwaitMove);
						let report = "there's already something there";
						Message::new(state, report, false, after_report).into_redirect()
					}
					CheckDrop::NothingToDrop => {
						let after_report = |state: RunState| state.into_effect(RunEffect::AwaitMove);
						let report = "you have nothing to drop";
						Message::new(state, report, false, after_report).into_redirect()
					}
					CheckDrop::CanDrop(spot) => {
						state.diary.message_line = Some("drop what?".into());
						let effect = RunEffect::await_menu("drop item", move |state, input| {
							DropItemEvent::S2Search(spot, input).into_run_event(state)
						});
						state.into_effect(effect)
					}
				}
			}
			DropItemEvent::S2Search(spot, input) => {
				state.diary.clear_message_lines();
				match input {
					MenuInput::Close => {
						state.into_effect(RunEffect::AwaitMove)
					}
					MenuInput::Item(ch) => {
						state.diary.message_line = Some(format!("Picked {} to drop", ch));
						state.into_effect(RunEffect::AwaitMove)
					}
				}
			}
		}
		// match game.player.object_id_with_letter(ch) {
		// 	None => {
		// 		game.diary.add_entry("no such item.")
		// 	}
		// 	Some(obj_id) => {
		// 		if game.player.check_object(obj_id, Object::is_being_wielded) {
		// 			if game.player.check_object(obj_id, Object::is_cursed) {
		// 				game.diary.add_entry(CURSE_MESSAGE);
		// 				return;
		// 			}
		// 			pack::unwield(&mut game.player);
		// 		} else if game.player.check_object(obj_id, Object::is_being_worn) {
		// 			if game.player.check_object(obj_id, Object::is_cursed) {
		// 				game.diary.add_entry(CURSE_MESSAGE);
		// 				return;
		// 			}
		// 			mv_aquatars(game);
		// 			pack::unwear(&mut game.player);
		// 			game.as_diary_mut().set_stats_changed(true);
		// 		} else if let Some(hand) = game.player.ring_hand(obj_id) {
		// 			if game.player.check_ring(hand, Object::is_cursed) {
		// 				game.diary.add_entry(CURSE_MESSAGE);
		// 				return;
		// 			}
		// 			un_put_hand(hand, game);
		// 		}
		// 		let place_obj = if let Some(obj) = game.player.pack_mut().object_if_mut(obj_id, |obj| obj.quantity > 1 && obj.what_is != Weapon) {
		// 			obj.quantity -= 1;
		// 			let mut new = obj.clone_with_new_id(&mut thread_rng());
		// 			new.quantity = 1;
		// 			new
		// 		} else {
		// 			let mut obj = pack::take_from_pack(obj_id, &mut game.player.rogue.pack).expect("take from pack");
		// 			obj.ichar = 'L';
		// 			obj
		// 		};
		// 		let obj_desc = get_obj_desc_legacy(&place_obj, game.player.settings.fruit.to_string(), &game.player);
		// 		place_at(place_obj, game.player.rogue.row, game.player.rogue.col, &mut game.level, &mut game.ground);
		// 		game.diary.add_entry(&format!("dropped {}", obj_desc));
		// 		todo!("reg_move(game);");
		// 	}
		// }
	}
}


enum CheckDrop {
	NowhereToDrop,
	NothingToDrop,
	CanDrop(LevelSpot),
}

fn check_drop(state: &RunState) -> CheckDrop {
	let spot = *state.level.rogue.spot.as_spot();
	if has_object_stairs_or_trap(spot, state) {
		CheckDrop::NowhereToDrop
	} else if state.as_rogue_pack().is_empty() {
		CheckDrop::NothingToDrop
	} else {
		CheckDrop::CanDrop(spot)
	}
}

fn has_object_stairs_or_trap(spot: LevelSpot, state: &RunState) -> bool {
	let feature = state.level.features.feature_at(spot);
	let stairs_or_trap = feature.is_stairs() || feature.is_any_trap();
	let has_object = state.level.try_object(spot).is_some();
	let has_object_stairs_or_trap = has_object || stairs_or_trap;
	has_object_stairs_or_trap
}
