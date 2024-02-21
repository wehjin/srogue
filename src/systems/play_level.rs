use crate::init::GameState;
use crate::render_system;
use crate::render_system::RenderAction;
use crate::resources::keyboard::{CANCEL_CHAR, rgetchar};
use crate::state::input::{KEY_REST, KEY_SEARCH};
use crate::state::player::PlayState;
use crate::systems::play_level::PlayResult::CleanedUp;
use crate::systems::play_once::{play_once, PlayOnceResult};

pub const UNKNOWN_COMMAND: &'static str = "unknown command";

#[derive(Clone, Debug)]
pub enum PlayResult {
	TrapDoorDown,
	StairsDown,
	StairsUp,
	ExitWon,
	ExitQuit,
	ExitSaved,
	CleanedUp(String),
}

pub fn play_level(game: &mut GameState) -> PlayResult {
	game.render(&[RenderAction::Init]);
	render_system::refresh(game);
	let mut player_state = PlayState::Idle;
	loop {
		if let Some(exit) = &game.player.cleaned_up {
			return CleanedUp(exit.to_string());
		}
		let next_state = match player_state {
			PlayState::Idle => when_idle(game),
			PlayState::Counting(digits) => when_counting(digits),
			PlayState::Busy(key_code, count) => when_busy(key_code, count, game),
			PlayState::Leaving(ending) => return ending.clone(),
		};
		player_state = next_state;
	}
}

fn when_idle(game: &mut GameState) -> PlayState {
	match play_once(None, game) {
		PlayOnceResult::Leaving(ending) => PlayState::Leaving(ending),
		PlayOnceResult::Counting(digits) => PlayState::Counting(digits),
		PlayOnceResult::Idle => PlayState::Idle,
	}
}

fn when_counting(digits: String) -> PlayState {
	let next_key = rgetchar();
	match next_key {
		CANCEL_CHAR => PlayState::Idle,
		'0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
			let mut new_digits = digits.clone();
			new_digits.push(next_key);
			PlayState::Counting(new_digits)
		}
		_ => {
			let count = digits.parse().expect(&format!("digits should parse {digits}"));
			PlayState::Busy(next_key, count)
		}
	}
}

fn when_busy(key_code: char, count: usize, game: &mut GameState) -> PlayState {
	if count == 0 {
		PlayState::Idle
	} else if job_is_repeatable(key_code) {
		match play_once(Some(key_code), game) {
			PlayOnceResult::Counting(_) => panic!("can't count while busy"),
			PlayOnceResult::Leaving(ending) => PlayState::Leaving(ending),
			PlayOnceResult::Idle => {
				if game.player.interrupted {
					PlayState::Idle
				} else {
					let lower_count = count - 1;
					if lower_count == 0 {
						PlayState::Idle
					} else {
						PlayState::Busy(key_code, lower_count)
					}
				}
			}
		}
	} else {
		match play_once(Some(key_code), game) {
			PlayOnceResult::Counting(_) => panic!("can't count while busy"),
			PlayOnceResult::Leaving(ending) => PlayState::Leaving(ending),
			PlayOnceResult::Idle => PlayState::Idle,
		}
	}
}

fn job_is_repeatable(key_code: char) -> bool {
	key_code == KEY_REST || key_code == KEY_SEARCH
}