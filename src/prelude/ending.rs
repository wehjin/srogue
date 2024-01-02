use crate::objects::object;

const HYPOTHERMIA: usize = 1;
const STARVATION: usize = 2;
const POISON_DART: usize = 3;
const QUIT: usize = 4;
const WIN: usize = 5;


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Ending<'a> {
	Monster(&'a object),
	Hypothermia,
	Starvation,
	PoisonDart,
	Quit,
	Win,
}

impl<'a> Ending<'a> {
	pub fn is_monster(&self) -> bool {
		if let Ending::Monster(_) = self {
			true
		} else {
			false
		}
	}
}