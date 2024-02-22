use crate::systems::play_level::PlayResult;

#[derive(Debug)]
pub enum PlayState {
	Idle,
	Counting(String),
	Busy { key_code: char, completed: usize, remaining: usize },
	Leaving(PlayResult),
}