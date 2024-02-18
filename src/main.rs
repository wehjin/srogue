#![feature(extern_types)]

use crate::console::ConsoleError;
use crate::init::{init, InitError, InitResult};
use crate::level::{clear_level, make_level, put_player};
use crate::message::print_stats;
use crate::monster::put_mons;
use crate::objects::{put_objects, put_stairs};
use crate::play::{play_level, PlayResult};
use crate::prelude::stat_const::STAT_ALL;
use crate::settings::SettingsError;
use crate::trap::add_traps;

pub mod components;
pub mod resources;
mod message;
mod level;
mod monster;
mod hit;
mod init;
mod inventory;
mod machdep;
mod motion;
mod objects;
mod pack;
mod potions;
mod play;
mod player;
mod random;
mod render_system;
mod ring;
mod room;
mod save;
mod score;
mod spec_hit;
mod throw;
mod trap;
mod r#use;
mod zap;
mod prelude;
mod weapons;
mod armors;
mod scrolls;

pub mod odds;
pub mod console;
pub mod settings;
pub(crate) mod files;
pub mod actions;

pub fn main() {
	let settings = match settings::load() {
		Ok(settings) => settings,
		Err(e) => {
			match e {
				SettingsError::LoginName => {
					println!("Hey!  Who are you?");
				}
			}
			return;
		}
	};
	let result = match init(settings) {
		Ok(result) => result,
		Err(error) => match error {
			InitError::NoConsole(error) => {
				match error {
					ConsoleError::ScreenTooSmall { min_rows, min_cols } => {
						println!();
						println!("must be played on {} x {} or better screen", min_rows, min_cols);
					}
				}
				return;
			}
			InitError::BadRestore(exit) => {
				if let Some(exit) = exit {
					eprintln!("\n{}", exit);
				}
				return;
			}
		},
	};
	let (mut game, console, mut restored) = match result {
		InitResult::ScoreOnly(_player, _console, _settings) => {
			return;
		}
		InitResult::Restored(game, console) => (game, console, true),
		InitResult::Initialized(game, console) => (game, console, false),
	};
	let mut exit_line = None;
	loop {
		if !restored {
			clear_level(&mut game);
			game.player.descend();
			make_level(&mut game);
			put_objects(&mut game);
			put_stairs(&mut game.player, &mut game.level);
			add_traps(&game.player, &mut game.level);
			put_mons(&mut game);
			put_player(game.level.party_room.into(), &mut game);
			print_stats(STAT_ALL, &mut game.player);
		}
		restored = false;
		match play_level(&mut game) {
			PlayResult::TrapDoorDown | PlayResult::StairsDown | PlayResult::StairsUp => {
				// Ignore and stay in loop
			}
			PlayResult::ExitWon | PlayResult::ExitQuit | PlayResult::ExitSaved => {
				break;
			}
			PlayResult::CleanedUp(exit) => {
				exit_line = Some(exit);
				break;
			}
		}
	}
	drop(console);
	if let Some(exit) = exit_line {
		println!("\n{}", exit);
	}
}
