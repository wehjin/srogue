use crate::random::{get_rand, rand_percent};
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::setup::npc;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{setup, DungeonLevel};
use crate::room::RoomBounds;
use std::collections::HashSet;

pub mod depth;

pub fn roll_party(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	// Venue
	let party_room = setup::roll_vault_or_maze(level);
	level.party_room = Some(party_room);
	// Favors
	let favors = if rand_percent(99) {
		let favors = roll_party_favors(party_room, level, stats);
		Some(favors)
	} else {
		None
	};
	// Guests
	if rand_percent(99) {
		let count = favors.unwrap_or(11);
		roll_party_guests(count, party_room, level);
	}
}

fn roll_party_guests(favors: usize, venue: RoomId, level: &mut DungeonLevel) {
	let search = level.room_at(venue).expect("invalid venue").bounds.inset(1, 1);
	let spots = roll_party_spots(search, 2 * favors, level, true);
	let level_boost = level.depth % 3;
	for spot in spots {
		let mut monster = npc::roll_monster(level.depth, level_boost);
		if !monster.m_flags.imitates {
			monster.m_flags.wakens = true;
		}
		level.put_monster(spot, monster);
	}
}

fn roll_party_favors(room_id: RoomId, level: &mut DungeonLevel, stats: &mut DungeonStats) -> usize {
	let room = level.room_at(room_id).expect("invalid room id");
	let search_bounds = room.bounds.inset(1, 1);
	let count = get_rand(5, 10) as usize;
	let vacant_spots = roll_party_spots(search_bounds, count, level, false);
	for spot in &vacant_spots {
		let object = setup::roll_object(level.depth, stats);
		level.put_object(*spot, object);
	}
	vacant_spots.len()
}

fn roll_party_spots(search_bounds: RoomBounds, count: usize, level: &DungeonLevel, allow_objects: bool) -> HashSet<LevelSpot> {
	let mut spots = HashSet::new();
	for _ in 0..count.min(search_bounds.area() as usize) {
		'search: for _ in 0..250 {
			let spot = search_bounds.roll_spot();
			if level.spot_is_vacant(spot, allow_objects, false) && !spots.contains(&spot) {
				spots.insert(spot);
				break 'search;
			}
		}
	}
	spots
}