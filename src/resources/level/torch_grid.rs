use crate::resources::level::grid::LevelGrid;
use crate::resources::level::size::LevelSpot;

#[derive(Debug, Clone)]
pub struct TorchGrid(LevelGrid<bool>);

impl TorchGrid {
	pub fn new() -> Self {
		Self(LevelGrid::new())
	}
	pub fn lit_at(&self, spot: LevelSpot) -> bool {
		self.0.value_at(spot)
	}
	pub fn light(&mut self, spot: LevelSpot) {
		self.0.put_value(spot, true)
	}
}

