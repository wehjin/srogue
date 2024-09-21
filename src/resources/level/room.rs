use crate::resources::level::sector::Sector;
use crate::room::RoomType;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RoomId {
	Big,
	Little(Sector, RoomType),
}

impl RoomId {
	pub fn room_type(&self) -> RoomType {
		match self {
			RoomId::Big => RoomType::Room,
			RoomId::Little(_, ty) => *ty,
		}
	}
}
