use crate::init::Dungeon;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::seed::EventSeed;
use crate::resources::play::state::RunState;
use rand::Rng;

#[derive(Debug)]
pub enum Message {
	PreAck { state: RunState, text: String, interrupt: bool },
	PostAck(RunState),
}

impl Message {
	fn new(state: RunState, text: impl AsRef<str>, interrupt: bool) -> Message {
		let message_event = Self::PreAck { state, text: text.as_ref().to_string(), interrupt };
		message_event
	}
	pub fn dispatch_new<R: Rng>(state: RunState, text: impl AsRef<str>, interrupt: bool, ctx: &mut RunContext<R>) -> RunState {
		let event = Self::new(state, text, interrupt).into_event();
		ctx.dispatch(event)
	}
}

impl StateAction for Message {
	fn into_event(self) -> RunEvent {
		RunEvent::Message(self)
	}

	fn dispatch<R: Rng>(self, _ctx: &mut RunContext<R>) -> RunStep {
		match self {
			Message::PreAck { mut state, text, interrupt } => {
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
					let message_event = Message::PostAck(state);
					RunStep::Redirect(RunEvent::Message(message_event))
				}
			}
			Message::PostAck(mut state) => {
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
	use crate::resources::play::event::message::Message;
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
		let new_state = Message::dispatch_new(state, "Hello", true, &mut ctx);
		assert_eq!(Some("Hello".to_string()), new_state.diary.message_line);
		assert!(new_state.diary.interrupted);
	}
	#[test]
	fn previous_message_line_works() {
		let mut ctx = RunContext::new(ChaChaRng::seed_from_u64(17), TestConsole);
		let mut state = RunState::init(ctx.rng());
		state.diary.message_line = Some("Hello".to_string());
		let new_state = Message::dispatch_new(state, "World", false, &mut ctx);
		assert_eq!(Some("World".to_string()), new_state.diary.message_line);
		assert!(!new_state.diary.interrupted);
	}
}

pub fn print_and_do<T: StateAction>(state: RunState, msg: impl AsRef<str>, interrupt: bool, action_seed: impl FnOnce(RunState) -> T + 'static) -> RunStep {
	print_and_redirect(state, msg, interrupt, |state| action_seed(state).into_event())
}

pub fn print_and_redirect(mut state: RunState, msg: impl AsRef<str>, interrupt: bool, event_seed: impl FnOnce(RunState) -> RunEvent + 'static) -> RunStep {
	// TODO if !save_is_interactive {return;}
	let diary = &mut state.diary;
	if interrupt {
		diary.interrupted = true;
		// TODO md_slurp().
	}
	if diary.message_line.is_none() {
		diary.message_line = Some(msg.as_ref().to_string());
		diary.next_message_line = None;
		RunStep::Redirect(event_seed(state))
	} else {
		diary.next_message_line = Some(msg.as_ref().to_string());
		let customer_event_seed = EventSeed::new(event_seed);
		let print_event_seed = EventSeed::new(|state| RunEvent::PrintNextAndRedirect(state, customer_event_seed));
		RunStep::Effect(state, RunEffect::DispatchAfterPlayerAck(print_event_seed))
	}
}