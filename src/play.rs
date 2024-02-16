#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::c_int;
use ncurses::{mv, refresh};

use crate::hit::{fight, HIT_MESSAGE};
use crate::init::GameState;
use crate::instruct::Instructions;
use crate::inventory::{inv_armor_weapon, inventory, single_inv};
use crate::level::{check_up, drop_check, show_average_hp, UpResult};
use crate::message::{CANCEL, check_message, message, remessage, rgetchar};
use crate::monster::show_monsters;
use crate::objects::{new_object_for_wizard, show_objects};
use crate::pack::{call_it, drop_0, kick_into_pack, take_off, wear, wield};
use crate::play::PlayResult::{CleanedUp, ExitWon, StairsDown, StairsUp};
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::r#move::{move_onto, multiple_move_rogue, one_move_rogue, rest};
use crate::r#use::{eat, quaff, read_scroll};
use crate::ring::{inv_rings, put_on_ring, remove_ring};
use crate::room::draw_magic_map;
use crate::save::save_game;
use crate::score::ask_quit;
use crate::throw::throw;
use crate::trap::{id_trap, search, show_traps};
use crate::zap::{wizard, wizardize, zapp};

pub static mut interrupted: bool = false;

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
	loop {
		if let Some(exit) = &game.player.cleaned_up {
			return CleanedUp(exit.to_string());
		}
		let ch = if let Some(deck_ch) = deck_ch {
			deck_ch
		} else {
			interrupted = false;
			if !HIT_MESSAGE.is_empty() {
				message(&HIT_MESSAGE, 1);
				HIT_MESSAGE.clear();
			}
			if game.level.trap_door {
				game.level.trap_door = false;
				return PlayResult::TrapDoorDown;
			}
			mv(game.player.rogue.row as i32, game.player.rogue.col as i32);
			refresh();
			let ch = rgetchar();
			check_message();
			count = 0;
			ch
		};
		deck_ch = None;
		match ch {
			'?' => {
				Instructions();
			}
			'.' => {
				rest(if count > 0 { count } else { 1 } as c_int, &mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			's' => {
				search(if count > 0 { count } else { 1 } as usize, false, &mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'i' => {
				inventory(AllObjects, &game.player);
			}
			'f' => {
				fight(false, &mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'F' => {
				fight(true, &mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'h' | 'j' | 'k' | 'l' | 'y' | 'u' | 'n' | 'b' => {
				one_move_rogue(ch, true, &mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' | '\x08' | '\x0a' | '\x0b' | '\x0c' | '\x19' | '\x15' | '\x0e' | '\x02' => {
				multiple_move_rogue(ch as i64, &mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'e' => {
				eat(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'q' => {
				quaff(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'r' => {
				read_scroll(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'm' => {
				move_onto(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'd' => {
				drop_0(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'P' => {
				put_on_ring(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'R' => {
				remove_ring(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'\x10' => {
				remessage();
			}
			'\x17' => {
				wizardize(&mut game.player);
			}
			'>' => {
				if drop_check(&game.player, &game.level) {
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
			')' | ']' => {
				inv_armor_weapon(ch == ')', &mut game.player);
			}
			'=' => {
				inv_rings(&game.player);
			}
			'^' => {
				id_trap(&game.player, &game.level);
			}
			'I' => {
				single_inv(None, &mut game.player);
			}
			'T' => {
				take_off(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'W' => {
				wear(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'w' => {
				wield(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'c' => {
				call_it(&mut game.player);
			}
			'z' => {
				zapp(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			't' => {
				throw(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			'v' => {
				message("rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims", 0);
			}
			'Q' => {
				if ask_quit(false, &mut game.player) {
					return PlayResult::ExitQuit;
				};
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
				if wizard {
					inventory(AllObjects, &game.player);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'\x13' => {
				if wizard {
					draw_magic_map(&mut game.mash, &mut game.level);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'\x14' => {
				if wizard {
					show_traps(&game.level);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'\x0f' => {
				if wizard {
					show_objects(&mut game.mash, &game.player, &game.level, &mut game.ground);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'\x01' => {
				show_average_hp(&game.player);
			}
			'\x03' => {
				if wizard {
					new_object_for_wizard(&mut game.player);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'\x0d' => {
				if wizard {
					show_monsters(&mut game.mash, &game.player, &mut game.level);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'S' => {
				if save_game(game) {
					return PlayResult::ExitSaved;
				}
			}
			',' => {
				kick_into_pack(&mut game.mash, &mut game.player, &mut game.level, &mut game.ground);
			}
			_ => {
				message(UNKNOWN_COMMAND, 0);
			}
		}
	}
}
