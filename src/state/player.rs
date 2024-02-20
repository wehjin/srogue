use crate::systems::play_level::PlayResult;

#[derive(Debug)]
pub enum PlayState {
	Idle,
	Counting(String),
	Busy(char, usize),
	Leaving(PlayResult),
}