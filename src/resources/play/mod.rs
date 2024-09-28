use crate::resources::player::{InputMode, PlayerInput};
use context::RunContext;
use event::{RunEvent, RunStep};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use state::RunState;

pub mod context;
pub mod effect;
pub mod event;
pub mod state;

pub fn run(console: impl TextConsole + 'static) {
	let mut ctx = RunContext::new(ChaChaRng::from_entropy(), console);
	dispatch(RunEvent::Init, &mut ctx);
}

pub fn dispatch<R: Rng>(start_event: RunEvent, ctx: &mut RunContext<R>) -> RunState {
	let mut next_event = start_event;
	loop {
		match next_event.dispatch(ctx) {
			RunStep::Exit(state) => {
				return state;
			}
			RunStep::Forward(event) => {
				next_event = event;
			}
			RunStep::Effect(state, effect) => {
				let console = ctx.console();
				console.draw_lines(state.to_lines());
				next_event = effect.perform_and_await(state, console);
			}
		}
	}
}

pub trait TextConsole {
	fn get_input(&self, mode: InputMode) -> PlayerInput;
	fn draw_lines(&mut self, lines: Vec<String>);
}

impl TextConsole for Box<dyn TextConsole> {
	fn get_input(&self, mode: InputMode) -> PlayerInput {
		self.as_ref().get_input(mode)
	}
	fn draw_lines(&mut self, lines: Vec<String>) {
		self.as_mut().draw_lines(lines)
	}
}
