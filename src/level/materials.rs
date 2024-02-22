use serde::{Deserialize, Serialize};

use crate::render_system::{FLOOR_CHAR, FLOOR_WITH_HIDDEN_TRAP_CHAR, STAIRS_CHAR, TRAP_CHAR};
use crate::room::DoorDirection;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
	Visible,
	Hidden,
}

impl Visibility {
	pub fn is_hidden(&self) -> bool { *self == Self::Hidden }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellMaterial {
	None,
	HorizontalWall,
	VerticalWall,
	Door(DoorDirection, Visibility),
	Floor(FloorFixture),
	Tunnel(Visibility, TunnelFixture),
}

impl CellMaterial {
	pub fn is_any_wall(&self) -> bool {
		match self {
			Self::HorizontalWall => true,
			Self::VerticalWall => true,
			_ => false,
		}
	}
	pub fn is_horizontal_wall(&self) -> bool {
		match self {
			Self::HorizontalWall => true,
			_ => false,
		}
	}
	pub fn is_vertical_wall(&self) -> bool {
		match self {
			Self::VerticalWall => true,
			_ => false,
		}
	}
	pub fn is_any_floor(&self) -> bool {
		match self {
			Self::Floor(_) => true,
			_ => false,
		}
	}
	pub fn is_any_door(&self) -> bool {
		match self {
			Self::Door(_, _) => true,
			_ => false,
		}
	}
	pub fn is_any_trap(&self) -> bool {
		match self {
			CellMaterial::Floor(fix) => fix.is_any_trap(),
			_ => false
		}
	}
	pub fn is_visible_trap(&self) -> bool {
		CellMaterial::Floor(FloorFixture::Trap(Visibility::Visible)) == *self
	}
	pub fn with_hidden_trap(&self) -> Self {
		match self {
			CellMaterial::Floor(_) => CellMaterial::Floor(FloorFixture::Trap(Visibility::Hidden)),
			_ => self.clone(),
		}
	}
	pub fn is_stairs(&self) -> bool {
		match self {
			CellMaterial::Floor(fix) => fix.is_stairs(),
			CellMaterial::Tunnel(_, fix) => fix.is_stairs(),
			_ => false,
		}
	}
	pub fn with_stairs(&self) -> Self {
		match self {
			CellMaterial::Floor(_) => CellMaterial::Floor(FloorFixture::Stairs),
			CellMaterial::Tunnel(viz, _) => CellMaterial::Tunnel(*viz, TunnelFixture::Stairs),
			_ => self.clone(),
		}
	}
	pub fn is_any_tunnel(&self) -> bool {
		match self {
			Self::Tunnel(_, _) => true,
			_ => false,
		}
	}
	pub fn is_not_tunnel(&self) -> bool {
		!self.is_any_tunnel()
	}
	pub fn to_visible(&self) -> Self {
		match self {
			CellMaterial::None => CellMaterial::None,
			CellMaterial::HorizontalWall => CellMaterial::HorizontalWall,
			CellMaterial::VerticalWall => CellMaterial::VerticalWall,
			CellMaterial::Door(dir, _) => CellMaterial::Door(*dir, Visibility::Visible),
			CellMaterial::Floor(fix) => CellMaterial::Floor(fix.to_visible()),
			CellMaterial::Tunnel(_, fix) => CellMaterial::Tunnel(Visibility::Visible, *fix),
		}
	}
	pub fn to_hidden(&self) -> Self {
		match self {
			CellMaterial::None => CellMaterial::None,
			CellMaterial::HorizontalWall => CellMaterial::HorizontalWall,
			CellMaterial::VerticalWall => CellMaterial::VerticalWall,
			CellMaterial::Door(dir, _) => CellMaterial::Door(*dir, Visibility::Hidden),
			CellMaterial::Floor(fix) => CellMaterial::Floor(fix.to_hidden()),
			CellMaterial::Tunnel(_, fix) => CellMaterial::Tunnel(Visibility::Hidden, *fix),
		}
	}
}

impl Default for CellMaterial {
	fn default() -> Self { Self::None }
}

impl CellMaterial {
	pub fn to_char(&self) -> char {
		match self {
			CellMaterial::None => ' ',
			CellMaterial::HorizontalWall => '-',
			CellMaterial::VerticalWall => '|',
			CellMaterial::Door(dir, vis) => {
				match vis {
					Visibility::Visible => '+',
					Visibility::Hidden => match dir.is_up_or_down() {
						true => '-',
						false => '|',
					},
				}
			}
			CellMaterial::Floor(fix) => {
				match fix {
					FloorFixture::None => FLOOR_CHAR,
					FloorFixture::Trap(viz) => match viz {
						Visibility::Visible => TRAP_CHAR,
						Visibility::Hidden => FLOOR_WITH_HIDDEN_TRAP_CHAR,
					}
					FloorFixture::Stairs => STAIRS_CHAR,
				}
			}
			CellMaterial::Tunnel(viz, fix) => {
				match viz {
					Visibility::Visible => match fix {
						TunnelFixture::Stairs => '%',
						TunnelFixture::None => '#',
					},
					Visibility::Hidden => ' ',
				}
			}
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub enum TunnelFixture {
	#[default]
	None,
	Stairs,
}

impl TunnelFixture {
	pub fn is_stairs(&self) -> bool {
		*self == TunnelFixture::Stairs
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub enum FloorFixture {
	#[default]
	None,
	Stairs,
	Trap(Visibility),
}


impl FloorFixture {
	pub fn as_char(&self, ground_char: char) -> char {
		match self {
			FloorFixture::None => ground_char,
			FloorFixture::Stairs => '%',
			FloorFixture::Trap(Visibility::Visible) => '^',
			FloorFixture::Trap(Visibility::Hidden) => ',',
		}
	}
	pub fn is_any_trap(&self) -> bool {
		match self {
			FloorFixture::None => false,
			FloorFixture::Stairs => false,
			FloorFixture::Trap(_) => true,
		}
	}
	pub fn is_stairs(&self) -> bool {
		match self {
			FloorFixture::None => false,
			FloorFixture::Stairs => true,
			FloorFixture::Trap(_) => false,
		}
	}
	pub fn is_hidden(&self) -> bool {
		match self {
			FloorFixture::None => false,
			FloorFixture::Stairs => false,
			FloorFixture::Trap(viz) => viz.is_hidden(),
		}
	}
	pub fn to_visible(&self) -> Self {
		match self {
			FloorFixture::None => FloorFixture::None,
			FloorFixture::Stairs => FloorFixture::Stairs,
			FloorFixture::Trap(_) => FloorFixture::Trap(Visibility::Visible),
		}
	}
	pub fn to_hidden(&self) -> Self {
		match self {
			FloorFixture::None => FloorFixture::None,
			FloorFixture::Stairs => FloorFixture::Stairs,
			FloorFixture::Trap(_) => FloorFixture::Trap(Visibility::Hidden),
		}
	}
}

