use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::DungeonLevel;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub enum RogueSpot {
	#[default]
	None,
	Vault(LevelSpot, RoomId),
	Passage(LevelSpot),
}

impl RogueSpot {
	pub fn from_spot(spot: LevelSpot, level: &DungeonLevel) -> Self {
		if let Some(room) = level.room_at_spot(spot) {
			if room.is_vault() {
				return RogueSpot::Vault(spot, room);
			}
		}
		RogueSpot::Passage(spot)
	}
	pub fn is_spot(&self, spot: LevelSpot) -> bool {
		self.try_spot() == Some(&spot)
	}
	pub fn as_spot(&self) -> &LevelSpot {
		self.try_spot().expect("no rogue spot")
	}
	pub fn try_spot(&self) -> Option<&LevelSpot> {
		match self {
			RogueSpot::None => None,
			RogueSpot::Vault(spot, _) => Some(spot),
			RogueSpot::Passage(spot) => Some(spot),
		}
	}
	pub fn is_vault(&self) -> bool {
		self.as_vault().is_some()
	}
	pub fn as_vault(&self) -> Option<&RoomId> {
		match self {
			RogueSpot::None => None,
			RogueSpot::Vault(_, room) => Some(room),
			RogueSpot::Passage(_) => None,
		}
	}
	pub fn is_in_room(&self, value: RoomId) -> bool {
		match self.as_vault() {
			None => false,
			Some(room_id) => *room_id == value,
		}
	}
	pub fn try_room(&self, level: &DungeonLevel) -> Option<RoomId> {
		if let Some(spot) = self.try_spot() {
			level.room_at_spot(*spot)
		} else {
			None
		}
	}
}