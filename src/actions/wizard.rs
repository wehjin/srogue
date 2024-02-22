use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::resources::input_line::get_input_line;

pub struct Wizardize;

impl PlayerAction for Wizardize {
	fn update(_input_key: char, game: &mut GameState) {
		wizardize(game)
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
