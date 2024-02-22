use crate::init::GameState;
use crate::systems::play_level::PlayResult;

pub trait PlayerAction {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult>;
}

pub mod action_set;
pub mod ground;
pub mod eat;
pub mod fight;
pub mod instruct;
pub mod inventory;
pub mod motion;
pub mod put_on_ring;
pub mod quaff;
pub mod read_scroll;
pub mod remove_ring;
pub mod rest;
pub mod screen;
pub mod search;
pub mod take_off;
pub mod wear;
pub mod wield;
pub mod wizard;