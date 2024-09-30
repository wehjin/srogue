use crate::init::Dungeon;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::seed::{EventSeed, StepSeed};
use crate::resources::play::state::RunState;
use rand::Rng;

#[derive(Debug)]
pub struct Message {
	state: RunState,
	text: String,
	interrupt: bool,
	post_step: StepSeed,
}

impl Message {
	pub fn new(state: RunState, text: impl AsRef<str>, interrupt: bool, post_step: impl FnOnce(RunState) -> RunStep + 'static) -> Message {
		Self { state, text: text.as_ref().to_string(), interrupt, post_step: StepSeed::new("post-print", post_step) }
	}
}

impl StateAction for Message {
	fn into_event(self) -> RunEvent {
		RunEvent::Message(self)
	}

	fn dispatch<R: Rng>(self, _ctx: &mut RunContext<R>) -> RunStep {
		match self {
			Self { mut state, text, interrupt, post_step } => {
				// TODO if !save_is_interactive {return;}
				let diary = state.as_diary_mut();
				if interrupt {
					diary.interrupted = true;
					// TODO md_slurp().
				}
				let diary = state.as_diary_mut();
				if diary.message_line.is_none() {
					diary.message_line = Some(text);
					diary.next_message_line = None;
					post_step.create_step(state)
				} else {
					diary.next_message_line = Some(text);
					let on_player_ack = EventSeed::new(|state| {
						RunEvent::PrintNextAndStep(state, post_step)
					});
					RunStep::Effect(state, RunEffect::AwaitAck(on_player_ack))
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::resources::play::context::RunContext;
	use crate::resources::play::event::message::Message;
	use crate::resources::play::event::state_action::StateAction;
	use crate::resources::play::event::RunStep;
	use crate::resources::play::state::RunState;
	use crate::resources::play::TextConsole;
	use crate::resources::player::{InputMode, PlayerInput};
	use rand::SeedableRng;
	use rand_chacha::ChaChaRng;

	struct TestConsole;
	impl TextConsole for TestConsole {
		fn get_input(&self, _mode: InputMode) -> PlayerInput {
			PlayerInput::Space
		}
		fn draw_lines(&mut self, _lines: Vec<String>) {}
	}

	#[test]
	fn no_previous_message_line_works() {
		let mut ctx = RunContext::new(ChaChaRng::seed_from_u64(17), TestConsole);
		let state = RunState::init(ctx.rng());
		let new_state = Message::new(state, "Hello", true, RunStep::Exit).run(&mut ctx);
		assert_eq!(Some("Hello".to_string()), new_state.diary.message_line);
		assert!(new_state.diary.interrupted);
	}
	#[test]
	fn previous_message_line_works() {
		let mut ctx = RunContext::new(ChaChaRng::seed_from_u64(17), TestConsole);
		let mut state = RunState::init(ctx.rng());
		state.diary.message_line = Some("Hello".to_string());
		let new_state = Message::new(state, "World", false, RunStep::Exit).run(&mut ctx);
		assert_eq!(Some("World".to_string()), new_state.diary.message_line);
		assert!(!new_state.diary.interrupted);
	}
}