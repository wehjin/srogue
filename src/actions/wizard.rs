use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::resources::input_line::get_input_line;
use crate::room::draw_magic_map;
use crate::systems::play_level::{PlayResult, UNKNOWN_COMMAND};
use crate::trap::show_traps;

pub struct ShowTraps;

impl PlayerAction for ShowTraps {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		if game.player.wizard {
			show_traps(game);
		} else {
			game.dialog.message(UNKNOWN_COMMAND, 0);
		}
		None
	}
}

pub struct DrawMagicMap;

impl PlayerAction for DrawMagicMap {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		if game.player.wizard {
			draw_magic_map(game);
		} else {
			game.dialog.message(UNKNOWN_COMMAND, 0);
		}
		None
	}
}

pub struct Wizardize;

impl PlayerAction for Wizardize {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		wizardize(game);
		None
	}
}

fn wizardize(game: &mut GameState) {
	if game.player.wizard {
		game.player.wizard = false;
		game.dialog.message("not wizard anymore", 0);
	} else {
		let line = get_input_line::<String>("wizard's password:", None, None, false, false, &mut game.dialog);
		if !line.is_empty() {
			//const PW: &str = "\u{A7}DV\u{BA}M\u{A3}\u{17}";
			const PW: &str = "neko?";
			if line == PW {
				game.player.wizard = true;
				game.player.settings.score_only = true;
				game.dialog.message("Welcome, mighty wizard!", 0);
			} else {
				game.dialog.message("sorry", 0);
			}
		}
	}
}
