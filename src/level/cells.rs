use serde::{Deserialize, Serialize};

use crate::level::materials::CellMaterial;
use crate::prelude::object_what::ObjectWhat;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DungeonCell {
	material: CellMaterial,
	object: ObjectWhat,
	monster: bool,
}

impl DungeonCell {
	pub fn is_any_wall(&self) -> bool { self.material.is_any_wall() }
	pub fn is_horizontal_wall(&self) -> bool { self.material.is_horizontal_wall() }
	pub fn is_vertical_wall(&self) -> bool { self.material.is_vertical_wall() }
	pub fn is_stairs(&self) -> bool { self.material.is_stairs() }

	pub fn add_stairs(&mut self) {
		self.material = self.material.with_stairs();
	}
	pub fn is_any_trap(&self) -> bool { self.material.is_any_trap() }
	pub fn add_hidden_trap(&mut self) {
		self.material = self.material.with_hidden_trap();
	}
	pub fn is_any_floor(&self) -> bool { self.material.is_any_floor() }
	pub fn is_any_door(&self) -> bool { self.material.is_any_door() }
	pub fn is_any_tunnel(&self) -> bool { self.material.is_any_tunnel() }
	pub fn is_not_tunnel(&self) -> bool { self.material.is_not_tunnel() }
	pub fn is_any_hidden(&self) -> bool {
		match self.material() {
			CellMaterial::None | CellMaterial::HorizontalWall | CellMaterial::VerticalWall => false,
			CellMaterial::Door(_, viz) => viz.is_hidden(),
			CellMaterial::Floor(fix) => fix.is_hidden(),
			CellMaterial::Tunnel(viz, _) => viz.is_hidden(),
		}
	}
	pub fn set_visible(&mut self) {
		self.material = self.material.to_visible();
	}
	pub fn set_hidden(&mut self) {
		self.material = self.material.to_hidden();
	}
}

impl DungeonCell {
	const NOTHING: Self = Self {
		material: CellMaterial::None,
		object: ObjectWhat::None,
		monster: false,
	};
	pub fn is_nothing(&self) -> bool { self == &Self::NOTHING }
	pub fn reset_to_nothing(&mut self) {
		*self = Self::NOTHING;
	}
}

impl DungeonCell {
	pub fn material(&self) -> &CellMaterial { &self.material }
	pub fn set_material_remove_others(&mut self, mat: CellMaterial) {
		self.material = mat;
		self.object = ObjectWhat::None;
		self.monster = false;
	}
	pub fn is_material_no_others(&self, mat: CellMaterial) -> bool {
		self.material == mat
			&& self.object == ObjectWhat::None
			&& self.monster == false
	}
}

impl DungeonCell {
	pub fn has_monster(&self) -> bool { self.monster }
	pub fn set_monster(&mut self, value: bool) {
		self.monster = value;
	}
}

impl DungeonCell {
	pub fn object_what(&self) -> ObjectWhat { self.object }
	pub fn has_object(&self) -> bool { self.object.is_object() }
	pub fn clear_object(&mut self) { self.object = ObjectWhat::None }
	pub fn set_object(&mut self, value: ObjectWhat) {
		self.object = value
	}
}
