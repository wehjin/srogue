use crate::init::Dungeon;
use crate::level::hp_raise;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::MessageEvent;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::step_seed::StepSeed;
use crate::resources::play::state::RunState;

impl GameEventVariant for UpgradeRogueEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::UpgradeRogue(self) }
}


#[derive(Debug)]
pub struct UpgradeRogueEvent {
	after_upgrade: StepSeed,
}

impl UpgradeRogueEvent {
	pub fn new(after_upgrade: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self { after_upgrade: StepSeed::new("upgrade-rogue", after_upgrade) }
	}
}

impl Dispatch for UpgradeRogueEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		let Self { after_upgrade } = self;
		if let Some(promotion_level) = state.as_fighter().exp.can_promote() {
			// Update rogue's hp and exp level.
			let hp = hp_raise(state.wizard(), state.rng());
			state.upgrade_hp(hp);
			state.as_fighter_mut().exp.set_level(promotion_level);
			state.as_diary_mut().set_stats_changed(true);
			// Report the promotion then try again since we may have more upgrades to consider.
			let post_report = |state| Self { after_upgrade }.into_redirect(state);
			let report = format!("welcome to level {}", promotion_level);
			MessageEvent::new(state, report, false, post_report).into_redirect()
		} else {
			// Done upgrading.
			after_upgrade.into_step(state)
		}
	}
}

