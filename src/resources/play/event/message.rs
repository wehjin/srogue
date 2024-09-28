use crate::init::Dungeon;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use rand::Rng;

#[derive(Debug)]
pub enum MessageEvent {
	PreAck { state: RunState, text: String, interrupt: bool },
	PostAck(RunState),
}

impl MessageEvent {
	pub fn dispatch<R: Rng>(state: RunState, text: impl AsRef<str>, interrupt: bool, ctx: &mut RunContext<R>) -> RunState {
		let message_event = Self::new(state, text, interrupt);
		ctx.dispatch(RunEvent::Message(message_event))
	}
	fn new(state: RunState, text: impl AsRef<str>, interrupt: bool) -> MessageEvent {
		let message_event = Self::PreAck { state, text: text.as_ref().to_string(), interrupt };
		message_event
	}

	pub fn into_step(self) -> RunStep {
		match self {
			MessageEvent::PreAck { mut state, text, interrupt } => {
				// TODO if !save_is_interactive {return;}
				if interrupt {
					let diary = state.as_diary_mut();
					diary.interrupted = true;
					// TODO md_slurp().
				}
				let diary = state.as_diary_mut();
				diary.next_message_line = Some(text);
				if state.as_diary().message_line.is_some() {
					RunStep::Effect(state, RunEffect::AwaitMessageAck)
				} else {
					let message_event = MessageEvent::PostAck(state);
					RunStep::Forward(RunEvent::Message(message_event))
				}
			}
			MessageEvent::PostAck(mut state) => {
				let diary = state.as_diary_mut();
				diary.message_line = diary.next_message_line.take();
				RunStep::Exit(state)
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
		let new_state = MessageEvent::dispatch(state, "Hello", true, &mut ctx);
		assert_eq!(Some("Hello".to_string()), new_state.diary.message_line);
		assert!(new_state.diary.interrupted);
	}
	#[test]
	fn previous_message_line_works() {
		let mut ctx = RunContext::new(ChaChaRng::seed_from_u64(17), TestConsole);
		let mut state = RunState::init(ctx.rng());
		state.diary.message_line = Some("Hello".to_string());
		let new_state = MessageEvent::dispatch(state, "World", false, &mut ctx);
		assert_eq!(Some("World".to_string()), new_state.diary.message_line);
		assert!(!new_state.diary.interrupted);
	}
}
