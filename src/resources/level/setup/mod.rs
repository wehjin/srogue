use crate::objects::Object;
use crate::odds::GOLD_PERCENT;
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::setup::random_what::RandomWhat;
use crate::resources::level::DungeonLevel;

pub fn roll_objects(level: &mut DungeonLevel, stats: &mut DungeonStats) {
	if level.is_max {
		let depth = level.depth;
		// TODO
		// if depth == party_depth.usize() {
		// 	make_party(game.player.cur_depth, game);
		// }
		let count = roll_object_count();
		for _ in 0..count {
			let spot = level.roll_object_spot();
			let object = roll_object(depth, stats);
			level.put_object(spot, object);
		}
		roll_gold(level);
	}
}

fn roll_gold(level: &mut DungeonLevel) {
	let rooms_and_mazes = level.vaults_and_mazes();
	for room_id in rooms_and_mazes {
		let room = level.as_room(room_id).expect("level should have room");
		if room.is_maze() || rand_percent(GOLD_PERCENT) {
			let search_bounds = room.bounds.inset(1, 1);
			for _ in 0..50 {
				let spot = search_bounds.roll_spot();
				if level.has_floor_or_tunnel_at_spot(spot) {
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
