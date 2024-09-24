use crate::level::constants::MAX_TRAP;
use crate::objects::Object;
use crate::odds::GOLD_PERCENT;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::AMULET_LEVEL;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::game::RogueSpot;
use crate::resources::level::design::roll_design;
use crate::resources::level::feature_grid::FeatureGrid;
use crate::resources::level::plain::PlainLevel;
use crate::resources::level::room::LevelRoom;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::sector::{ALL_SECTORS, COL0, COL3, ROW0, ROW3};
use crate::resources::level::setup::npc::roll_monsters;
use crate::resources::level::setup::random_what::RandomWhat;
use crate::resources::level::torch_grid::TorchGrid;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::room::{RoomBounds, RoomType};
use crate::trap::trap_kind::TrapKind;
use crate::trap::Trap;
use rand::prelude::SliceRandom;
use rand::Rng;

pub mod npc;

pub fn roll_level(level_kind: &LevelKind, stats: &mut DungeonStats, rng: &mut impl Rng) -> DungeonLevel {
	let mut level = roll_rooms(level_kind.depth, level_kind.is_max, level_kind.party_type, rng);
	roll_amulet(&level_kind, &mut level, rng);
	roll_objects(&mut level, stats, rng);
	roll_stairs(&mut level, rng);
	roll_traps(&mut level, rng);
	roll_monsters(&mut level, rng);
	rogue::roll_rogue(&mut level, rng);
	level
}

fn roll_amulet(level_kind: &LevelKind, level: &mut DungeonLevel, rng: &mut impl Rng) {
	if !level_kind.post_amulet && level_kind.depth >= AMULET_LEVEL as usize {
		let amulet = Object::new(ObjectWhat::Amulet, rng);
		let spot = level.roll_vacant_spot(false, false, false, rng);
		level.put_object(spot, amulet);
	}
}

fn roll_traps(level: &mut DungeonLevel, rng: &mut impl Rng) {
	for i in 0..roll_traps_count(level, rng) {
		let trap = Trap { trap_type: TrapKind::random(rng), ..Trap::default() };
		let spot = match level.party_room {
			Some(party_room) if i == 0 => {
				let bounds = level.as_room(party_room).bounds.inset(1, 1);
				let mut found = None;
				'search: for _ in 0..15 {
					let spot = bounds.roll_spot(rng);
					if level.spot_is_vacant(spot, false, true) {
						found = Some(spot);
						break 'search;
					}
				}
				found.unwrap_or_else(|| {
					level.roll_vacant_spot(true, true, false, rng)
				})
			}
			_ => level.roll_vacant_spot(true, true, false, rng),
		};
		level.put_trap(spot, trap);
	}
}

fn roll_traps_count(level: &DungeonLevel, rng: &mut impl Rng) -> usize {
	const AMULET_LEVEL_AND_TWO: usize = (AMULET_LEVEL + 2) as usize;
	match level.depth {
		0..=2 => 0,
		3..=7 => rng.gen_range(0..=2),
		8..=11 => rng.gen_range(1..=2),
		12..=16 => rng.gen_range(2..=3),
		17..=21 => rng.gen_range(2..=4),
		22..=AMULET_LEVEL_AND_TWO => rng.gen_range(3..=5),
		_ => rng.gen_range(5..=MAX_TRAP)
	}
}

fn roll_rooms(depth: usize, is_max: bool, party_type: PartyType, rng: &mut impl Rng) -> DungeonLevel {
	if roll_build_big_room(party_type, rng) {
		let y = ROW0 + 1;
		let x = ROW3 - 6;
		let y1 = ROW3 - 1;
		let x1 = COL3 - 11;
		let y2 = COL3 - 1;
		let bounds = RoomBounds {
			top: rng.gen_range(ROW0..=y),
			bottom: rng.gen_range(x..=y1),
			left: rng.gen_range(COL0..=10),
			right: rng.gen_range(x1..=y2),
		};
		let room = LevelRoom { ty: RoomType::Room, bounds, ..LevelRoom::default() };
		DungeonLevel {
			depth,
			is_max,
			ty: party_type,
			rooms: vec![(RoomId::Big, room)].into_iter().collect(),
			features: FeatureGrid::new().put_walls_and_floor(bounds),
			torches: TorchGrid::new(),
			rogue_spot: RogueSpot::None,
			party_room: Some(RoomId::Big),
			lighting_enabled: false,
			objects: Default::default(),
			monsters: Default::default(),
		}
	} else {
		let design = roll_design(rng);
		let level = PlainLevel::new(depth, rng)
			.add_rooms(design, rng)
			.add_mazes(rng)
			.connect_spaces(rng)
			.add_deadends(rng)
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
		DungeonLevel {
			depth,
			is_max,
			ty: party_type,
			rooms,
			features: map,
			torches: TorchGrid::new(),
			rogue_spot: RogueSpot::None,
			party_room: None,
			lighting_enabled: false,
			objects: Default::default(),
			monsters: Default::default(),
		}
	}
}

fn roll_build_big_room(level_type: PartyType, rng: &mut impl Rng) -> bool {
	match level_type {
		PartyType::PartyBig => true,
		PartyType::PartyRollBig => rng.gen_ratio(1, 100),
		PartyType::NoParty => false
	}
}

pub fn roll_stairs(level: &mut DungeonLevel, rng: &mut impl Rng) {
	let spot = level.roll_vacant_spot(false, false, false, rng);
	level.put_stairs(spot);
}

pub fn roll_objects(level: &mut DungeonLevel, stats: &mut DungeonStats, rng: &mut impl Rng) {
	if level.is_max {
		if level.ty.is_party() {
			party::roll_party(level, stats, rng);
		}
		for _ in 0..roll_object_count(rng) {
			let spot = level.roll_vacant_spot(false, false, false, rng);
			let object = roll_object(level.depth, &mut stats.food_drops, rng);
			level.put_object(spot, object);
		}
		roll_gold(level, rng);
	}
}

fn roll_vault_or_maze(level: &DungeonLevel, rng: &mut impl Rng) -> RoomId {
	let mut rooms = level.vault_and_maze_rooms();
	rooms.shuffle(rng);
	*rooms.first().expect("no vault or maze in level")
}


fn roll_gold(level: &mut DungeonLevel, rng: &mut impl Rng) {
	for room_id in level.vault_and_maze_rooms() {
		let room = level.as_room(room_id);
		let is_maze = room.is_maze();
		if is_maze || rng.gen_ratio(GOLD_PERCENT as u32, 100) {
			let search_bounds = room.bounds.inset(1, 1);
			'search: for _ in 0..50 {
				let spot = search_bounds.roll_spot(rng);
				if level.spot_is_floor_or_tunnel(spot) {
					let object = Object::roll_gold(level.depth, is_maze, rng);
					level.put_object(spot, object);
					break 'search;
				}
			}
		}
	}
}

pub fn roll_object(depth: usize, all_food_drops: &mut usize, rng: &mut impl Rng) -> Object {
	let what = if *all_food_drops < depth / 2 {
		*all_food_drops += 1;
		RandomWhat::Food
	} else {
		RandomWhat::roll(rng)
	};
	match what {
		RandomWhat::Scroll => Object::roll_scroll(rng),
		RandomWhat::Potion => Object::roll_potion(rng),
		RandomWhat::Weapon => Object::roll_weapon(true, rng),
		RandomWhat::Armor => Object::roll_armor(rng),
		RandomWhat::Wand => Object::roll_wand(rng),
		RandomWhat::Food => Object::roll_food(false, rng),
		RandomWhat::Ring => Object::roll_ring(true, rng),
	}
}

fn roll_object_count(rng: &mut impl Rng) -> usize {
	let mut n = if rng.gen_bool(0.5) { rng.gen_range(2..=4) } else { rng.gen_range(3..=5) };
	while rng.gen_ratio(33, 100) {
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

pub mod party;
pub mod random_what;
pub mod rogue;

