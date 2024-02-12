#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use libc::{c_int};
use ncurses::{mv, refresh};
use crate::hit::{fight, HIT_MESSAGE};
use crate::init::GameState;
use crate::instruct::Instructions;
use crate::inventory::{inv_armor_weapon, inventory, single_inv};
use crate::level::{check_up, drop_check, show_average_hp};
use crate::message::{CANCEL, check_message, message, remessage, rgetchar};
use crate::monster::show_monsters;
use crate::objects::{level_objects, new_object_for_wizard, show_objects};
use crate::pack::{call_it, drop_0, kick_into_pack, take_off, wear, wield};
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::r#move::{move_onto, multiple_move_rogue, one_move_rogue, rest};
use crate::r#use::{eat, quaff, read_scroll};
use crate::ring::{inv_rings, put_on_ring, remove_ring};
use crate::room::draw_magic_map;
use crate::save::save_game;
use crate::score::quit;
use crate::throw::throw;
use crate::trap::{id_trap, search, show_traps, trap_door};
use crate::zap::{wizard, wizardize, zapp};


pub static mut interrupted: bool = false;

pub const UNKNOWN_COMMAND: &'static str = "unknown command";

pub unsafe fn play_level(game: &mut GameState) {
	let mut count = 0;
	let mut deck_ch = None;
	loop {
		let ch = if let Some(deck_ch) = deck_ch {
			deck_ch
		} else {
			interrupted = false;
			if !HIT_MESSAGE.is_empty() {
				message(&HIT_MESSAGE, 1);
				HIT_MESSAGE.clear();
			}
			if trap_door {
				trap_door = false;
				return;
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
				rest(if count > 0 { count } else { 1 } as c_int, &mut game.player, &mut game.level);
			}
			's' => {
				search(if count > 0 { count } else { 1 } as usize, false, &mut game.player, &mut game.level);
			}
			'i' => {
				inventory(&mut game.player.rogue.pack, AllObjects);
			}
			'f' => {
				fight(false, &mut game.player, &mut game.level);
			}
			'F' => {
				fight(true, &mut game.player, &mut game.level);
			}
			'h' | 'j' | 'k' | 'l' | 'y' | 'u' | 'n' | 'b' => {
				one_move_rogue(ch, true, &mut game.player, &mut game.level);
			}
			'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' | '\x08' | '\x0a' | '\x0b' | '\x0c' | '\x19' | '\x15' | '\x0e' | '\x02' => {
				multiple_move_rogue(ch as i64, &mut game.player, &mut game.level);
			}
			'e' => {
				eat(&mut game.player, &mut game.level);
			}
			'q' => {
				quaff(&mut game.player, &mut game.level);
			}
			'r' => {
				read_scroll(&mut game.player, &mut game.level);
			}
			'm' => {
				move_onto(&mut game.player, &mut game.level);
			}
			'd' => {
				drop_0(&mut game.player, &mut game.level);
			}
			'P' => {
				put_on_ring(&mut game.player, &mut game.level);
			}
			'R' => {
				remove_ring(&mut game.player, &mut game.level);
			}
			'\x10' => {
				remessage();
			}
			'\x17' => {
				wizardize();
			}
			'>' => {
				if drop_check(&game.player, &game.level) {
					return;
				}
			}
			'<' => {
				if check_up(game) {
					return;
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
				take_off(&mut game.player, &mut game.level);
			}
			'W' => {
				wear(&mut game.player, &mut game.level);
			}
			'w' => {
				wield(&mut game.player, &mut game.level);
			}
			'c' => {
				call_it(&game.player);
			}
			'z' => {
				zapp(&mut game.player, &mut game.level);
			}
			't' => {
				throw(&mut game.player, &mut game.level);
			}
			'v' => {
				message("rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims", 0);
			}
			'Q' => {
				quit(false, &mut game.player);
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
					inventory(&mut level_objects, AllObjects);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'\x13' => {
				if wizard {
					draw_magic_map(&mut game.level);
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
					show_objects(&game.player, &game.level);
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
					show_monsters(&game.player, &mut game.level);
				} else {
					message(UNKNOWN_COMMAND, 0);
				}
			}
			'S' => {
				save_game(game);
			}
			',' => {
				kick_into_pack(&mut game.player, &mut game.level);
			}
			_ => {
				message(UNKNOWN_COMMAND, 0);
			}
		}
	}
}
