use crate::level::constants::MAX_TRAP;
use crate::objects::Object;
use crate::odds::GOLD_PERCENT;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::AMULET_LEVEL;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::design::roll_design;
use crate::resources::level::map::LevelMap;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room::LevelRoom;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::resources::level::setup::npc::roll_monsters;
use crate::resources::level::setup::random_what::RandomWhat;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::room::{RoomBounds, RoomType};
use crate::trap::trap_kind::TrapKind;
use crate::trap::Trap;
use rand::prelude::SliceRandom;
pub mod npc;

pub fn roll_level(level_kind: &LevelKind, stats: &mut DungeonStats) -> DungeonLevel {
	let mut level = roll_rooms(level_kind.depth, level_kind.is_max, level_kind.party_type);
	roll_amulet(&level_kind, &mut level);
	roll_objects(&mut level, stats);
	roll_stairs(&mut level);
	roll_traps(&mut level);
	roll_monsters(&mut level);
	level
}

fn roll_amulet(level_kind: &LevelKind, level: &mut DungeonLevel) {
	if !level_kind.post_amulet && level_kind.depth >= AMULET_LEVEL as usize {
		let amulet = Object::new(ObjectWhat::Amulet);
		let spot = level.roll_vacant_spot(false, false, false);
		level.put_object(spot, amulet);
	}
}

fn roll_traps(level: &mut DungeonLevel) {
	for i in 0..roll_traps_count(level) {
		let trap = Trap { trap_type: TrapKind::random(), ..Trap::default() };
		let spot = match level.party_room {
			Some(party_room) if i == 0 => {
				let room = level.room_at(party_room).expect("invalid party room");
				let bounds = room.bounds.inset(1, 1);
				let mut found = None;
				'search: for _ in 0..15 {
					let spot = bounds.roll_spot();
					if level.spot_is_vacant(spot, false, true) {
						found = Some(spot);
						break 'search;
					}
				}
				found.unwrap_or_else(|| {
					level.roll_vacant_spot(true, true, false)
				})
			}
			_ => level.roll_vacant_spot(true, true, false),
		};
		level.put_trap(spot, trap);
	}
}

fn roll_traps_count(level: &DungeonLevel) -> usize {
	const AMULET_LEVEL_AND_TWO: usize = (AMULET_LEVEL + 2) as usize;
	match level.depth {
		0..=2 => 0,
		3..=7 => get_rand(0, 2),
		8..=11 => get_rand(1, 2),
		12..=16 => get_rand(2, 3),
		17..=21 => get_rand(2, 4),
		22..=AMULET_LEVEL_AND_TWO => get_rand(3, 5),
		_ => get_rand(5, MAX_TRAP)
	}
}

fn roll_rooms(depth: usize, is_max: bool, party_type: PartyType) -> DungeonLevel {
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
			party_room: Some(RoomId::Big),
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
		DungeonLevel { depth, is_max, ty: party_type, rooms, map, rogue_spot: LevelSpot::from_i64(0, 0), party_room: None }
	}
}

fn roll_build_big_room(level_type: PartyType) -> bool {
	match level_type {
		PartyType::PartyBig => true,
		PartyType::PartyRollBig => rand_percent(1),
		PartyType::NoParty => false
	}
}

pub fn roll_stairs(level: &mut DungeonLevel) {
	let spot = level.roll_vacant_spot(false, false, false);
	level.put_stairs(spot);
}

pub fn roll_objects(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	if level.is_max {
		if level.ty.is_party() {
			party::roll_party(level, stats);
		}
		for _ in 0..roll_object_count() {
			let spot = level.roll_vacant_spot(false, false, false);
			let object = roll_object(level.depth, stats);
			level.put_object(spot, object);
		}
		roll_gold(level);
	}
}

fn roll_vault_or_maze(level: &DungeonLevel) -> RoomId {
	let mut rooms = level.vaults_and_mazes();
	rooms.shuffle(&mut rand::thread_rng());
	*rooms.first().expect("no vault or maze in level")
}


fn roll_gold(level: &mut DungeonLevel) {
	let rooms_and_mazes = level.vaults_and_mazes();
	for room_id in rooms_and_mazes {
		let room = level.room_at(room_id).expect("level should have room");
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

#[derive(Copy, Clone)]
pub struct LevelKind {
	pub depth: usize,
	pub is_max: bool,
	pub post_amulet: bool,
	pub party_type: PartyType,
}

pub mod random_what;
pub mod party;
