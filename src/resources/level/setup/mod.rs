use crate::objects::Object;
use crate::odds::GOLD_PERCENT;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::design::roll_design;
use crate::resources::level::map::LevelMap;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room::LevelRoom;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::resources::level::setup::random_what::RandomWhat;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{DungeonLevel, LevelType};
use crate::room::{RoomBounds, RoomType};
use rand::prelude::SliceRandom;
use std::collections::HashSet;

pub fn roll_filled_level(depth: usize, is_max: bool, party_type: LevelType, stats: &mut DungeonStats) -> DungeonLevel {
	let mut level = roll_level_with_rooms(depth, is_max, party_type);
	roll_objects(&mut level, stats);
	roll_stairs(&mut level);
	level
}

fn roll_level_with_rooms(depth: usize, is_max: bool, party_type: LevelType) -> DungeonLevel {
	if roll_build_big_room(party_type) {
		let bounds = RoomBounds {
			top: get_rand(ROW0, ROW0 + 1),
			bottom: get_rand(ROW3 - 6, ROW3 - 1),
			left: get_rand(COL0, 10),
			right: get_rand(COL3 - 11, COL3 - 1),
		};
		let room = LevelRoom { ty: RoomType::Room, bounds, ..LevelRoom::default() };
		DungeonLevel {
			depth,
			is_max,
			ty: party_type,
			rooms: vec![(RoomId::Big, room)].into_iter().collect(),
			map: LevelMap::new().put_walls_and_floor(bounds),
			rogue_spot: LevelSpot::from_i64(0, 0),
		}
	} else {
		let design = roll_design();
		let level = PlainLevel::new(depth)
			.add_rooms(design)
			.add_mazes()
			.connect_spaces()
			.add_deadends()
			;
		let rooms = ALL_SECTORS
			.into_iter()
			.map(|sector| {
				let space = level.space(sector);
				let room_id = RoomId::Little(sector, space.ty);
				(room_id, *space)
			})
			.collect();
		let map = level.into_map();
		DungeonLevel { depth, is_max, ty: party_type, rooms, map, rogue_spot: LevelSpot::from_i64(0, 0) }
	}
}

pub fn roll_stairs(level: &mut DungeonLevel) {
	let spot = level.roll_required_vacant_spot();
	level.put_stairs(spot);
}

pub fn roll_objects(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	if level.is_max {
		if level.ty.is_party() {
			roll_party(level, stats);
		}
		for _ in 0..roll_object_count() {
			let spot = level.roll_required_vacant_spot();
			let object = roll_object(level.depth, stats);
			level.put_object(spot, object);
		}
		roll_gold(level);
	}
}

fn roll_party(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	let room_id = roll_vault_or_maze(level);

	// Favors
	let _objects_added = if rand_percent(99) {
		let added = roll_party_objects(room_id, level, stats);
		Some(added)
	} else {
		None
	};
	// TODO Guests
	// if rand_percent(99) {
	// 	let count = objects_added.unwrap_or(11);
	// 	roll_party_monsters(count, room_id, level, stats);
	// }
}

fn roll_party_objects(room_id: RoomId, level: &mut DungeonLevel, stats: &mut DungeonStats) -> usize {
	let room = level.as_room(room_id).expect("invalid room id");
	let search_bounds = room.bounds.inset(1, 1);
	let count = get_rand(5, 10) as usize;
	let vacant_spots = roll_optional_vacant_spots(search_bounds, count, level);
	for spot in &vacant_spots {
		let object = roll_object(level.depth, stats);
		level.put_object(*spot, object);
	}
	vacant_spots.len()
}

fn roll_optional_vacant_spots(search_bounds: RoomBounds, count: usize, level: &mut DungeonLevel) -> HashSet<LevelSpot> {
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

fn roll_build_big_room(level_type: LevelType) -> bool {
	match level_type {
		LevelType::PartyBig => true,
		LevelType::PartyRollBig => rand_percent(1),
		LevelType::Plain => false
	}
}