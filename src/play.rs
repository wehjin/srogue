#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::c_int;
use ncurses::{mv, refresh};

use crate::actions::PlayerActionSet;
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::hit::{fight, HIT_MESSAGE};
use crate::init::{GameState, GameSystem};
use crate::instruct::Instructions;
use crate::inventory::{inv_armor_weapon, inventory, single_inv};
use crate::level::{check_up, drop_check, show_average_hp, UpResult};
use crate::message::{CANCEL, remessage, rgetchar};
use crate::monster::show_monsters;
use crate::objects::{new_object_for_wizard, show_objects};
use crate::pack::{call_it, drop_0, kick_into_pack};
use crate::play::PlayResult::{CleanedUp, ExitWon, StairsDown, StairsUp};
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::r#move::{move_onto, multiple_move_rogue, one_move_rogue, reg_move, rest};
use crate::r#use::{eat, quaff, read_scroll};
use crate::ring::inv_rings;
use crate::room::draw_magic_map;
use crate::save::save_game;
use crate::score::ask_quit;
use crate::throw::throw;
use crate::trap::{id_trap, search, show_traps};
use crate::zap::{wizardize, zapp};


pub const UNKNOWN_COMMAND: &'static str = "unknown command";

pub enum PlayResult {
	TrapDoorDown,
	StairsDown,
	StairsUp,
	ExitWon,
	ExitQuit,
	ExitSaved,
	CleanedUp(String),
}

pub unsafe fn play_level(game: &mut GameState) -> PlayResult {
	let mut count = 0;
	let mut deck_ch = None;
	let player_actions = PlayerActionSet::new(vec![
		('P', Box::new(PutOnRing)),
		('R', Box::new(RemoveRing)),
		('T', Box::new(TakeOff)),
		('W', Box::new(Wear)),
		('w', Box::new(Wield)),
	]);
	loop {
		if let Some(exit) = &game.player.cleaned_up {
			return CleanedUp(exit.to_string());
		}
		let ch = {
			let ch = match deck_ch {
				Some(deck_ch) => deck_ch,
				None => {
					game.player.interrupted = false;
					if !HIT_MESSAGE.is_empty() {
						game.player.interrupt_and_slurp();
						game.dialog.message(&HIT_MESSAGE, 1);
						HIT_MESSAGE.clear();
					}
					if game.level.trap_door {
						game.level.trap_door = false;
						return PlayResult::TrapDoorDown;
					}
					mv(game.player.rogue.row as i32, game.player.rogue.col as i32);
					refresh();
					let ch = rgetchar();
					game.dialog.clear_message();
					count = 0;
					ch
				}
			};
			deck_ch = None;
			ch
		};
		game.next_system = GameSystem::PlayerActions;
		if let Some(player_action) = player_actions.get(ch) {
			player_action.commit(game);
			if game.next_system == GameSystem::MonsterActions {
				reg_move(game);
			}
		} else {
			match ch {
				'?' => Instructions(game),
				'.' => rest(if count > 0 { count } else { 1 } as c_int, game),
				's' => search(if count > 0 { count } else { 1 } as usize, false, game),
				'i' => inventory(AllObjects, game),
				'f' => fight(false, game),
				'F' => fight(true, game),
				'h' | 'j' | 'k' | 'l' | 'y' | 'u' | 'n' | 'b' => {
					one_move_rogue(ch, true, game);
				}
				'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' | '\x08' | '\x0a' | '\x0b' | '\x0c' | '\x19' | '\x15' | '\x0e' | '\x02' => multiple_move_rogue(ch as i64, game),
				'e' => eat(game),
				'q' => quaff(game),
				'r' => read_scroll(game),
				'm' => move_onto(game),
				'd' => drop_0(game),
				'\x10' => remessage(game),
				'\x17' => wizardize(game),
				'>' => {
					if drop_check(game) {
						return StairsDown;
					}
				}
				'<' => {
					match check_up(game) {
						UpResult::KeepLevel => {
							// Ignore and stay in loop
						}
						UpResult::UpLevel => {
							return StairsUp;
						}
						UpResult::WonGame => {
							return ExitWon;
						}
					}
				}
				')' | ']' => inv_armor_weapon(ch == ')', game),
				'=' => inv_rings(game),
				'^' => id_trap(game),
				'I' => single_inv(None, game),
				'c' => call_it(game),
				'z' => zapp(game),
				't' => throw(game),
				'v' => game.dialog.message("rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims", 0),
				'Q' => {
					if ask_quit(false, game) {
						return PlayResult::ExitQuit;
					}
				}
				'0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
					mv(game.player.rogue.row as i32, game.player.rogue.col as i32);
					refresh();
					let mut count_ch = ch;
					loop {
						if count < 100 {
							count = (10 * count) + count_ch.to_digit(10).expect("digit");
						}
						count_ch = rgetchar();
						if !count_ch.is_digit(10) {
							if count_ch != CANCEL {
								deck_ch = Some(count_ch)
							}
							break;
						}
					}
				}
				' ' => {}
				'\x09' => {
					if game.player.wizard {
						inventory(AllObjects, game);
					} else {
						game.dialog.message(UNKNOWN_COMMAND, 0);
					}
				}
				'\x13' => {
					if game.player.wizard {
						draw_magic_map(&mut game.mash, &mut game.level);
					} else {
						game.dialog.message(UNKNOWN_COMMAND, 0);
					}
				}
				'\x14' => {
					if game.player.wizard {
						show_traps(&game.level);
					} else {
						game.dialog.message(UNKNOWN_COMMAND, 0);
					}
				}
				'\x0f' => {
					if game.player.wizard {
						show_objects(game);
					} else {
						game.dialog.message(UNKNOWN_COMMAND, 0);
					}
				}
				'\x01' => {
					show_average_hp(game);
				}
				'\x03' => {
					if game.player.wizard {
						new_object_for_wizard(game);
					} else {
						game.dialog.message(UNKNOWN_COMMAND, 0);
					}
				}
				'\x0d' => {
					if game.player.wizard {
						show_monsters(game);
					} else {
						game.dialog.message(UNKNOWN_COMMAND, 0);
					}
				}
				'S' => {
					if save_game(game) {
						return PlayResult::ExitSaved;
					}
				}
				',' => {
					kick_into_pack(game);
				}
				_ => {
					game.dialog.message(UNKNOWN_COMMAND, 0);
				}
			}
		}
	}
}
