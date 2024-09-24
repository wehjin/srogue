use crate::random::rand_percent;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::setup::npc;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{setup, DungeonLevel};
use crate::room::RoomBounds;
use rand::Rng;
use std::collections::HashSet;

pub mod depth;

pub fn roll_party(level: &mut DungeonLevel, stats: &mut DungeonStats, rng: &mut impl Rng) {
	// Venue
	let party_room = setup::roll_vault_or_maze(level, rng);
	level.party_room = Some(party_room);
	// Favors
	let favors = if rand_percent(99) {
		let favors = roll_party_favors(party_room, level, stats, rng);
		Some(favors)
	} else {
		None
	};
	// Guests
	if rand_percent(99) {
		let count = favors.unwrap_or(11);
		roll_party_guests(count, party_room, level, rng);
	}
}

fn roll_party_guests(favors: usize, venue: RoomId, level: &mut DungeonLevel, rng: &mut impl Rng) {
	let search = level.as_room(venue).bounds.inset(1, 1);
	let spots = roll_party_spots(search, 2 * favors, level, true);
	let level_boost = level.depth % 3;
	for spot in spots {
		let mut monster = npc::roll_monster(level.depth, level_boost, rng);
		if !monster.m_flags.imitates {
			monster.m_flags.wakens = true;
		}
		level.put_monster(spot, monster);
	}
}

fn roll_party_favors(room_id: RoomId, level: &mut DungeonLevel, stats: &mut DungeonStats, rng: &mut impl Rng) -> usize {
	let search_bounds = level.as_room(room_id).bounds.inset(1, 1);
	let count = rng.gen_range(5..=10);
	let vacant_spots = roll_party_spots(search_bounds, count, level, false);
	for spot in &vacant_spots {
		let object = setup::roll_object(level.depth, stats, rng);
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