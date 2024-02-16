const HYPOTHERMIA: usize = 1;
const STARVATION: usize = 2;
const POISON_DART: usize = 3;
const QUIT: usize = 4;
const WIN: usize = 5;


#[derive(Clone)]
pub enum Ending {
	Monster(String),
	Hypothermia,
	Starvation,
	PoisonDart,
	Quit,
	Win,
}

impl Ending {
	pub fn is_monster(&self) -> bool {
		if let Ending::Monster(_) = self {
			true
		} else {
			false
		}
	}
	pub fn is_quit(&self) -> bool {
		if let Ending::Quit = self {
			true
		} else {
			false
		}
	}
	pub fn is_win(&self) -> bool {
		if let Ending::Win = self {
			true
		} else {
			false
		}
	}
}