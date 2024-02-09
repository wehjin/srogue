#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_int};
use ncurses::{mv, refresh};
use crate::prelude::*;
use crate::prelude::object_what::PackFilter::AllObjects;


pub static mut interrupted: bool = false;

pub static unknown_command: &'static str = "unknown command";


pub unsafe fn play_level(game: &mut GameState) {
	let mut count = 0;
	let mut deck_ch = None;
	loop {
		let ch = if let Some(deck_ch) = deck_ch {
			deck_ch
		} else {
			interrupted = false;
			if !hit_message.is_empty() {
				message(&hit_message, 1);
				hit_message.clear();
			}
			if trap_door {
				trap_door = false;
				return;
			}
			mv(rogue.row as i32, rogue.col as i32);
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
				rest(if count > 0 { count } else { 1 } as c_int, &game.depth, &mut game.level);
			}
			's' => {
				search(if count > 0 { count } else { 1 } as usize, false, &game.depth, &mut game.level);
			}
			'i' => {
				inventory(&mut rogue.pack, AllObjects);
			}
			'f' => {
				fight(false, &game.depth, &mut game.level);
			}
			'F' => {
				fight(true, &game.depth, &mut game.level);
			}
			'h' | 'j' | 'k' | 'l' | 'y' | 'u' | 'n' | 'b' => {
				one_move_rogue(ch, true, &game.depth, &mut game.level);
			}
			'H' | 'J' | 'K' | 'L' | 'B' | 'Y' | 'U' | 'N' | '\x08' | '\x0a' | '\x0b' | '\x0c' | '\x19' | '\x15' | '\x0e' | '\x02' => {
				multiple_move_rogue(ch as i64, &game.depth, &mut game.level);
			}
			'e' => {
				eat(&game.depth, &mut game.level);
			}
			'q' => {
				quaff(&game.depth, &mut game.level);
			}
			'r' => {
				read_scroll(&game.depth, &mut game.level);
			}
			'm' => {
				move_onto(&game.depth, &mut game.level);
			}
			'd' => {
				drop_0(&game.depth, &mut game.level);
			}
			'P' => {
				put_on_ring(&game.depth, &mut game.level);
			}
			'R' => {
				remove_ring(&game.depth, &mut game.level);
			}
			'\x10' => {
				remessage();
			}
			'\x17' => {
				wizardize();
			}
			'>' => {
				if drop_check(&game.level) {
					return;
				}
			}
			'<' => {
				if check_up(game) {
					return;
				}
			}
			')' | ']' => {
				inv_armor_weapon(ch == ')');
			}
			'=' => {
				inv_rings();
			}
			'^' => {
				id_trap(&game.level);
			}
			'I' => {
				single_inv(None);
			}
			'T' => {
				take_off(&game.depth, &mut game.level);
			}
			'W' => {
				wear(&game.depth, &mut game.level);
			}
			'w' => {
				wield(&game.depth, &mut game.level);
			}
			'c' => {
				call_it();
			}
			'z' => {
				zapp(&game.depth, &mut game.level);
			}
			't' => {
				throw(&game.depth, &mut game.level);
			}
			'v' => {
				message("rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims", 0);
			}
			'Q' => {
				quit(false, game.depth.max);
			}
			'0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
				mv(rogue.row as i32, rogue.col as i32);
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
					message(unknown_command, 0);
				}
			}
			'\x13' => {
				if wizard {
					draw_magic_map(&mut game.level);
				} else {
					message(unknown_command, 0);
				}
			}
			'\x14' => {
				if wizard {
					show_traps(&game.level);
				} else {
					message(unknown_command, 0);
				}
			}
			'\x0f' => {
				if wizard {
					show_objects(&game.level);
				} else {
					message(unknown_command, 0);
				}
			}
			'\x01' => {
				show_average_hp();
			}
			'\x03' => {
				if wizard {
					new_object_for_wizard();
				} else {
					message(unknown_command, 0);
				}
			}
			'\x0d' => {
				if wizard {
					show_monsters();
				} else {
					message(unknown_command, 0);
				}
			}
			'S' => {
				save_game(game);
			}
			',' => {
				kick_into_pack(&game.depth, &mut game.level);
			}
			_ => {
				message(unknown_command, 0);
			}
		}
	}
}
