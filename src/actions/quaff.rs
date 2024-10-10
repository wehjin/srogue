use crate::init::Dungeon;
use crate::objects::ObjectId;
use crate::potions::kind::PotionKind;
use crate::potions::quaff::{quaff_potion, PotionEffect};
use crate::prelude::object_what::ObjectWhat::Potion;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::MessageEvent;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::upgrade_rogue::UpgradeRogueEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::menu_event_seed::MenuInput;
use crate::resources::play::state::RunState;

impl GameEventVariant for QuaffPotionEvent {
	fn into_game_event(self) -> GameEvent {
		GameEvent::QuaffPotion(self)
	}
}

#[derive(Debug)]
pub enum QuaffPotionEvent {
	S1Start,
	S2Select(MenuInput),
	S3Cancel(Option<String>),
	S4Sip(ObjectId),
	S5Gulp(ObjectId, PotionKind),
	S6End,
}

impl QuaffPotionEvent {
	pub fn new() -> Self { Self::S1Start }
}

impl Dispatch for QuaffPotionEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		match self {
			Self::S1Start => {
				state.diary.clear_message_lines();
				state.diary.message_line = Some("quaff what?".to_string());
				state.into_effect(
					RunEffect::await_menu(
						"quaff-potion",
						|state, input| Self::S2Select(input).into_run_event(state),
					)
				)
			}
			Self::S2Select(input) => {
				match input {
					MenuInput::Close => Self::S3Cancel(None).into_redirect(state),
					MenuInput::Item(letter) => {
						match state.level.rogue.obj_id_if_letter(letter) {
							None => Self::S3Cancel(Some("no such item.".to_string())).into_redirect(state),
							Some(obj_id) => Self::S4Sip(obj_id).into_redirect(state),
						}
					}
				}
			}
			Self::S3Cancel(report) => {
				state.diary.message_line = report;
				state.into_effect(RunEffect::AwaitMove)
			}
			Self::S4Sip(obj_id) => {
				state.diary.clear_message_lines();
				match state.as_rogue_pack().as_object(obj_id).potion_kind() {
					None => Self::S3Cancel(Some("you can't drink that".to_string())).into_redirect(state),
					Some(kind) => Self::S5Gulp(obj_id, kind).into_redirect(state),
				}
			}
			Self::S5Gulp(obj_id, kind) => {
				state.as_rogue_pack_mut().reduce_quantity_or_remove(obj_id);
				state.level.rogue.notes.identify_if_un_called(Potion, kind.to_index());
				state.as_diary_mut().set_stats_changed(true);

				let effect = quaff_potion(kind, &mut state);
				match effect {
					PotionEffect::None => Self::S6End.into_redirect(state),
					PotionEffect::Report(reports) => {
						let after_print = |state| Self::S6End.into_redirect(state);
						MessageEvent::multiple(state, reports, false, after_print).into_redirect()
					}
					PotionEffect::Upgrade => {
						let after_upgrade = |state| Self::S6End.into_redirect(state);
						UpgradeRogueEvent::new(after_upgrade).into_redirect(state)
					}
				}
			}
			Self::S6End => RegMoveEvent::new().into_redirect(state),
		}
	}
}

pub const STRANGE_FEELING: &'static str = "you have a strange feeling for a moment, then it passes";
