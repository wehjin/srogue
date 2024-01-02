#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::{io};
use std::io::Write;
use libc::c_short;
use settings::nick_name;
use crate::{console, settings};
use crate::prelude::*;
use crate::prelude::armor_kind::RINGMAIL;
use crate::prelude::object_what::ObjectWhat::{Armor, Weapon};
use crate::prelude::weapon_kind::{ARROW, BOW, MACE};
use crate::settings::{rest_file, score_only};

pub static mut cant_int: bool = false;
pub static mut did_int: bool = false;
pub static mut save_is_interactive: bool = true;
pub static mut error_file: &'static str = "rogue.esave";
pub static BYEBYE_STRING: &'static str = "Okay, bye bye!";

pub struct GameState {
	seed: [u8; 32],
}

impl GameState {
	pub fn new() -> Self {
		GameState {
			seed: [1u8; 32],
		}
	}

	pub fn set_seed(&mut self, seed: u32) {
		let bytes = {
			let mut parts: [u8; 4] = [0; 4];
			for i in 0..4 {
				parts[i] = (seed >> (i * 8)) as u8;
			}
			parts
		};
		for i in 0..self.seed.len() {
			self.seed[i] = bytes[i % bytes.len()]
		}
	}
}

pub unsafe fn init() -> bool {
	match get_login_name() {
		None => {
			clean_up("Hey!  Who are you?");
		}
		Some(name) => {
			settings::set_login_name(&name);
		}
	}
	if !score_only() && rest_file().is_none() {
		print!("Hello {}, just a moment while I dig the dungeon...", match nick_name() {
			None => settings::login_name(),
			Some(name) => name,
		});
		io::stdout().flush().expect("flush stdout");
	}

	ncurses::initscr();
	if ncurses::LINES() < 24 || ncurses::COLS() < 80 {
		clean_up("must be played on 24 x 80 or better screen");
	}
	console::up();

	let mut game = GameState::new();
	md_heed_signals();

	if score_only() {
		put_scores(None);
	}
	game.set_seed(md_get_seed());
	if let Some(rest_file) = rest_file() {
		restore(rest_file);
		return true;
	}
	mix_colors();
	get_wand_and_ring_materials();
	make_scroll_titles();
	level_objects.next_object = 0 as *mut obj;
	level_monsters.next_object = 0 as *mut obj;
	player_init();
	party_counter = get_rand(1, 10);
	ring_stats(false);
	return false;
}

unsafe fn player_init() {
	rogue.pack.next_object = 0 as *mut obj;

	let mut obj = alloc_object();
	get_food(&mut *obj, true);
	add_to_pack(obj, &mut rogue.pack, 1);

	let obj = alloc_object();           /* initial armor */
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = Armor;
		obj.which_kind = RINGMAIL;
		obj.class = RINGMAIL as isize + 2;
		obj.is_protected = 0;
		obj.d_enchant = 1;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
	do_wear(&mut *obj);

	let obj = alloc_object();           /* initial weapons */
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = Weapon;
		obj.which_kind = MACE;
		obj.damage = "2d3".to_string();
		obj.hit_enchant = 1;
		obj.d_enchant = 1;
		obj.identified = true;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
	do_wield(&mut *obj);

	let obj = alloc_object();
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = Weapon;
		obj.which_kind = BOW;
		obj.damage = "1d2".to_string();
		obj.hit_enchant = 1;
		obj.d_enchant = 0;
		obj.identified = true;
	}
	add_to_pack(obj, &mut rogue.pack, 1);

	let obj = alloc_object();
	{
		let obj: &mut obj = &mut *obj;
		obj.what_is = Weapon;
		obj.which_kind = ARROW;
		obj.quantity = get_rand(25, 35) as c_short;
		obj.damage = "1d2".to_string();
		obj.hit_enchant = 0;
		obj.d_enchant = 0;
		obj.identified = true;
	}
	add_to_pack(obj, &mut rogue.pack, 1);
}

pub unsafe fn clean_up(estr: &str) {
	if save_is_interactive {
		if console::is_up() {
			ncurses::wmove(ncurses::stdscr(), (DROWS - 1) as i32, 0);
			ncurses::refresh();
			console::down();
		}
		print!("\n{}\n", estr);
	}
	ncurses::endwin();
	md_exit(0);
}


pub unsafe fn byebye(ask_quit: bool) {
	md_ignore_signals();
	if ask_quit {
		quit(true);
	} else {
		clean_up(BYEBYE_STRING);
	}
	md_heed_signals();
}

#[no_mangle]
pub unsafe extern "C" fn onintr() -> i64 {
	md_ignore_signals();
	if cant_int {
		did_int = true;
	} else {
		check_message();
		message("interrupt", 1);
	}
	md_heed_signals();
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn error_save() {
	save_is_interactive = false;
	save_into_file(error_file);
	clean_up("");
}
