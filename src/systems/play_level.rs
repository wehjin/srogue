use crate::init::GameState;
use crate::render_system;
use crate::render_system::RenderAction;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::state::input::{KEY_REST, KEY_SEARCH};
use crate::state::player::PlayState;
use crate::systems::play_level::LevelResult::CleanedUp;
use crate::systems::play_once::{play_once, OnceResult};

pub const UNKNOWN_COMMAND: &'static str = "unknown command";

#[derive(Clone, Debug)]
pub enum LevelResult {
	TrapDoorDown,
	StairsDown,
	StairsUp,
	ExitWon,
	ExitQuit,
	ExitSaved,
	CleanedUp(String),
}

pub fn play_level(game: &mut GameState) -> LevelResult {
	game.render(&[RenderAction::Init]);
	render_system::refresh(game);
	let mut player_state = PlayState::Idle;
	loop {
		if let Some(exit) = &game.player.cleaned_up {
			return CleanedUp(exit.to_string());
		}
		let next_state = match player_state {
			PlayState::Idle => match play_once(None, game) {
				OnceResult::Leaving(ending) => PlayState::Leaving(ending),
				OnceResult::Counting(digits) => PlayState::Counting(digits),
				OnceResult::Idle => PlayState::Idle,
			},
			PlayState::Counting(digits) => when_counting(digits),
			PlayState::Busy { key_code, completed, remaining } => {
				when_busy(key_code, completed, remaining, game)
			}
			PlayState::Leaving(ending) => return ending.clone(),
		};
		player_state = next_state;
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
			PlayState::Busy {
				key_code: next_key,
				completed: 0,
				remaining: count,
			}
		}
	}
}

fn when_busy(key_code: char, completed: usize, remaining: usize, game: &mut GameState) -> PlayState {
	if remaining == 0 {
		PlayState::Idle
	} else if job_is_repeatable(key_code) {
		match play_once(Some(key_code), game) {
			OnceResult::Counting(_) => panic!("can't count while busy"),
			OnceResult::Leaving(ending) => PlayState::Leaving(ending),
			OnceResult::Idle => {
				if game.player.interrupted || remaining == 1 {
					PlayState::Idle
				} else {
					PlayState::Busy {
						key_code,
						completed: completed + 1,
						remaining: remaining - 1,
					}
				}
			}
		}
	} else {
		match play_once(Some(key_code), game) {
			OnceResult::Counting(_) => panic!("can't count while busy"),
			OnceResult::Leaving(ending) => PlayState::Leaving(ending),
			OnceResult::Idle => PlayState::Idle,
		}
	}
}

fn job_is_repeatable(key_code: char) -> bool {
	key_code == KEY_REST || key_code == KEY_SEARCH
}