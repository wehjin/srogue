use crate::console::ConsoleError;
use crate::init::{init, InitError, InitResult};
use crate::level::{make_level, put_player_legacy};
use crate::monster::put_mons;
use crate::objects::{put_objects, put_stairs};
use crate::resources::play::{run, TextConsole};
use crate::resources::player::{InputMode, PlayerInput};
use crate::settings::SettingsError;
use crate::systems::play_level::{play_level, LevelResult};
use crate::trap::add_traps;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::prelude::Rect;
use ratatui::widgets::Paragraph;
use ratatui::DefaultTerminal;

pub mod components;
pub mod resources;
pub mod state;
pub mod queries;
mod message;
mod level;
mod monster;
mod hit;
mod init;
mod inventory;
mod machdep;
mod motion;
pub mod objects;
mod pack;
mod potions;
mod player;
mod random;
mod render_system;
mod ring;
mod room;
mod save;
mod score;
mod spec_hit;
mod systems;
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

pub fn main() -> anyhow::Result<()> {
	let mut terminal = ratatui::init();
	terminal.clear().expect("failed to clear");

	let console = TerminalConsole(terminal);
	run(console);

	ratatui::restore();
	Ok(())
}

struct TerminalConsole(DefaultTerminal);
impl TextConsole for TerminalConsole {
	fn get_input(&self, mode: InputMode) -> PlayerInput {
		loop {
			match event::read().unwrap() {
				Event::FocusGained => {}
				Event::FocusLost => {}
				Event::Key(key) => {
					let input = match mode {
						InputMode::Any => match key.code {
							KeyCode::Char(char) => match char {
								'?' => PlayerInput::Help,
								'i' => PlayerInput::Menu,
								_ => PlayerInput::Close,
							},
							_ => PlayerInput::Close,
						},
						InputMode::Alert => PlayerInput::Close,
					};
					return input;
				}
				Event::Mouse(_) => {}
				Event::Paste(_) => {}
				Event::Resize(_, _) => {}
			};
		}
	}
	fn draw(&mut self, lines: Vec<String>) {
		self.0.draw(|frame| {
			let frame_area = frame.area();
			for row in 0..lines.len() {
				let paragraph = Paragraph::new(lines[row].as_str());
				let line_area = Rect::new(frame_area.x, frame_area.y + row as u16, frame_area.width, 1);
				frame.render_widget(paragraph, line_area);
			}
		}).expect("failed to draw lines");
	}
}

pub fn main_legacy() -> anyhow::Result<()> {
	fern::Dispatch::new()
		.chain(std::io::stderr())
		.chain(fern::log_file("srogue.log")?)
		.apply()?;
	log_panics::init();

	let settings = match settings::load() {
		Ok(settings) => settings,
		Err(e) => match e {
			SettingsError::LoginName => {
				println!("Hey!  Who are you?");
				return Ok(());
			}
		},
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
				return Ok(());
			}
			InitError::BadRestore(exit) => {
				if let Some(exit) = exit {
					eprintln!("\n{}", exit);
				}
				return Ok(());
			}
		},
	};
	let (mut game, console, mut restored) = match result {
		InitResult::ScoreOnly(_player, _console, _settings) => {
			return Ok(());
		}
		InitResult::Restored(game, console) => (game, console, true),
		InitResult::Initialized(game, console) => (game, console, false),
	};
	let mut exit_line = None;
	loop {
		if !restored {
			game.clear_level();
			game.player.descend();
			make_level(&mut game);
			put_objects(&mut game);
			put_stairs(&mut game.player, &mut game.level);
			add_traps(&game.player, &mut game.level);
			put_mons(&mut game);
			put_player_legacy(game.level.party_room.into(), &mut game);
			game.stats_changed = true;
		}
		restored = false;
		match play_level(&mut game) {
			LevelResult::TrapDoorDown | LevelResult::StairsDown | LevelResult::StairsUp => {
				// Ignore and stay in loop
			}
			LevelResult::ExitWon | LevelResult::ExitQuit | LevelResult::ExitSaved => {
				break;
			}
			LevelResult::CleanedUp(exit) => {
				exit_line = Some(exit);
				break;
			}
		}
	}
	drop(console);
	if let Some(exit) = exit_line {
		println!("\n{}", exit);
	}

	Ok(())
}
