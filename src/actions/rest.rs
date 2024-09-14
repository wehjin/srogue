use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::level::show_average_hp;
use crate::objects::NoteStatus::Called;
use crate::objects::Title;
use crate::pack;
use crate::prelude::object_what::ObjectWhat::{Potion, Ring, Scroll, Wand};
use crate::prelude::object_what::PackFilter::AnyFrom;
use crate::resources::input_line::get_input_line;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::save::save_game;
use crate::score::ask_quit;
use crate::systems::play_level::LevelResult;

pub struct SaveGame;

impl GameUpdater for SaveGame {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if save_game(game) {
			Some(LevelResult::ExitSaved)
		} else {
			None
		}
	}
}

pub struct ShowAverageHp;

impl GameUpdater for ShowAverageHp {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		show_average_hp(game);
		None
	}
}

pub struct Ignore;

impl GameUpdater for Ignore {
	fn update(_game: &mut GameState) -> Option<LevelResult> {
		None
	}
}

pub struct Quit;

impl GameUpdater for Quit {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if ask_quit(false, game) {
			Some(LevelResult::ExitQuit)
		} else {
			None
		}
	}
}


pub struct Version;

impl GameUpdater for Version {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		game.dialog.message("rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims", 0);
		None
	}
}

pub struct CallIt;

impl GameUpdater for CallIt {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		call_it(game);
		None
	}
}

pub struct Rest;

impl GameUpdater for Rest {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		rest(game);
		None
	}
}

fn rest(game: &mut GameState) {
	game.yield_turn_to_monsters();
}

fn call_it(game: &mut GameState) {
	let ch = pack::pack_letter("call what?", AnyFrom(vec![Scroll, Potion, Wand, Ring]), game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			let what = game.player.object_what(obj_id);
			match what {
				Scroll | Potion | Wand | Ring => {
					let kind = game.player.object_kind(obj_id);
					let new_name = get_input_line::<String>("call it:", None, Some(game.player.notes.title(what, kind as usize).as_str()), true, true, &mut game.dialog);
					if !new_name.is_empty() {
						let id = game.player.notes.note_mut(what, kind as usize);
						id.status = Called;
						id.title = Title::UserString(new_name);
					}
				}
				_ => {
					game.dialog.message("surely you already know what that's called", 0);
					return;
				}
			}
		}
	}
}


