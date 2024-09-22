use crate::resources::rogue::depth::RogueDepth;

pub mod depth;


#[derive(Debug, Copy, Clone, Default)]
pub struct Rogue {
	pub has_amulet: bool,
	pub depth: RogueDepth,
}

impl Rogue {
	pub fn descend(&mut self) {
		self.depth.descend();
	}
}
