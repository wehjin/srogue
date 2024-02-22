use crate::init::GameState;

pub trait PlayerAction {
	fn update(input_key: char, game: &mut GameState);
}

pub mod action_set;
pub mod drop_item;
pub mod eat;
pub mod fight;
pub mod instruct;
pub mod inventory;
pub mod motion;
pub mod move_onto;
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