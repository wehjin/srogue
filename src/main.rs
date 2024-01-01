#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#![feature(extern_types)]

extern "C" {}

use std::sync::OnceLock;
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
use crate::prelude::stat_const::STAT_ALL;

pub mod odds;

pub struct User {
	pub saved_uid: uid_t,
	pub true_uid: uid_t,
}

pub fn user() -> &'static User {
	static USER: OnceLock<User> = OnceLock::new();
	USER.get_or_init(|| unsafe {
		User { saved_uid: geteuid(), true_uid: getuid() }
	})
}

#[no_mangle]
pub unsafe extern "C" fn turn_into_games() {
	if setuid(user().saved_uid) == -1 {
		perror(b"setuid(restore)\0" as *const u8 as *const libc::c_char);
		clean_up("");
	}
}

#[no_mangle]
pub unsafe extern "C" fn turn_into_user() {
	if setuid(user().true_uid) == -1 {
		perror(b"setuid(restore)\0" as *const u8 as *const libc::c_char);
		clean_up("");
	}
}

pub mod console;
pub mod settings;

pub fn main() {
	unsafe {
		setuid(user().true_uid);

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
			print_stats(STAT_ALL);
			level_ready = true;
		}
	};
}
