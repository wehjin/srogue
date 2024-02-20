use std::collections::HashMap;

use crate::actions::PlayerAction;

#[derive(Default)]
pub struct PlayerActionSet {
	actions: HashMap<char, Box<dyn PlayerAction + Send + Sync>>,
}

impl PlayerActionSet {
	pub fn add(&mut self, key: char, action: Box::<dyn PlayerAction + Send + Sync>) {
		assert!(!self.actions.contains_key(&key));
		self.actions.insert(key, action);
	}
	pub fn get(&self, key: char) -> Option<&Box<dyn PlayerAction + Send + Sync>> {
		self.actions.get(&key)
	}
}

impl PlayerActionSet {
	pub fn new(actions: Vec<(char, Box<dyn PlayerAction + Send + Sync>)>) -> Self {
		let mut set = Self::default();
		for (key, action) in actions {
			set.add(key, action);
		}
		set
	}
}

