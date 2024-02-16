use std::collections::HashMap;

use crate::init::GameState;

pub mod put_on_ring;
pub mod remove_ring;
pub mod take_off;
pub mod wear;
pub mod wield;

pub trait PlayerAction {
	fn commit(&self, game: &mut GameState);
}

#[derive(Default)]
pub struct PlayerActionSet {
	actions: HashMap<char, Box<dyn PlayerAction>>,
}

impl PlayerActionSet {
	pub fn add(&mut self, key: char, action: Box::<dyn PlayerAction>) {
		assert!(!self.actions.contains_key(&key));
		self.actions.insert(key, action);
	}
	pub fn get(&self, key: char) -> Option<&Box<dyn PlayerAction>> {
		self.actions.get(&key)
	}
}

impl PlayerActionSet {
	pub fn new(actions: Vec<(char, Box<dyn PlayerAction>)>) -> Self {
		let mut set = Self::default();
		for (key, action) in actions {
			set.add(key, action);
		}
		set
	}
}


