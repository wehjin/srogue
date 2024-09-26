use crate::resources::play::event::RunEvent;
use crate::resources::play::state::RunState;
use crate::resources::play::TextConsole;
use crate::resources::player::{InputMode, PlayerInput};

pub enum RunEffect {
	Exit,
	AwaitPlayerMove,
	AwaitModalClose,
}

impl RunEffect {
	pub fn perform_await(&self, state: RunState, console: &impl TextConsole) -> Option<RunEvent> {
		let next_event = match self {
			RunEffect::Exit => return None,
			RunEffect::AwaitPlayerMove => await_player_move(state, console),
			RunEffect::AwaitModalClose => await_modal_close(state, console),
		};
		Some(next_event)
	}
}
fn await_modal_close(state: RunState, console: &impl TextConsole) -> RunEvent {
	let _input = console.get_input(InputMode::Alert);
	RunEvent::PlayerCloseModal(state)
}

fn await_player_move(state: RunState, console: &impl TextConsole) -> RunEvent {
	match console.get_input(InputMode::Any) {
		PlayerInput::Close => RunEvent::PlayerQuit(state),
		PlayerInput::Help => RunEvent::PlayerOpenHelp(state),
		PlayerInput::Menu => RunEvent::PlayerOpenInventory(state),
	}
}
