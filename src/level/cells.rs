use serde::{Deserialize, Serialize};

use crate::prelude::object_what::ObjectWhat;
use crate::room::DoorDirection;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellMaterial {
	None,
	HorizontalWall,
	VerticalWall,
	Door(DoorDirection),
	Floor,
	Tunnel,
}

impl CellMaterial {
	pub fn is_door(&self) -> bool {
		match self {
			Self::Door(_) => true,
			_ => false,
		}
	}
	pub fn is_wall(&self) -> bool {
		match self {
			Self::HorizontalWall | Self::VerticalWall => true,
			_ => false,
		}
	}
}

impl Default for CellMaterial {
	fn default() -> Self { Self::None }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellFixture {
	None,
	Trap,
	Stairs,
}

impl Default for CellFixture {
	fn default() -> Self { Self::None }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DungeonCell {
	material: CellMaterial,
	fixture: CellFixture,
	hidden: bool,
	object: ObjectWhat,
	monster: bool,
}

impl DungeonCell {
	const NOTHING: Self = Self {
		material: CellMaterial::None,
		fixture: CellFixture::None,
		hidden: false,
		object: ObjectWhat::None,
		monster: false,
	};
	pub fn reset_to_nothing(&mut self) {
		*self = Self::NOTHING;
	}
	pub fn set_material_and_clear_others(&mut self, mat: CellMaterial) {
		self.material = mat;
		self.fixture = CellFixture::None;
		self.object = ObjectWhat::None;
		self.monster = false;
		self.hidden = false;
	}
	pub fn set_monster(&mut self, value: bool) {
		self.monster = value;
	}
	pub fn set_object(&mut self, value: ObjectWhat) {
		self.object = value
	}
	pub fn clear_object(&mut self) {
		self.object = ObjectWhat::None
	}
	pub fn set_hidden(&mut self, value: bool) {
		self.hidden = value;
	}
	pub fn set_fixture(&mut self, value: CellFixture) {
		self.fixture = value;
	}
	pub fn object_what(&self) -> ObjectWhat { self.object }
	pub fn is_nothing(&self) -> bool { self == &Self::NOTHING }
	pub fn material(&self) -> &CellMaterial { &self.material }
	pub fn is_wall(&self) -> bool { self.material.is_wall() }
	pub fn is_door(&self) -> bool { self.material.is_door() }
	pub fn is_floor(&self) -> bool { self.material == CellMaterial::Floor }
	pub fn is_tunnel(&self) -> bool { self.material == CellMaterial::Tunnel }
	pub fn is_stairs(&self) -> bool { self.fixture == CellFixture::Stairs }
	pub fn is_trap(&self) -> bool { self.fixture == CellFixture::Trap }
	pub fn has_monster(&self) -> bool { self.monster }
	pub fn has_object(&self) -> bool { self.object.is_object() }
	pub fn is_hidden(&self) -> bool { self.hidden }
	pub fn is_material(&self, mat: CellMaterial) -> bool { self.material == mat }
	pub fn is_not_material(&self, mat: CellMaterial) -> bool {
		!self.is_material(mat)
	}
	pub fn is_material_only(&self, mat: CellMaterial) -> bool {
		self.material == mat
			&& self.fixture == CellFixture::None
			&& self.object == ObjectWhat::None
			&& self.monster == false
			&& self.hidden == false
	}
}
