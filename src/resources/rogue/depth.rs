use crate::player::LAST_DUNGEON;
#[derive(Debug, Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct RogueDepth {
	current: usize,
	max: usize,
}
impl RogueDepth {
	pub fn new(start: usize) -> Self { Self { current: start, max: start } }
	pub fn usize(&self) -> usize { self.current }
	pub fn max(&self) -> usize { self.max }
	pub fn is_max(&self) -> bool { self.current == self.max }
	pub fn descend(self) -> Self {
		if self.current < LAST_DUNGEON as usize {
			let current = self.current + 1;
			let max = self.max.max(current);
			return Self { current, max };
		}
		self
	}
}
