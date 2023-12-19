#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![feature(extern_types)]

extern "C" {
	static mut level_objects: object;
	static mut level_monsters: object;
	static mut party_room: libc::c_short;
}

use std::process;
use crate::prelude::*;

mod message;
mod level;
mod monster;
mod hit;
mod init;
mod instruct;
mod inventory;
mod machdep;
mod r#move;
mod objects;
mod pack;
mod play;
mod random;
mod ring;
mod room;
mod save;
mod score;
mod spec_hit;
mod throw;
mod trap;
mod r#use;
mod zap;

mod prelude;

use libc::{setuid, perror, geteuid, getuid};

pub mod odds;

#[no_mangle]
pub unsafe extern "C" fn turn_into_games(saved_uid: uid_t) {
	if setuid(saved_uid) == -(1 as libc::c_int) {
		perror(b"setuid(restore)\0" as *const u8 as *const libc::c_char);
		clean_up(b"\0" as *const u8 as *const libc::c_char);
	}
}

#[no_mangle]
pub unsafe extern "C" fn turn_into_user(true_uid: uid_t) {
	if setuid(true_uid) == -(1 as libc::c_int) {
		perror(b"setuid(restore)\0" as *const u8 as *const libc::c_char);
		clean_up(b"\0" as *const u8 as *const libc::c_char);
	}
}

pub fn main() {
	unsafe {
		let saved_uid = geteuid();
		let true_uid = getuid();
		setuid(true_uid);

		let mut level_ready = init();
		loop {
			if level_ready {
				play_level();
				free_stuff(&mut level_objects);
				free_stuff(&mut level_monsters);
			}

			clear_level();
			make_level();
			put_objects();
			put_stairs();
			add_traps();
			put_mons();
			put_player(party_room);
			print_stats(0o377 as libc::c_int);
			level_ready = true;
		}
	};
}
