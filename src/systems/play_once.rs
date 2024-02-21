use keyboard::CTRL_W_CHAR;

use crate::actions::PLAYER_ACTIONS;
use crate::hit::fight;
use crate::init::{GameState, GameTurn};
use crate::inventory::{inv_armor_weapon, inventory, single_inv};
use crate::level::{check_up, drop_check, show_average_hp, UpResult};
use crate::monster::show_monsters;
use crate::motion::{move_onto, multiple_move_rogue, one_move_rogue, reg_move, rest};
use crate::objects::{new_object_for_wizard, show_objects};
use crate::pack::{call_it, drop_0, kick_into_pack};
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::r#use::{eat, quaff, read_scroll};
use crate::render_system;
use crate::resources::keyboard;
use crate::resources::keyboard::rgetchar;
use crate::ring::inv_rings;
use crate::room::draw_magic_map;
use crate::save::save_game;
use crate::score::ask_quit;
use crate::systems::play_level::{PlayResult, UNKNOWN_COMMAND};
use crate::systems::play_level::PlayResult::{ExitWon, StairsDown, StairsUp};
use crate::systems::play_once::PlayOnceResult::{Counting, Leaving};
use crate::throw::throw;
use crate::trap::{id_trap, search, show_traps};
use crate::zap::{wizardize, zapp};

pub enum PlayOnceResult {
	Counting(String),
	Leaving(PlayResult),
	Idle,
}

pub fn play_once(key_code: Option<char>, game: &mut GameState) -> PlayOnceResult {
	if let Some(ending) = check_reset_loop_flags(game) {
		return Leaving(ending);
	};
	let key_code = key_code.unwrap_or_else(rgetchar);
	// Keep rgetchar above clear_message(). Otherwise, the dialog row on screen
	// does not draw correctly.
	game.dialog.clear_message();
	game.turn = GameTurn::Player;
	if let Some(player_action) = PLAYER_ACTIONS.get(key_code) {
		player_action.commit(game);
		if game.turn == GameTurn::Monsters {
			reg_move(game);
		}
	} else {
		match key_code {
			'.' => rest(1, game),
			's' => search(1, false, game),
			'i' => inventory(AllObjects, game),
			'f' => fight(false, game),
			'F' => fight(true, game),
			'h' | 'j' | 'k' | 'l' | 'y' | 'u' | 'n' | 'b' => {
				one_move_rogue(key_code, true, game);
			}
			'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N'
			| keyboard::CTRL_H | keyboard::CTRL_J | keyboard::CTRL_K | keyboard::CTRL_L
			| keyboard::CTRL_Y | keyboard::CTRL_U | keyboard::CTRL_N | keyboard::CTRL_B =>
				multiple_move_rogue(key_code as i64, game),
			'e' => eat(game),
			'q' => quaff(game),
			'r' => read_scroll(game),
			'm' => move_onto(game),
			'd' => drop_0(game),
			'\x10' => game.dialog.re_message(),
			CTRL_W_CHAR => wizardize(game),
			'>' => if drop_check(game) {
				return Leaving(StairsDown);
			},
			'<' => match check_up(game) {
				UpResult::KeepLevel => {
					// Ignore and stay in loop
				}
				UpResult::UpLevel => {
					return Leaving(StairsUp);
				}
				UpResult::WonGame => {
					return Leaving(ExitWon);
				}
			},
			')' | ']' => inv_armor_weapon(key_code == ')', game),
			'=' => inv_rings(game),
			'^' => id_trap(game),
			'I' => single_inv(None, game),
			'c' => call_it(game),
			'z' => zapp(game),
			't' => throw(game),
			'v' => game.dialog.message("rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims", 0),
			'Q' => if ask_quit(false, game) {
				return Leaving(PlayResult::ExitQuit);
			},
			'0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
				render_system::refresh(game);
				return Counting(key_code.to_string());
			}
			' ' => {}
			'\x09' => if game.player.wizard {
				inventory(AllObjects, game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x13' => if game.player.wizard {
				draw_magic_map(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x14' => if game.player.wizard {
				show_traps(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x0f' => if game.player.wizard {
				show_objects(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x01' => {
				show_average_hp(game);
			}
			'\x03' => if game.player.wizard {
				new_object_for_wizard(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'\x0d' => if game.player.wizard {
				show_monsters(game);
			} else {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			},
			'S' => if save_game(game) {
				return Leaving(PlayResult::ExitSaved);
			},
			',' => {
				kick_into_pack(game);
			}
			_ => {
				game.dialog.message(UNKNOWN_COMMAND, 0);
			}
		}
	}
	render_system::refresh(game);
	return PlayOnceResult::Idle;
}

fn check_reset_loop_flags(game: &mut GameState) -> Option<PlayResult> {
	game.player.interrupted = false;
	if !game.player.hit_message.is_empty() {
		game.player.interrupt_and_slurp();
		game.dialog.message(&game.player.hit_message, 1);
		game.player.hit_message.clear();
	}
	if game.level.trap_door {
		game.level.trap_door = false;
		return Some(PlayResult::TrapDoorDown);
	}
	render_system::refresh(game);
	return None;
}

