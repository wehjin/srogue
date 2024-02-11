#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::{io};
use std::io::Write;
use libc::{c_short};
use settings::nick_name;
use crate::{console, settings};
use crate::level::constants::DROWS;
use crate::player::Player;
use crate::prelude::*;
use crate::armors::constants::RINGMAIL;
use crate::prelude::object_what::ObjectWhat::{Armor, Weapon};
use crate::settings::{rest_file, score_only};
use crate::weapons::constants::{ARROW, BOW, MACE};

pub static mut cant_int: bool = false;
pub static mut did_int: bool = false;
pub static mut save_is_interactive: bool = true;
pub const ERROR_FILE: &'static str = "player.rogue.esave";
pub const BYEBYE_STRING: &'static str = "Okay, bye bye!";

pub struct GameState {
	seed: [u8; 32],
	pub player: Player,
	pub level: Level,
}

impl GameState {
	pub fn new() -> Self {
		GameState {
			seed: [1u8; 32],
			player: Player::new(),
			level: Level::new(),
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

pub unsafe fn init() -> (GameState, bool) {
	match get_login_name() {
		None => {
			clean_up("Hey!  Who are you?");
		}
		Some(name) => {
			settings::set_login_name(&name);
		}
	}
	if !score_only() && rest_file().is_none() {
		print!("Hello {}, just a moment while I dig the dungeon...", nick_name().unwrap_or_else(|| settings::login_name()));
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
		put_scores(None, &game.player);
	}
	game.set_seed(md_get_seed());
	if let Some(rest_file) = rest_file() {
		restore(&rest_file, &mut game);
		return (game, true);
	}
	mix_colors();
	get_wand_and_ring_materials();
	make_scroll_titles();
	level_objects.clear();
	MASH.clear();
	player_init(&mut game.player);
	ring_stats(false, &mut game.player, &mut game.level);
	return (game, false);
}

fn player_init(player: &mut Player) {
	player.rogue.pack.clear();
	// Food
	{
		let mut obj = alloc_object();
		get_food(&mut obj, true);
		player.combine_or_add_item_to_pack(obj);
	}
	// Armor
	{
		let mut obj = alloc_object();
		obj.what_is = Armor;
		obj.which_kind = RINGMAIL;
		obj.class = RINGMAIL as isize + 2;
		obj.is_protected = 0;
		obj.d_enchant = 1;
		let added = player.combine_or_add_item_to_pack(obj);
		do_wear(added, player);
	}
	// Mace
	{
		let mut obj = alloc_object();
		obj.what_is = Weapon;
		obj.which_kind = MACE;
		obj.hit_enchant = 1;
		obj.d_enchant = 1;
		obj.identified = true;
		let added = player.combine_or_add_item_to_pack(obj);
		do_wield(added, player);
	}
	// Bow
	{
		let mut obj = alloc_object();
		obj.what_is = Weapon;
		obj.which_kind = BOW;
		obj.hit_enchant = 1;
		obj.d_enchant = 0;
		obj.identified = true;
		player.combine_or_add_item_to_pack(obj);
	}
	// Arrows
	{
		let mut obj = alloc_object();
		obj.what_is = Weapon;
		obj.which_kind = ARROW;
		obj.quantity = get_rand(25, 35) as c_short;
		obj.hit_enchant = 0;
		obj.d_enchant = 0;
		obj.identified = true;
		player.combine_or_add_item_to_pack(obj);
	}
	player.party_counter = get_rand(1, 10);
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


pub unsafe fn byebye(ask_quit: bool, player: &mut Player) {
	md_ignore_signals();
	if ask_quit {
		quit(true, player);
	} else {
		clean_up(BYEBYE_STRING);
	}
	md_heed_signals();
}

pub unsafe fn onintr() {
	md_ignore_signals();
	if cant_int {
		did_int = true;
	} else {
		check_message();
		message("interrupt", 1);
	}
	md_heed_signals();
}

pub unsafe fn error_save(game: &GameState) {
	save_is_interactive = false;
	save_into_file(ERROR_FILE, game);
	clean_up("");
}
