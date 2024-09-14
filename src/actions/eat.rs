use crate::actions::GameUpdater;
use crate::components::hunger::HungerLevel;
use crate::init::GameState;
use crate::level::add_exp;
use crate::pack::pack_letter;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::ObjectWhat::Food;
use crate::prelude::object_what::PackFilter::Foods;
use crate::random::{get_rand, rand_percent};
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::LevelResult;

pub struct Eat;

impl GameUpdater for Eat {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		eat(game);
		None
	}
}


fn eat(game: &mut GameState) {
	let ch = pack_letter("eat what?", Foods, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			if game.player.object_what(obj_id) != Food {
				game.dialog.message("you can't eat that", 0);
				return;
			}
			let kind = game.player.object_kind(obj_id);
			let moves = if kind == FRUIT || rand_percent(60) {
				let msg = if kind == RATION {
					"yum, that tasted good".to_string()
				} else {
					format!("my, that was a yummy {}", &game.player.settings.fruit)
				};
				game.dialog.message(&msg, 0);
				get_rand(900, 1100)
			} else {
				game.dialog.message("yuk, that food tasted awful", 0);
				add_exp(2, true, game);
				get_rand(700, 900)
			};
			game.player.rogue.moves_left /= 3;
			game.player.rogue.moves_left += moves;
			game.player.hunger = HungerLevel::Normal;
			game.stats_changed = true;
			crate::r#use::vanish(obj_id, true, game);
		}
	}
}
