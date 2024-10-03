use crate::resources::play::event::game::GameEventVariant;
use crate::resources::play::event::one_move::OneMoveEvent;
use crate::resources::play::event::RunEvent;
use crate::resources::play::seed::EventSeed;
use crate::resources::play::state::RunState;
use crate::resources::play::TextConsole;
use crate::resources::player::{InputMode, PlayerInput};

#[derive(Debug)]
pub enum RunEffect {
	AwaitMove,
	AwaitModalClose,
	AwaitAck(EventSeed),
}

impl RunEffect {
	pub fn perform_and_await(self, state: RunState, console: &mut impl TextConsole) -> RunEvent {
		match self {
			RunEffect::AwaitMove => await_player_move(state, console),
			RunEffect::AwaitModalClose => await_modal_close(state, console),
			RunEffect::AwaitAck(event_seed) => {
				let _input = console.get_input(InputMode::Alert);
				event_seed.create_event(state)
			}
		}
	}
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
		PlayerInput::Arrow(direction) => OneMoveEvent::new(direction, true).into_run_event(state),
	}
}
