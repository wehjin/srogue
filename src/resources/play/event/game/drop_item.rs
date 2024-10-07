use crate::init::Dungeon;
use crate::inventory::get_obj_desc;
use crate::monster::mv_aquatars;
use crate::objects::{Object, ObjectId};
use crate::pack::{take_from_pack, CURSE_MESSAGE};
use crate::prelude::object_what::ObjectWhat;
use crate::resources::avatar::Avatar;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::Message;
use crate::resources::play::event::reg_move::RegMoveEvent;
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
	S2SearchPack(LevelSpot, MenuInput),
	S3CheckWeapon(ObjectId),
	S4CheckArmor(ObjectId),
	S5CheckRing(ObjectId),
	S6DropItem(ObjectId),
}

impl DropItemEvent {
	pub fn new() -> Self {
		Self::S1Start
	}
}

impl Dispatch for DropItemEvent {
	fn dispatch(self, mut state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			Self::S1Start => {
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
							DropItemEvent::S2SearchPack(spot, input).into_run_event(state)
						});
						state.into_effect(effect)
					}
				}
			}
			Self::S2SearchPack(spot, input) => {
				state.diary.clear_message_lines();
				match input {
					MenuInput::Close => {
						state.into_effect(RunEffect::AwaitMove)
					}
					MenuInput::Item(ch) => match state.level.rogue.obj_id_if_letter(ch) {
						None => {
							state.diary.message_line = Some("no such item.".to_string());
							state.into_effect(RunEffect::AwaitMove)
						}
						Some(obj_id) => {
							Self::S6DropItem(obj_id).into_redirect(state)
						}
					},
				}
			}
			Self::S3CheckWeapon(obj_id) => {
				state.diary.clear_message_lines();
				let wielded = state.level.rogue.check_object(obj_id, Object::is_being_wielded);
				match wielded {
					true => {
						let cursed = state.level.rogue.check_object(obj_id, Object::is_cursed);
						if cursed {
							state.diary.message_line = Some(CURSE_MESSAGE.to_string());
							state.into_effect(RunEffect::AwaitMove)
						} else {
							state.unwield_weapon();
							Self::S6DropItem(obj_id).into_redirect(state)
						}
					}
					false => Self::S4CheckArmor(obj_id).into_redirect(state),
				}
			}
			Self::S4CheckArmor(obj_id) => {
				state.diary.clear_message_lines();
				let worn = state.level.rogue.check_object(obj_id, Object::is_being_worn);
				match worn {
					true => {
						let cursed = state.level.rogue.check_object(obj_id, Object::is_cursed);
						if cursed {
							state.diary.message_line = Some(CURSE_MESSAGE.to_string());
							state.into_effect(RunEffect::AwaitMove)
						} else {
							let mut state = mv_aquatars(state);
							state.unwear_armor();
							state.as_diary_mut().set_stats_changed(true);
							Self::S6DropItem(obj_id).into_redirect(state)
						}
					}
					false => Self::S5CheckRing(obj_id).into_redirect(state),
				}
			}
			Self::S5CheckRing(obj_id) => {
				let ring_hand = state.ring_hand(obj_id);
				match ring_hand {
					Some(hand) => {
						let cursed = state.check_ring(hand, Object::is_cursed);
						if cursed {
							state.diary.message_line = Some(CURSE_MESSAGE.to_string());
							state.into_effect(RunEffect::AwaitMove)
						} else {
							state.un_put_ring(hand);
							Self::S6DropItem(obj_id).into_redirect(state)
						}
					}
					None => Self::S6DropItem(obj_id).into_redirect(state),
				}
			}
			Self::S6DropItem(obj_id) => {
				let object = take_object(obj_id, &mut state);
				let object_desc = get_obj_desc(&object, &state);
				let spot = *state.level.rogue.spot.as_spot();
				state.level.put_object(spot, object);
				let report = format!("dropped {}", object_desc);
				let after_report = |state| RegMoveEvent::new().into_redirect(state);
				Message::new(state, report, false, after_report).into_redirect()
			}
		}
	}
}

fn take_object(obj_id: ObjectId, state: &mut RunState) -> Object {
	let can_lower_quantity = |obj: &Object| obj.quantity > 1 && obj.what_is != ObjectWhat::Weapon;
	match state.as_rogue_pack_mut().object_if_mut(obj_id, can_lower_quantity) {
		Some(obj) => {
			obj.quantity -= 1;
			let mut new = obj.clone();
			new.id = ObjectId::random(state.rng());
			new.quantity = 1;
			new
		}
		None => {
			let mut obj = take_from_pack(obj_id, state.as_rogue_pack_mut()).expect("take from pack");
			obj.ichar = 'L';
			obj
		}
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
