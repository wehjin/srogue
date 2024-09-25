use crate::resources::game::RogueSpot;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::DungeonLevel;
use rand::Rng;

pub fn roll_rogue(level: &mut DungeonLevel, rng: &mut impl Rng) {
	let rogue_spot = roll_rogue_spot(level, level.party_room, rng);
	level.put_rogue(rogue_spot);
	match rogue_spot {
		RogueSpot::None => {}
		RogueSpot::Vault(_, room) => level.light_room(room),
		RogueSpot::Passage(spot) => level.light_tunnel_spot(spot),
	}

	// TODO Wake the room
	// TODO Write new-level message.
	// TODO Update screen.
}

fn roll_rogue_spot(level: &DungeonLevel, avoid_room: Option<RoomId>, rng: &mut impl Rng) -> RogueSpot {
	let spot: LevelSpot;
	let room: RoomId;
	let mut attempt = 0usize;
	'search: loop {
		let candidate = level.roll_vacant_spot(true, false, true, rng);
		let candidate_room = level.room_at_spot(candidate).expect("invalid spot");
		if avoid_room.is_none() || Some(candidate_room) != avoid_room || attempt > 2 {
			room = candidate_room;
			spot = candidate;
			break 'search;
		}
		attempt += 1;
	}
	if room.is_vault() {
		RogueSpot::Vault(spot, room)
	} else {
		RogueSpot::Passage(spot)
	}
}