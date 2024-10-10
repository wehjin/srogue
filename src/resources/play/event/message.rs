use crate::init::Dungeon;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::seed::event_seed::EventSeed;
use crate::resources::play::seed::step_seed::StepSeed;
use crate::resources::play::state::RunState;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct MessageEvent {
	state: RunState,
	reports: VecDeque<String>,
	interrupt: bool,
	post_step: StepSeed,
	first_round: bool,
}

impl MessageEvent {
	pub fn new(state: RunState, text: impl AsRef<str>, interrupt: bool, post_step: impl FnOnce(RunState) -> RunStep + 'static) -> MessageEvent {
		Self {
			state,
			reports: VecDeque::from(vec![text.as_ref().to_string()]),
			interrupt,
			post_step: StepSeed::new("post-print", post_step),
			first_round: true,
		}
	}
	pub fn multiple(state: RunState, text: Vec<String>, interrupt: bool, post_step: impl FnOnce(RunState) -> RunStep + 'static) -> MessageEvent {
		Self {
			state,
			reports: VecDeque::from(text),
			interrupt,
			post_step: StepSeed::new("post-print", post_step),
			first_round: true,
		}
	}
	pub fn run_await_exit(state: RunState, text: impl AsRef<str>, interrupt: bool, ctx: &mut RunContext) -> RunState {
		let action = MessageEvent::new(state, text, interrupt, RunStep::Exit);
		ctx.run_action_await_exit(action)
	}
}

impl StateAction for MessageEvent {
	fn into_event(self) -> RunEvent {
		RunEvent::Message(self)
	}

	fn dispatch(self, _ctx: &mut RunContext) -> RunStep {
		match self {
			Self { mut state, mut reports, interrupt, post_step, first_round } => {
				// TODO if !save_is_interactive {return;}
				let diary = state.as_diary_mut();
				if interrupt {
					diary.interrupted = true;
					// TODO md_slurp().
				}
				if first_round {
					if diary.message_line.is_none() {
						diary.message_line = reports.pop_front();
						diary.next_message_line = reports.pop_front();
					} else {
						diary.next_message_line = reports.pop_front();
					}
				} else {
					diary.message_line = diary.next_message_line.take();
					diary.next_message_line = reports.pop_front();
				}
				if diary.next_message_line.is_none() {
					post_step.into_step(state)
				} else {
					let after_ack = EventSeed::new(move |state| {
						Self { state, reports, interrupt, post_step, first_round: false }.into_event()
					});
					state.into_effect(RunEffect::AwaitAck(after_ack))
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::resources::play::context::RunContext;
	use crate::resources::play::event::message::MessageEvent;
	use crate::resources::play::state::RunState;
	use crate::resources::play::TextConsole;
	use crate::resources::player::{InputMode, PlayerInput};
	use rand::SeedableRng;
	use rand_chacha::ChaCha8Rng;

	struct TestConsole;
	impl TextConsole for TestConsole {
		fn get_input(&self, _mode: InputMode) -> PlayerInput {
			PlayerInput::Space
		}
		fn draw_lines(&mut self, _lines: Vec<String>) {}
	}

	#[test]
	fn no_previous_message_line_works() {
		let rng = ChaCha8Rng::seed_from_u64(17);
		let mut ctx = RunContext::new(TestConsole);
		let state = RunState::init(rng);
		let new_state = MessageEvent::run_await_exit(state, "Hello", true, &mut ctx);
		assert_eq!(Some("Hello".to_string()), new_state.diary.message_line);
		assert!(new_state.diary.interrupted);
	}
	#[test]
	fn previous_message_line_works() {
		let rng = ChaCha8Rng::seed_from_u64(17);
		let mut ctx = RunContext::new(TestConsole);
		let mut state = RunState::init(rng);
		state.diary.message_line = Some("Hello".to_string());
		let new_state = MessageEvent::run_await_exit(state, "World", false, &mut ctx);
		assert_eq!(Some("World".to_string()), new_state.diary.message_line);
		assert!(!new_state.diary.interrupted);
	}
}