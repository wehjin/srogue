pub mod stats;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum DungeonVisor {
	Map,
	Help,
	Inventory,
}