use crate::objects::ObjectId;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::ObjectWhat;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::upgrade_rogue::UpgradeRogueEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::menu_event_seed::MenuInput;
use crate::resources::play::state::RunState;
use rand::Rng;

impl GameEventVariant for EatMealEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::EatMeal(self) }
}

#[derive(Debug)]
pub enum EatMealEvent {
	S1Start,
	S2Select(MenuInput),
	S3Chew(ObjectId),
	S4Swallow(ObjectId),
}

impl EatMealEvent {
	pub fn new() -> Self { Self::S1Start }
}

impl Dispatch for EatMealEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		match self {
			Self::S1Start => {
				state.diary.clear_message_lines();
				state.diary.message_line = Some("eat what?".to_string());
				state.into_effect(
					RunEffect::await_menu(
						"eat",
						|state, input| Self::S2Select(input).into_run_event(state),
					)
				)
			}
			Self::S2Select(input) => {
				match input {
					MenuInput::Close => state.into_effect(RunEffect::AwaitMove),
					MenuInput::Item(ch) => {
						match state.level.rogue.obj_id_if_letter(ch) {
							None => {
								state.diary.message_line = Some("no such item.".to_string());
								state.into_effect(RunEffect::AwaitMove)
							}
							Some(obj_id) => {
								Self::S3Chew(obj_id).into_redirect(state)
							}
						}
					}
				}
			}
			Self::S3Chew(obj_id) => {
				if state.pack_object(obj_id).what_is != ObjectWhat::Food {
					state.diary.message_line = Some("you can't eat that".to_string());
					state.into_effect(RunEffect::AwaitMove)
				} else {
					Self::S4Swallow(obj_id).into_redirect(state)
				}
			}
			Self::S4Swallow(obj_id) => {
				let meal = Meal::prepare(state.pack_object(obj_id).which_kind, &mut state);
				{
					let fighter = state.as_fighter_mut();
					fighter.moves_left = fighter.moves_left / 3 + meal.calories;
					state.diary.set_stats_changed(true);
				}
				if state.pack_object(obj_id).quantity > 1 {
					let object = state.pack_object_mut(obj_id);
					object.quantity -= 1;
				} else {
					let pack = state.as_rogue_pack_mut();
					pack.remove(obj_id);
				}
				state.diary.message_line = Some(meal.message);
				if meal.gains_exp {
					state.as_fighter_mut().exp.add_points(2);
					let after_report = |state| RegMoveEvent::new().into_redirect(state);
					UpgradeRogueEvent::new(after_report).into_redirect(state)
				} else {
					RegMoveEvent::new().into_redirect(state)
				}
			}
		}
	}
}

struct Meal {
	calories: isize,
	message: String,
	gains_exp: bool,
}

impl Meal {
	pub fn prepare(kind: u16, state: &mut RunState) -> Self {
		if kind == FRUIT || state.roll_chance(60) {
			Self {
				calories: state.rng.gen_range(900..=1100),
				message: if kind == RATION {
					"yum, that tasted good".to_string()
				} else {
					format!("my, that was a yummy {}", &state.settings.fruit)
				},
				gains_exp: false,
			}
		} else {
			Self {
				calories: state.rng.gen_range(700..=900),
				message: "yuk, that food tasted awful".to_string(),
				gains_exp: true,
			}
		}
	}
}
