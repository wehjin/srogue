use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::objects::NoteStatus::Called;
use crate::objects::Title;
use crate::pack;
use crate::prelude::object_what::ObjectWhat::{Potion, Ring, Scroll, Wand};
use crate::prelude::object_what::PackFilter::AnyFrom;
use crate::resources::input_line::get_input_line;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::systems::play_level::PlayResult;

pub struct CallIt;

impl PlayerAction for CallIt {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		call_it(game);
		None
	}
}

pub struct Rest;

impl PlayerAction for Rest {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
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


