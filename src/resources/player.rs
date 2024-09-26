use crate::motion::MoveDirection;

pub enum PlayerInput {
	Close,
	Help,
	Menu,
	Arrow(MoveDirection),
}

pub enum InputMode {
	Any,
	Alert,
}

