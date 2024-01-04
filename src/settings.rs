use std::env;
use std::sync::RwLock;

pub static SETTINGS: RwLock<Settings> = RwLock::new(Settings {
	score_only: false,
	rest_file: None,
	fruit: String::new(),
	save_file: None,
	jump: false,
	nick_name: None,
	ask_quit: false,
	show_skull: false,
	login_name: String::new(),
});

fn set(settings: Settings) {
	*SETTINGS.write().unwrap() = settings;
}

pub fn set_login_name(login_name: &str) {
	let mut settings = (&*SETTINGS.read().unwrap()).clone();
	settings.login_name = login_name.to_string();
	set(settings);
}

pub fn login_name() -> String {
	let s = &*SETTINGS.read().unwrap();
	s.login_name.to_string()
}

pub fn score_only() -> bool { (&*SETTINGS.read().unwrap()).score_only }

pub fn set_score_only(score_only: bool) {
	let mut settings = (&*SETTINGS.read().unwrap()).clone();
	settings.score_only = score_only;
	set(settings);
}

pub fn rest_file() -> Option<String> {
	let s = &*SETTINGS.read().unwrap();
	s.rest_file.clone()
}

pub fn nick_name() -> Option<String> {
	let s = &*SETTINGS.read().unwrap();
	s.nick_name.clone()
}

pub fn save_file() -> Option<String> {
	let s = &*SETTINGS.read().unwrap();
	s.save_file.clone()
}

pub fn fruit() -> String {
	let s = &*SETTINGS.read().unwrap();
	s.fruit.to_string()
}

pub fn jump() -> bool { (&*SETTINGS.read().unwrap()).jump }

pub fn show_skull() -> bool { (&*SETTINGS.read().unwrap()).show_skull }

pub fn load() {
	let mut settings = Settings {
		score_only: false,
		rest_file: None,
		fruit: "slime-mold ".to_string(),
		save_file: None,
		jump: true,
		nick_name: None,
		ask_quit: true,
		show_skull: true,
		login_name: "PLACEHOLDER".to_string(),
	};
	settings.do_args();
	settings.do_opts();
	set(settings);
}

#[derive(Clone)]
pub struct Settings {
	pub score_only: bool,
	pub rest_file: Option<String>,
	pub fruit: String,
	pub save_file: Option<String>,
	pub jump: bool,
	pub nick_name: Option<String>,
	pub ask_quit: bool,
	pub show_skull: bool,
	pub login_name: String,
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


