#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn is_digit() -> libc::c_char;
	static mut trap_door: libc::c_char;
}

use libc::c_short;
use crate::prelude::*;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: libc::c_short,
	pub _pad_x: libc::c_short,
	pub _pad_top: libc::c_short,
	pub _pad_left: libc::c_short,
	pub _pad_bottom: libc::c_short,
	pub _pad_right: libc::c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = ncurses::chtype;


#[no_mangle]
pub static mut interrupted: bool = false;
#[no_mangle]
pub static mut unknown_command: *mut libc::c_char = b"unknown command\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn play_level() {
	let mut ch: libc::c_short = 0;
	let mut count: i64 = 0;
	loop {
		interrupted = false;
		if !hit_message.is_empty() {
			message(&hit_message, 1);
			hit_message.clear();
		}
		if trap_door != 0 {
			trap_door = 0;
			return;
		}
		ncurses::wmove(ncurses::stdscr(), rogue.row as i64, rogue.col as i64);
		ncurses::refresh();
		ch = rgetchar() as libc::c_short;
		check_message();
		count = 0 as i64;
		loop {
			match ch as i64 {
				63 => {
					Instructions();
					break;
				}
				46 => {
					rest(
						if count > 0 as i64 { count } else { 1 },
					);
					break;
				}
				115 => {
					search(if count > 0 { count } else { 1 } as c_short, 0);
					break;
				}
				105 => {
					inventory(
						&mut rogue.pack,
						0o777 as i64 as libc::c_ushort as i64,
					);
					break;
				}
				102 => {
					fight(0 as i64);
					break;
				}
				70 => {
					fight(1);
					break;
				}
				104 | 106 | 107 | 108 | 121 | 117 | 110 | 98 => {
					one_move_rogue(ch as i64, 1);
					break;
				}
				72 | 74 | 75 | 76 | 66 | 89 | 85 | 78 | 8 | 10 | 11 | 12 | 25 | 21 | 14
				| 2 => {
					multiple_move_rogue(ch as i64);
					break;
				}
				101 => {
					eat();
					break;
				}
				113 => {
					quaff();
					break;
				}
				114 => {
					read_scroll();
					break;
				}
				109 => {
					move_onto();
					break;
				}
				100 => {
					drop_0();
					break;
				}
				80 => {
					put_on_ring();
					break;
				}
				82 => {
					remove_ring();
					break;
				}
				16 => {
					remessage();
					break;
				}
				23 => {
					wizardize();
					break;
				}
				62 => {
					if drop_check() != 0 {
						return;
					}
					break;
				}
				60 => {
					if check_up() != 0 {
						return;
					}
					break;
				}
				41 | 93 => {
					inv_armor_weapon((ch as i64 == ')' as i32) as i64);
					break;
				}
				61 => {
					inv_rings();
					break;
				}
				94 => {
					id_trap();
					break;
				}
				73 => {
					single_inv(0 as i64);
					break;
				}
				84 => {
					take_off();
					break;
				}
				87 => {
					wear();
					break;
				}
				119 => {
					wield();
					break;
				}
				99 => {
					call_it();
					break;
				}
				122 => {
					zapp();
					break;
				}
				116 => {
					throw();
					break;
				}
				118 => {
					message(
						b"rogue-clone: Version II. (Tim Stoehr was here), tektronix!zeus!tims\0"
							as *const u8 as *const libc::c_char,
						0 as i64,
					);
					break;
				}
				81 => {
					quit(0 as i64);
				}
				48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {}
				32 => {
					break;
				}
				9 => {
					if wizard != 0 {
						inventory(
							&mut level_objects,
							0o777 as i64 as libc::c_ushort as i64,
						);
					} else {
						message(unknown_command, 0 as i64);
					}
					break;
				}
				19 => {
					if wizard != 0 {
						draw_magic_map();
					} else {
						message(unknown_command, 0 as i64);
					}
					break;
				}
				20 => {
					if wizard != 0 {
						show_traps();
					} else {
						message(unknown_command, 0 as i64);
					}
					break;
				}
				15 => {
					if wizard != 0 {
						show_objects();
					} else {
						message(unknown_command, 0 as i64);
					}
					break;
				}
				1 => {
					show_average_hp();
					break;
				}
				3 => {
					if wizard != 0 {
						new_object_for_wizard();
					} else {
						message(unknown_command, 0 as i64);
					}
					break;
				}
				13 => {
					if wizard != 0 {
						show_monsters();
					} else {
						message(unknown_command, 0 as i64);
					}
					break;
				}
				83 => {
					save_game();
					break;
				}
				44 => {
					kick_into_pack();
					break;
				}
				_ => {
					message(unknown_command, 0 as i64);
					break;
				}
			}
			ncurses::wmove(ncurses::stdscr(), rogue.row as i64, rogue.col as i64);
			ncurses::refresh();
			loop {
				if count < 100 as i64 {
					count = 10 as i64 * count + (ch as i64 - '0' as i32);
				}
				ch = rgetchar() as libc::c_short;
				if !(is_digit(ch as i64) != 0) {
					break;
				}
			}
			if !(ch as i64 != '\u{1b}' as i32) {
				break;
			}
		}
	};
}
