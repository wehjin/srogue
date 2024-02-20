use std::collections::HashMap;

use crate::init::GameState;
use crate::objects::ObjectId;
use crate::prelude::DungeonSpot;
use crate::render_system::{backend, FLAME_CHAR};
use crate::render_system::backend::{get_char, set_char, swap_char};
use crate::throw::ThrowEnding;

pub fn animate_throw(obj_id: ObjectId, throw_ending: &ThrowEnding, game: &GameState) {
	let path = throw_ending.flight_path();
	let what = game.player.object_what(obj_id);
	let char = what.to_char();
	backend::show_cursor(false);
	for spot in path {
		if game.player_can_see(*spot) {
			let restore_char = swap_char(char, *spot);
			backend::move_cursor(*spot);
			backend::await_frame();
			set_char(restore_char, *spot);
		}
	}
	backend::await_frame();
	backend::show_cursor(true);
}

pub fn animate_flame_broil(path: &Vec<DungeonSpot>) {
	let mut originals = HashMap::new();
	backend::stand_out(true);
	backend::show_cursor(false);
	for spot in path {
		originals.insert(*spot, get_char(*spot));
		set_char(FLAME_CHAR, *spot);
		backend::await_frame();
	}
	backend::stand_out(false);
	for spot in path {
		let original = originals[spot];
		set_char(original, *spot);
		backend::await_frame();
	}
	backend::show_cursor(true);
}

