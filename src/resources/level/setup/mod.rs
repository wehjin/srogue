use crate::objects::Object;
use crate::odds::GOLD_PERCENT;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::setup::random_what::RandomWhat;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::DungeonLevel;
use crate::room::RoomBounds;
use rand::prelude::SliceRandom;
use std::collections::HashSet;

pub fn roll_objects(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	if level.is_max {
		if level.ty.is_party() {
			roll_party(level, stats);
		}
		for _ in 0..roll_object_count() {
			let spot = level.roll_object_spot();
			let object = roll_object(level.depth, stats);
			level.put_object(spot, object);
		}
		roll_gold(level);
	}
}

fn roll_party(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	let room_id = roll_vault_or_maze(level);

	// Favors
	let objects_added = if rand_percent(99) {
		let added = roll_party_objects(room_id, level, stats);
		Some(added)
	} else {
		None
	};
	// Guests
	if rand_percent(99) {
		let count = objects_added.unwrap_or(11);
		roll_party_monsters(count, room_id, level, stats);
	}
}

fn roll_party_objects(room_id: RoomId, level: &mut DungeonLevel, stats: &mut DungeonStats) -> usize {
	let room = level.as_room(room_id).expect("invalid room id");
	let search_bounds = room.bounds.inset(1, 1);
	let count = get_rand(5, 10) as usize;
	let vacant_spots = roll_vacant_spots(search_bounds, count, level);
	for spot in &vacant_spots {
		let object = roll_object(level.depth, stats);
		level.put_object(*spot, object);
	}
	vacant_spots.len()
}

fn roll_vacant_spots(search_bounds: RoomBounds, count: usize, level: &mut DungeonLevel) -> HashSet<LevelSpot> {
	let mut spots = HashSet::new();
	for _ in 0..count.min(search_bounds.area() as usize) {
		'search: for _ in 0..250 {
			let spot = search_bounds.roll_spot();
			if level.spot_is_vacant(spot) && !spots.contains(&spot) {
				spots.insert(spot);
				break 'search;
			}
		}
	}
	spots
}

fn roll_party_monsters(_count: usize, _room_id: RoomId, _level: &mut DungeonLevel, _stats: &mut DungeonStats) {
	// TODO
}

fn roll_vault_or_maze(level: &DungeonLevel) -> RoomId {
	let mut rooms = level.vaults_and_mazes();
	rooms.shuffle(&mut rand::thread_rng());
	*rooms.first().expect("no vault or maze in level")
}

fn roll_gold(level: &mut DungeonLevel) {
	let rooms_and_mazes = level.vaults_and_mazes();
	for room_id in rooms_and_mazes {
		let room = level.as_room(room_id).expect("level should have room");
		if room.is_maze() || rand_percent(GOLD_PERCENT) {
			let search_bounds = room.bounds.inset(1, 1);
			for _ in 0..50 {
				let spot = search_bounds.roll_spot();
				if level.spot_is_floor_or_tunnel(spot) {
					let object = Object::roll_gold(level.depth, room.is_maze());
					level.put_object(spot, object);
					break;
				}
			}
		}
	}
}

fn roll_object(depth: usize, stats: &mut DungeonStats) -> Object {
	let what = if stats.food_drops < depth / 2 {
		stats.food_drops += 1;
		RandomWhat::Food
	} else {
		RandomWhat::roll()
	};
	match what {
		RandomWhat::Scroll => Object::roll_scroll(),
		RandomWhat::Potion => Object::roll_potion(),
		RandomWhat::Weapon => Object::roll_weapon(true),
		RandomWhat::Armor => Object::roll_armor(),
		RandomWhat::Wand => Object::roll_wand(),
		RandomWhat::Food => Object::roll_food(false),
		RandomWhat::Ring => Object::roll_ring(true),
	}
}


fn roll_object_count() -> usize {
	let mut n = if coin_toss() { get_rand(2, 4) } else { get_rand(3, 5) };
	while rand_percent(33) {
		n += 1;
	}
	n
}

pub mod random_what;
