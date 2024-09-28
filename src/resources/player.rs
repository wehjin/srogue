use crate::motion::MoveDirection;

pub enum PlayerInput {
	Close,
	Help,
	Menu,
	Arrow(MoveDirection),
	Space,
}

pub enum InputMode {
	Any,
	Alert,
}

