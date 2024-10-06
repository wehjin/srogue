use crate::motion::MoveDirection;

pub enum PlayerInput {
	Arrow(MoveDirection),
	Close,
	Drop,
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
