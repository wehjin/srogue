use crate::objects::Object;
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
		for _ in 0..roll_drop_count() {
			let object = roll_object(depth, stats);
			let spot = level.roll_drop_spot();
			level.put_object(spot, object);
		}
		//put_gold(game.player.cur_depth, &mut game.level, &mut game.ground);
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


fn roll_drop_count() -> usize {
	let mut n = if coin_toss() { get_rand(2, 4) } else { get_rand(3, 5) };
	while rand_percent(33) {
		n += 1;
	}
	n
}

pub mod random_what;
