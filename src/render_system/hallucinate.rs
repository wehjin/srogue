use ncurses::chtype;

use crate::init::GameState;
use crate::random::get_rand;
use crate::render_system::{get_ch, gr_obj_ch, set_ch};

pub fn show_hallucination(game: &mut GameState) {
	if game.player.blind.is_active() {
		return;
	}
	fn is_object_ch(ch: chtype) -> bool {
		match ch as u8 as char {
			' ' | '.' | '#' | '+' => false,
			_ => true
		}
	}
	pub fn is_monster_ch(ch: chtype) -> bool {
		match ch as u8 as char {
			'A'..='Z' => true,
			_ => false,
		}
	}
	for obj in game.ground.objects() {
		let spot = obj.to_spot();
		let ch = get_ch(&spot);
		if !game.player_is_at(spot) && !is_monster_ch(ch) && is_object_ch(ch) {
			let random_ch = gr_obj_ch();
			set_ch(random_ch, &spot);
		}
	}
	for monster in &game.mash.monsters {
		let ch = get_ch(&monster.spot);
		if is_monster_ch(ch) {
			let random_ch = get_rand(chtype::from('A'), chtype::from('Z'));
			set_ch(random_ch, &monster.spot);
		}
	}
}


