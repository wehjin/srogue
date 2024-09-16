use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::monster::show_monsters;
use crate::objects::{new_object_for_wizard, show_objects};

use crate::resources::input_line::get_input_line;
use crate::room::draw_magic_map;
use crate::systems::play_level::{LevelResult, UNKNOWN_COMMAND};
use crate::trap::show_traps;

pub struct ShowMonsters;

impl GameUpdater for ShowMonsters {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wizard {
			show_monsters(game);
		} else {
			game.diary.add_entry(UNKNOWN_COMMAND);
		}
		None
	}
}

pub struct NewObjectForWizard;

impl GameUpdater for NewObjectForWizard {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wizard {
			new_object_for_wizard(game);
		} else {
			game.diary.add_entry(UNKNOWN_COMMAND);
		}
		None
	}
}

pub struct ShowObjects;

impl GameUpdater for ShowObjects {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wizard {
			show_objects(game);
		} else {
			game.diary.add_entry(UNKNOWN_COMMAND);
		}
		None
	}
}

pub struct ShowTraps;

impl GameUpdater for ShowTraps {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wizard {
			// TODO Fix this to show where the trap is without making it visible so that the Search and IdTrap actions can still work.
			show_traps(game);
		} else {
			game.diary.add_entry(UNKNOWN_COMMAND);
		}
		None
	}
}

pub struct DrawMagicMap;

impl GameUpdater for DrawMagicMap {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wizard {
			draw_magic_map(game);
		} else {
			game.diary.add_entry(UNKNOWN_COMMAND);
		}
		None
	}
}

pub struct Wizardize;

impl GameUpdater for Wizardize {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		wizardize(game);
		None
	}
}

fn wizardize(game: &mut GameState) {
	if game.player.wizard {
		game.player.wizard = false;
		game.diary.add_entry("not wizard anymore");
	} else {
		let line = get_input_line::<String>("wizard's password:", None, None, false, false, &mut game.diary);
		if !line.is_empty() {
			//const PW: &str = "\u{A7}DV\u{BA}M\u{A3}\u{17}";
			const PW: &str = "neko?";
			if line == PW {
				game.player.wizard = true;
				game.player.settings.score_only = true;
				game.diary.add_entry("Welcome, mighty wizard!");
			} else {
				game.diary.add_entry("sorry");
			}
		}
	}
}
