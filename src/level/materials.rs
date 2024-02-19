use ncurses::chtype;
use serde::{Deserialize, Serialize};

use crate::render_system::{FLOOR_CHAR, FLOOR_WITH_HIDDEN_TRAP_CHAR, STAIRS_CHAR, TRAP_CHAR};
use crate::room::DoorDirection;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
	Visible,
	Hidden,
}

impl Visibility {
	pub fn is_hidden(&self) -> bool { self == &Self::Hidden }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellMaterial {
	None,
	HorizontalWall,
	VerticalWall,
	Door(DoorDirection, Visibility),
	Floor(Fixture),
	Tunnel(Visibility, Fixture),
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
	pub fn fixture(&self) -> Option<&Fixture> {
		match self {
			CellMaterial::Floor(fix) => Some(&fix),
			CellMaterial::Tunnel(_, fix) => Some(&fix),
			_ => None,
		}
	}
	pub fn is_any_trap(&self) -> bool {
		self.fixture().map(Fixture::is_any_trap).unwrap_or(false)
	}
	pub fn is_visible_trap(&self) -> bool {
		CellMaterial::Floor(Fixture::Trap(Visibility::Visible)) == *self
	}
	pub fn with_hidden_trap(&self) -> Self {
		match self {
			CellMaterial::Floor(_) => CellMaterial::Floor(Fixture::Trap(Visibility::Hidden)),
			CellMaterial::Tunnel(viz, _) => CellMaterial::Tunnel(*viz, Fixture::Trap(Visibility::Hidden)),
			_ => self.clone(),
		}
	}
	pub fn is_stairs(&self) -> bool {
		self.fixture().map(Fixture::is_stairs).unwrap_or(false)
	}
	pub fn with_stairs(&self) -> Self {
		match self {
			CellMaterial::Floor(_) => CellMaterial::Floor(Fixture::Stairs),
			CellMaterial::Tunnel(viz, _) => CellMaterial::Tunnel(*viz, Fixture::Stairs),
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
			CellMaterial::Tunnel(_, fix) => CellMaterial::Tunnel(Visibility::Visible, fix.to_visible()),
		}
	}
	pub fn to_hidden(&self) -> Self {
		match self {
			CellMaterial::None => CellMaterial::None,
			CellMaterial::HorizontalWall => CellMaterial::HorizontalWall,
			CellMaterial::VerticalWall => CellMaterial::VerticalWall,
			CellMaterial::Door(dir, _) => CellMaterial::Door(*dir, Visibility::Hidden),
			CellMaterial::Floor(fix) => CellMaterial::Floor(fix.to_hidden()),
			CellMaterial::Tunnel(_, fix) => CellMaterial::Tunnel(Visibility::Hidden, fix.to_hidden()),
		}
	}
}

impl Default for CellMaterial {
	fn default() -> Self { Self::None }
}

impl CellMaterial {
	pub fn to_chtype(&self) -> chtype {
		chtype::from(self.to_char())
	}
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
					Fixture::None => FLOOR_CHAR,
					Fixture::Trap(viz) => match viz {
						Visibility::Visible => TRAP_CHAR,
						Visibility::Hidden => FLOOR_WITH_HIDDEN_TRAP_CHAR,
					}
					Fixture::Stairs => STAIRS_CHAR,
				}
			}
			CellMaterial::Tunnel(viz, fix) => {
				match viz {
					Visibility::Visible => {
						match fix {
							Fixture::Stairs => '%',
							Fixture::None | Fixture::Trap(_) => '#'
						}
					}
					Visibility::Hidden => ' ',
				}
			}
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub enum Fixture {
	#[default]
	None,
	Stairs,
	Trap(Visibility),
}

impl Fixture {
	pub fn as_char(&self, ground_char: char) -> char {
		match self {
			Fixture::None => ground_char,
			Fixture::Stairs => '%',
			Fixture::Trap(Visibility::Visible) => '^',
			Fixture::Trap(Visibility::Hidden) => ',',
		}
	}
	pub fn is_any_trap(&self) -> bool {
		match self {
			Fixture::None => false,
			Fixture::Stairs => false,
			Fixture::Trap(_) => true,
		}
	}
	pub fn is_stairs(&self) -> bool {
		match self {
			Fixture::None => false,
			Fixture::Stairs => true,
			Fixture::Trap(_) => false,
		}
	}
	pub fn is_hidden(&self) -> bool {
		match self {
			Fixture::None => false,
			Fixture::Stairs => false,
			Fixture::Trap(viz) => viz.is_hidden(),
		}
	}
	pub fn to_visible(&self) -> Self {
		match self {
			Fixture::None => Fixture::None,
			Fixture::Stairs => Fixture::Stairs,
			Fixture::Trap(_) => Fixture::Trap(Visibility::Visible),
		}
	}
	pub fn to_hidden(&self) -> Self {
		match self {
			Fixture::None => Fixture::None,
			Fixture::Stairs => Fixture::Stairs,
			Fixture::Trap(_) => Fixture::Trap(Visibility::Hidden),
		}
	}
}

