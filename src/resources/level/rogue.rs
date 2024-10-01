use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::wake::{wake_room, WakeType};
use crate::resources::level::DungeonLevel;
use crate::resources::rogue::spot::RogueSpot;
use rand::Rng;
use rand_chacha::ChaCha8Rng;

pub fn put_player(mut level: DungeonLevel, mut rng: ChaCha8Rng) -> (DungeonLevel, ChaCha8Rng) {
	let rogue_spot = roll_rogue_spot(&level, level.party_room, &mut rng);
	level.put_rogue(rogue_spot);
	match rogue_spot {
		RogueSpot::None => {}
		RogueSpot::Vault(_, room) => level.light_room(room),
		RogueSpot::Passage(spot) => level.light_tunnel_spot(spot),
	};
	let (level, rng) = wake_room(WakeType::DropIn(rogue_spot.try_room(&level)), level, rng);
	(level, rng)
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