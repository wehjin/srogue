use crate::resources::player::{InputMode, PlayerInput};
use context::RunContext;
use event::{RunEvent, RunStep};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use state::RunState;

pub mod context;
pub mod effect;
pub mod event;
pub mod seed;
pub mod state;

pub fn run(console: impl TextConsole + 'static) {
	let rng = ChaCha8Rng::from_entropy();
	let mut ctx = RunContext::new(console);
	dispatch(RunEvent::Init(rng), &mut ctx);
}

pub fn dispatch(start_event: RunEvent, ctx: &mut RunContext) -> RunState {
	let mut next_event = start_event;
	loop {
		match next_event.dispatch(ctx) {
			RunStep::Exit(state) => {
				assert_eq!(&None, &state.diary.cleaned_up);
				return state;
			}
			RunStep::Redirect(event) => {
				next_event = event;
			}
			RunStep::Effect(state, effect) => {
				assert_eq!(&None, &state.diary.cleaned_up);
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
