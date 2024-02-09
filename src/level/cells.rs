use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CellKind {
	Object,
	Monster,
	Stairs,
	HorizontalWall,
	VerticalWall,
	Door,
	Floor,
	Tunnel,
	Trap,
	Hidden,
}


#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct DungeonCell(u16);

impl DungeonCell {
	pub fn set_nothing(&mut self) {
		self.0 = 0;
	}
	pub fn set_only_kind(&mut self, kind: CellKind) {
		self.0 = Self::flag_for_kind(kind);
	}
	pub fn add_kind(&mut self, kind: CellKind) {
		self.0 |= Self::flag_for_kind(kind);
	}
	pub fn remove_kind(&mut self, kind: CellKind) {
		self.0 &= !Self::flag_for_kind(kind);
	}

	pub fn add_hidden(&mut self) {
		self.add_kind(CellKind::Hidden);
	}
	pub fn is_nothing(&self) -> bool {
		self.0 == 0
	}
	pub fn is_door(&self) -> bool { self.is_kind(CellKind::Door) }
	pub fn is_floor(&self) -> bool { self.is_kind(CellKind::Floor) }
	pub fn is_tunnel(&self) -> bool { self.is_kind(CellKind::Tunnel) }
	pub fn is_monster(&self) -> bool { self.is_kind(CellKind::Monster) }
	pub fn is_stairs(&self) -> bool { self.is_kind(CellKind::Stairs) }
	pub fn is_hidden(&self) -> bool { self.is_kind(CellKind::Hidden) }
	pub fn is_trap(&self) -> bool { self.is_kind(CellKind::Trap) }
	pub fn is_object(&self) -> bool { self.is_kind(CellKind::Object) }
	pub fn is_kind(&self, kind: CellKind) -> bool {
		(self.0 & Self::flag_for_kind(kind)) != 0
	}
	pub fn is_only_kind(&self, kind: CellKind) -> bool {
		self.0 == Self::flag_for_kind(kind)
	}
	pub fn is_not_kind(&self, kind: CellKind) -> bool {
		!self.is_kind(kind)
	}
	pub fn is_any_kind(&self, kinds: &[CellKind]) -> bool {
		kinds.iter().position(|kind| self.is_kind(*kind)).is_some()
	}
	pub fn is_other_kind(&self, kinds: &[CellKind]) -> bool {
		let mask = kinds.iter().fold(
			0u16,
			|all, next| {
				all | Self::flag_for_kind(*next)
			},
		);
		(self.0 & !mask) != 0
	}
	fn flag_for_kind(kind: CellKind) -> u16 {
		match kind {
			CellKind::Object => 0o1,
			CellKind::Monster => 0o2,
			CellKind::Stairs => 0o4,
			CellKind::HorizontalWall => 0o10,
			CellKind::VerticalWall => 0o20,
			CellKind::Door => 0o40,
			CellKind::Floor => 0o100,
			CellKind::Tunnel => 0o200,
			CellKind::Trap => 0o400,
			CellKind::Hidden => 0o1000,
		}
	}
}
