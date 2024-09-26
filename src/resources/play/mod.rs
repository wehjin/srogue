use crate::resources::player::{InputMode, PlayerInput};
use effect::RunEffect;
use event::{RunEvent, RunStep};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use state::RunState;

pub mod effect;
pub mod event;
pub mod state;

pub fn run(mut console: impl TextConsole) {
	let mut rng = ChaChaRng::from_entropy();
	let mut next_event = Some(RunEvent::Init);
	while let Some(event) = next_event.take() {
		let RunStep { state, effect } = event.dispatch(&mut rng);
		next_event = render_perform_await(state, effect, &mut console);
	}
}

fn render_perform_await(state: RunState, effect: RunEffect, console: &mut impl TextConsole) -> Option<RunEvent> {
	console.draw_lines(state.to_lines());
	let next_event = effect.perform_await(state, console);
	next_event
}

pub trait TextConsole {
	fn get_input(&self, mode: InputMode) -> PlayerInput;
	fn draw_lines(&mut self, lines: Vec<String>);
}
