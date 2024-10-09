use crate::motion::MoveDirection;

pub enum PlayerInput {
	Arrow(MoveDirection),
	Close,
	Drop,
	Eat,
	Help,
	Menu,
	Select(char),
	Space,
}

pub enum InputMode {
	Any,
	Alert,
	Menu,
}
