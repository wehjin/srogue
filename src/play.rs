#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	fn wrefresh(_: *mut WINDOW) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	static mut rogue: fighter;
	static mut level_objects: object;
	fn is_digit() -> libc::c_char;
	static mut hit_message: [libc::c_char; 0];
	static mut wizard: libc::c_char;
	static mut trap_door: libc::c_char;
}

use crate::prelude::*;

pub type chtype = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: libc::c_short,
	pub _curx: libc::c_short,
	pub _maxy: libc::c_short,
	pub _maxx: libc::c_short,
	pub _begy: libc::c_short,
	pub _begx: libc::c_short,
	pub _flags: libc::c_short,
	pub _attrs: attr_t,
	pub _bkgd: chtype,
	pub _notimeout: libc::c_int,
	pub _clear: libc::c_int,
	pub _leaveok: libc::c_int,
	pub _scroll: libc::c_int,
	pub _idlok: libc::c_int,
	pub _idcok: libc::c_int,
	pub _immed: libc::c_int,
	pub _sync: libc::c_int,
	pub _use_keypad: libc::c_int,
	pub _delay: libc::c_int,
	pub _line: *mut ldat,
	pub _regtop: libc::c_short,
	pub _regbottom: libc::c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: libc::c_short,
}

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
pub type attr_t = chtype;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: libc::c_short,
	pub hp_max: libc::c_short,
	pub str_current: libc::c_short,
	pub str_max: libc::c_short,
	pub pack: object,
	pub gold: libc::c_long,
	pub exp: libc::c_short,
	pub exp_points: libc::c_long,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub fchar: libc::c_short,
	pub moves_left: libc::c_short,
}

pub type fighter = fight;

#[no_mangle]
pub static mut interrupted: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut unknown_command: *mut libc::c_char = b"unknown command\0" as *const u8
	as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn play_level() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut count: libc::c_int = 0;
	loop {
		interrupted = 0 as libc::c_int as libc::c_char;
		if *hit_message.as_mut_ptr().offset(0 as libc::c_int as isize) != 0 {
			message(hit_message.as_mut_ptr(), 1 as libc::c_int);
			*hit_message
				.as_mut_ptr()
				.offset(0 as libc::c_int as isize) = 0 as libc::c_int as libc::c_char;
		}
		if trap_door != 0 {
			trap_door = 0 as libc::c_int as libc::c_char;
			return;
		}
		wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int);
		wrefresh(stdscr);
		ch = rgetchar() as libc::c_short;
		check_message();
		count = 0 as libc::c_int;
		loop {
			match ch as libc::c_int {
				63 => {
					Instructions();
					break;
				}
				46 => {
					rest(
						if count > 0 as libc::c_int { count } else { 1 as libc::c_int },
					);
					break;
				}
				115 => {
					search(
						if count > 0 as libc::c_int { count } else { 1 as libc::c_int },
						0 as libc::c_int,
					);
					break;
				}
				105 => {
					inventory(
						&mut rogue.pack,
						0o777 as libc::c_int as libc::c_ushort as libc::c_int,
					);
					break;
				}
				102 => {
					fight(0 as libc::c_int);
					break;
				}
				70 => {
					fight(1 as libc::c_int);
					break;
				}
				104 | 106 | 107 | 108 | 121 | 117 | 110 | 98 => {
					one_move_rogue(ch as libc::c_int, 1 as libc::c_int);
					break;
				}
				72 | 74 | 75 | 76 | 66 | 89 | 85 | 78 | 8 | 10 | 11 | 12 | 25 | 21 | 14
				| 2 => {
					multiple_move_rogue(ch as libc::c_int);
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
					inv_armor_weapon((ch as libc::c_int == ')' as i32) as libc::c_int);
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
					single_inv(0 as libc::c_int);
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
						0 as libc::c_int,
					);
					break;
				}
				81 => {
					quit(0 as libc::c_int);
				}
				48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {}
				32 => {
					break;
				}
				9 => {
					if wizard != 0 {
						inventory(
							&mut level_objects,
							0o777 as libc::c_int as libc::c_ushort as libc::c_int,
						);
					} else {
						message(unknown_command, 0 as libc::c_int);
					}
					break;
				}
				19 => {
					if wizard != 0 {
						draw_magic_map();
					} else {
						message(unknown_command, 0 as libc::c_int);
					}
					break;
				}
				20 => {
					if wizard != 0 {
						show_traps();
					} else {
						message(unknown_command, 0 as libc::c_int);
					}
					break;
				}
				15 => {
					if wizard != 0 {
						show_objects();
					} else {
						message(unknown_command, 0 as libc::c_int);
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
						message(unknown_command, 0 as libc::c_int);
					}
					break;
				}
				13 => {
					if wizard != 0 {
						show_monsters();
					} else {
						message(unknown_command, 0 as libc::c_int);
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
					message(unknown_command, 0 as libc::c_int);
					break;
				}
			}
			wmove(stdscr, rogue.row as libc::c_int, rogue.col as libc::c_int);
			wrefresh(stdscr);
			loop {
				if count < 100 as libc::c_int {
					count = 10 as libc::c_int * count + (ch as libc::c_int - '0' as i32);
				}
				ch = rgetchar() as libc::c_short;
				if !(is_digit(ch as libc::c_int) != 0) {
					break;
				}
			}
			if !(ch as libc::c_int != '\u{1b}' as i32) {
				break;
			}
		}
	};
}
