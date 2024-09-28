use serde::{Deserialize, Serialize};
use std::env;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SettingsError { LoginName }

pub fn load() -> Result<Settings, SettingsError> {
	let mut settings = Settings::default();
	settings.do_args();
	settings.do_opts();
	Ok(settings)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Settings {
	pub login_name: String,
	pub fruit: String,
	pub score_only: bool,
	pub rest_file: Option<String>,
	pub save_file: Option<String>,
	pub jump: bool,
	pub nick_name: Option<String>,
	pub ask_quit: bool,
	pub show_skull: bool,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			login_name: whoami::username(),
			fruit: "slime-mold ".to_string(),
			score_only: false,
			rest_file: None,
			save_file: None,
			jump: true,
			nick_name: None,
			ask_quit: true,
			show_skull: true,
		}
	}
}

impl Settings {
	pub fn player_name(&self) -> &String {
		if let Some(nick_name) = &self.nick_name {
			nick_name
		} else {
			&self.login_name
		}
	}
}

impl Settings {
	fn do_args(&mut self) {
		let args = env::args().collect::<Vec<_>>();
		for s in &args[1..] {
			if s.starts_with('-') {
				if s[1..].find('s').is_some() {
					self.score_only = true;
				}
			} else {
				self.rest_file = Some(s.clone());
			}
		}
	}

	fn do_opts(&mut self) {
		const DIVIDER: char = ',';
		if let Ok(opts) = env::var("ROGUEOPTS") {
			const FRUIT_EQ: &'static str = "fruit=";
			const FILE_EQ: &'static str = "file=";
			const NAME: &'static str = "name=";

			for opt in opts.split(DIVIDER) {
				let opt = opt.trim();
				if opt.starts_with(FRUIT_EQ) {
					self.fruit = format!("{} ", opt[FRUIT_EQ.len()..].to_string());
				} else if opt.starts_with(FILE_EQ) {
					self.save_file = Some(opt[FILE_EQ.len()..].to_string());
				} else if opt == "nojump" {
					self.jump = false;
				} else if opt.starts_with(NAME) {
					self.nick_name = Some(opt[NAME.len()..].to_string())
				} else if opt == "noaskquit" {
					self.ask_quit = false;
				} else if opt == "noskull" || opt == "notomb" {
					self.show_skull = false;
				}
			}
		}
	}
}


