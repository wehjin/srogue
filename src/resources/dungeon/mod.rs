pub mod stats;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DungeonVisor {
	Map,
	Help,
	Inventory,
}