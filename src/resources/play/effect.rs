use crate::actions::eat::EatMealEvent;
use crate::actions::quaff::QuaffPotionEvent;
use crate::resources::play::event::game::drop_item::DropItemEvent;
use crate::resources::play::event::game::GameEventVariant;
use crate::resources::play::event::one_move::OneMoveEvent;
use crate::resources::play::event::RunEvent;
use crate::resources::play::seed::event_seed::EventSeed;
use crate::resources::play::seed::menu_event_seed::{MenuEventSeed, MenuInput};
use crate::resources::play::state::RunState;
use crate::resources::play::TextConsole;
use crate::resources::player::{InputMode, PlayerInput};

#[derive(Debug)]
pub enum RunEffect {
	AwaitMove,
	AwaitModalClose,
	AwaitAck(EventSeed),
	AwaitMenu(MenuEventSeed),
}

impl RunEffect {
	pub fn await_menu(name: impl AsRef<str>, into_event: impl FnOnce(RunState, MenuInput) -> RunEvent + 'static) -> RunEffect {
		Self::AwaitMenu(MenuEventSeed::new(name, into_event))
	}
	pub fn perform_and_await(self, state: RunState, console: &mut impl TextConsole) -> RunEvent {
		match self {
			Self::AwaitMove => await_player_move(state, console),
			Self::AwaitModalClose => await_modal_close(state, console),
			Self::AwaitAck(event_seed) => {
				let _input = console.get_input(InputMode::Alert);
				event_seed.create_event(state)
			}
			Self::AwaitMenu(event_seed) => {
				let input = console.get_input(InputMode::Menu);
				match input {
					PlayerInput::MenuSelect(ch) => event_seed.create_event(state, MenuInput::Item(ch)),
					_ => event_seed.create_event(state, MenuInput::Close),
				}
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
		PlayerInput::Arrow(direction) => OneMoveEvent::new(direction, true).into_run_event(state),
		PlayerInput::Drop => DropItemEvent::new().into_run_event(state),
		PlayerInput::Eat => EatMealEvent::new().into_run_event(state),
		PlayerInput::Help => RunEvent::PlayerOpenHelp(state),
		PlayerInput::Menu => RunEvent::PlayerOpenInventory(state),
		PlayerInput::Quaff => QuaffPotionEvent::new().into_run_event(state),
		PlayerInput::Close | PlayerInput::MenuSelect(_) | PlayerInput::Space => RunEvent::PlayerQuit(state),
	}
}
