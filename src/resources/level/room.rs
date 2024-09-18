use crate::resources::level::sector::Sector;

pub const LITTLE_ROOMS: [RoomId; 9] = [
	RoomId::Little(Sector::TopLeft), RoomId::Little(Sector::TopCenter), RoomId::Little(Sector::TopRight),
	RoomId::Little(Sector::MiddleLeft), RoomId::Little(Sector::MiddleCenter), RoomId::Little(Sector::MiddleRight),
	RoomId::Little(Sector::BottomLeft), RoomId::Little(Sector::BottomCenter), RoomId::Little(Sector::BottomRight),
];
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RoomId {
	Big,
	Little(Sector),
}
