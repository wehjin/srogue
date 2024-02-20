use std::collections::HashMap;

use crate::init::GameState;
use crate::objects::ObjectId;
use crate::prelude::DungeonSpot;
use crate::render_system;
use crate::render_system::{FLAME_CHAR, get_char, set_char, swap_char};
use crate::throw::ThrowEnding;

pub fn animate_throw(obj_id: ObjectId, throw_ending: &ThrowEnding, game: &GameState) {
	let path = throw_ending.flight_path();
	let what = game.player.object_what(obj_id);
	let char = what.to_char();
	render_system::show_cursor(false);
	for spot in path {
		if game.player_can_see(*spot) {
			let restore_char = swap_char(char, *spot);
			render_system::move_curs(spot);
			render_system::await_frame();
			set_char(restore_char, *spot);
		}
	}
	render_system::await_frame();
	render_system::show_cursor(true);
}

pub fn animate_flame_broil(path: &Vec<DungeonSpot>) {
	let mut originals = HashMap::new();
	render_system::standout(true);
	render_system::show_cursor(false);
	for spot in path {
		originals.insert(*spot, get_char(*spot));
		set_char(FLAME_CHAR, *spot);
		render_system::await_frame();
	}
	render_system::standout(false);
	for spot in path {
		let original = originals[spot];
		set_char(original, *spot);
		render_system::await_frame();
	}
	render_system::show_cursor(true);
}

