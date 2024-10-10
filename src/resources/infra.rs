use crate::init::GameState;
use crate::level::Level;
use crate::player::RoomMark;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::ALL_SECTORS;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::state::RunState;
use crate::room::RoomBounds;

pub trait Infra {
	fn get_room_at(&self, row: i64, col: i64) -> Option<RoomId>;
	fn room_bounds(&self, room_id: RoomId) -> RoomBounds;
	fn see_invisible(&self) -> bool;
	fn set_see_invisible(&mut self, value: bool);
	fn detect_monster(&self) -> bool;
}

impl Infra for Level {
	fn get_room_at(&self, row: i64, col: i64) -> Option<RoomId> {
		let room = self.room(row, col);
		let room_id = match room {
			RoomMark::None => None,
			RoomMark::Passage => None,
			RoomMark::Cavern(i) => {
				let sector = ALL_SECTORS[i];
				let room_type = self.rooms[i].room_type;
				Some(RoomId::Little(sector, room_type))
			}
		};
		room_id
	}

	fn room_bounds(&self, room_id: RoomId) -> RoomBounds {
		let sector = room_id.as_sector().unwrap();
		self.rooms[*sector as usize].to_wall_bounds()
	}

	fn see_invisible(&self) -> bool {
		self.see_invisible
	}

	fn set_see_invisible(&mut self, value: bool) { self.see_invisible = value }

	fn detect_monster(&self) -> bool {
		self.detect_monster
	}
}

impl Infra for GameState {
	fn get_room_at(&self, row: i64, col: i64) -> Option<RoomId> { self.level.get_room_at(row, col) }

	fn room_bounds(&self, room_id: RoomId) -> RoomBounds {
		self.level.room_bounds(room_id)
	}

	fn see_invisible(&self) -> bool {
		self.level.see_invisible()
	}

	fn set_see_invisible(&mut self, value: bool) { self.level.set_see_invisible(value) }

	fn detect_monster(&self) -> bool {
		self.level.detect_monster()
	}
}

impl Infra for RunState {
	fn get_room_at(&self, row: i64, col: i64) -> Option<RoomId> {
		self.level.room_at_spot(LevelSpot::from_i64(row, col))
	}

	fn room_bounds(&self, room_id: RoomId) -> RoomBounds {
		self.level.as_room(room_id).bounds.clone()
	}

	fn see_invisible(&self) -> bool {
		self.level.see_invisible
	}

	fn set_see_invisible(&mut self, value: bool) { self.level.see_invisible = value }

	fn detect_monster(&self) -> bool {
		self.level.detect_monster
	}
}