use crate::resources::play::event::message::MessageEvent;
use crate::resources::play::event::one_move::OneMoveEvent;
use crate::resources::play::event::RunEvent;
use crate::resources::play::seed::EventSeed;
use crate::resources::play::state::RunState;
use crate::resources::play::TextConsole;
use crate::resources::player::{InputMode, PlayerInput};

#[derive(Debug)]
pub enum RunEffect {
	AwaitPlayerMove,
	AwaitModalClose,
	AwaitMessageAck,
	DispatchAfterPlayerAck(EventSeed),
}

impl RunEffect {
	pub fn perform_and_await(self, state: RunState, console: &mut impl TextConsole) -> RunEvent {
		match self {
			RunEffect::AwaitPlayerMove => await_player_move(state, console),
			RunEffect::AwaitModalClose => await_modal_close(state, console),
			RunEffect::AwaitMessageAck => await_message_ack(state, console),
			RunEffect::DispatchAfterPlayerAck(event_seed) => {
				let _input = console.get_input(InputMode::Alert);
				event_seed.into_event(state)
			}
		}
	}
}

fn await_message_ack(state: RunState, console: &impl TextConsole) -> RunEvent {
	let _input = console.get_input(InputMode::Alert);
	let message_event = MessageEvent::PostAck(state);
	RunEvent::Message(message_event)
}

fn await_modal_close(state: RunState, console: &impl TextConsole) -> RunEvent {
	let _input = console.get_input(InputMode::Alert);
	RunEvent::PlayerCloseModal(state)
}

fn await_player_move(state: RunState, console: &impl TextConsole) -> RunEvent {
	match console.get_input(InputMode::Any) {
		PlayerInput::Close | PlayerInput::Space => RunEvent::PlayerQuit(state),
		PlayerInput::Help => RunEvent::PlayerOpenHelp(state),
		PlayerInput::Menu => RunEvent::PlayerOpenInventory(state),
		PlayerInput::Arrow(direction) => RunEvent::OneMove(OneMoveEvent(state, direction)),
	}
}
