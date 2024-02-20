use crate::init::GameState;
use crate::random::get_rand;
use crate::render_system::DISGUISE_CHARS;
use crate::render_system::backend::{get_char, set_char};

pub fn show_hallucination(game: &mut GameState) {
	if game.player.blind.is_active() {
		return;
	}
	fn is_object_ch(ch: char) -> bool {
		match ch as u8 as char {
			' ' | '.' | '#' | '+' => false,
			_ => true
		}
	}
	fn is_monster_ch(ch: char) -> bool {
		match ch as u8 as char {
			'A'..='Z' => true,
			_ => false,
		}
	}
	for obj in game.ground.objects() {
		let spot = obj.to_spot();
		let ch = get_char(spot);
		if !game.player_is_at(spot) && !is_monster_ch(ch) && is_object_ch(ch) {
			let random_ch = gr_obj_char();
			set_char(random_ch, spot);
		}
	}
	for monster in &game.mash.monsters() {
		let ch = get_char(monster.spot);
		if is_monster_ch(ch) {
			let random_ch = get_rand('A', 'Z');
			set_char(random_ch, monster.spot);
		}
	}
}

pub(crate) fn gr_obj_char() -> char {
	let index = get_rand(0, DISGUISE_CHARS.len() - 1);
	let char = DISGUISE_CHARS[index];
	char
}
