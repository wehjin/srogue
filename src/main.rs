#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]
#![feature(extern_types)]

use crate::console::ConsoleError;
use crate::init::{init, InitError, InitResult};
use crate::level::{clear_level, make_level, put_player};
use crate::message::print_stats;
use crate::monster::{MASH, put_mons};
use crate::objects::{level_objects, put_objects, put_stairs};
use crate::play::{play_level, PlayResult};
use crate::player::RoomMark;
use crate::prelude::stat_const::STAT_ALL;
use crate::settings::SettingsError;
use crate::trap::add_traps;

mod message;
mod level;
mod monster;
mod hit;
mod init;
mod instruct;
mod inventory;
mod machdep;
mod r#move;
mod objects;
mod pack;
mod potions;
mod play;
mod player;
mod random;
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
pub mod hunger;
pub(crate) mod files;

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
	let result = match unsafe { init(settings) } {
		Ok(result) => result,
		Err(error) => match error {
			InitError::NoConsole(error) => match error {
				ConsoleError::ScreenTooSmall { min_rows, min_cols } => {
					println!("\nmust be played on {} x {} or better screen", min_rows, min_cols);
					return;
				}
			},
			InitError::BadRestore(exit) => {
				if let Some(exit) = exit {
					eprintln!("\n{}", exit);
				}
				return;
			}
		}
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
			clear_level(&mut game.player, &mut game.level);
			game.player.descend();
			unsafe { make_level(&game.player, &mut game.level) };
			unsafe { put_objects(&mut game.player, &mut game.level); }
			unsafe { put_stairs(&mut game.player, &mut game.level); }
			unsafe { add_traps(&game.player, &mut game.level); }
			unsafe { put_mons(&game.player, &mut game.level); }
			unsafe {
				let avoid_room = match game.level.party_room {
					None => RoomMark::None,
					Some(rn) => RoomMark::Area(rn),
				};
				put_player(avoid_room, &mut game.player, &mut game.level);
			}
			unsafe { print_stats(STAT_ALL, &mut game.player); }
		}
		restored = false;
		match unsafe { play_level(&mut game) } {
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
		unsafe { level_objects.clear(); }
		unsafe { MASH.clear(); }
	}
	drop(console);
	if let Some(exit) = exit_line {
		println!("\n{}", exit);
	}
}
