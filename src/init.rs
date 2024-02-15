#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::io;
use std::io::Write;

use libc::c_short;

use crate::armors::constants::RINGMAIL;
use crate::console;
use crate::console::{Console, ConsoleError};
use crate::init::InitError::NoConsole;
use crate::level::Level;
use crate::machdep::{md_heed_signals, md_ignore_signals};
use crate::message::{check_message, message};
use crate::monster::MonsterMash;
use crate::objects::{alloc_object, get_food, LEVEL_OBJECTS};
use crate::pack::{do_wear, do_wield};
use crate::player::Player;
use crate::prelude::object_what::ObjectWhat::{Armor, Weapon};
use crate::random::get_rand;
use crate::ring::ring_stats;
use crate::save::restore;
use crate::score::put_scores;
use crate::settings::Settings;
use crate::weapons::constants::{ARROW, BOW, MACE};

pub static mut cant_int: bool = false;
pub static mut did_int: bool = false;
pub static mut save_is_interactive: bool = true;
pub const ERROR_FILE: &'static str = "player.rogue.esave";
pub const BYEBYE_STRING: &'static str = "Okay, bye bye!";

pub enum InitError {
	NoConsole(ConsoleError),
	BadRestore(Option<String>),
}

pub enum InitResult {
	ScoreOnly(Player, Console, Settings),
	Restored(GameState, Console),
	Initialized(GameState, Console),
}

pub unsafe fn init(settings: Settings) -> Result<InitResult, InitError> {
	if !settings.score_only && settings.rest_file.is_none() {
		print!("Hello {}, just a moment while I dig the dungeon...", settings.player_name());
		io::stdout().flush().expect("flush stdout");
	}
	let console = match console::start() {
		Ok(console) => console,
		Err(error) => {
			return Err(NoConsole(error));
		}
	};
	md_heed_signals();
	if settings.score_only {
		let mut player = Player::new(settings.clone());
		put_scores(None, &mut player);
		return Ok(InitResult::ScoreOnly(Player::new(settings.clone()), console, settings));
	}

	let mut game = GameState {
		player: Player::new(settings),
		level: Level::new(),
		mash: MonsterMash::new(),
	};
	if let Some(rest_file) = game.player.settings.rest_file.clone() {
		return if restore(&rest_file, &mut game) {
			Ok(InitResult::Restored(game, console))
		} else {
			Err(InitError::BadRestore(game.player.cleaned_up.clone()))
		};
	}
	game.player.notes.assign_dynamic_titles();
	LEVEL_OBJECTS.clear();
	player_init(&mut game.player);
	ring_stats(false, &mut game.mash, &mut game.player, &mut game.level);
	return Ok(InitResult::Initialized(game, console));
}

pub struct GameState {
	pub player: Player,
	pub level: Level,
	pub mash: MonsterMash,
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

pub unsafe fn clean_up(estr: &str, player: &mut Player) {
	player.cleaned_up = Some(estr.to_string());
}

// pub unsafe fn byebye(_ask_quit: bool, _player: &mut Player) {
// 	unimplemented!("bye bye");
// 	// md_ignore_signals();
// 	// if ask_quit {
// 	// 	ask_quit(true, player);
// 	// } else {
// 	// 	clean_up(BYEBYE_STRING);
// 	// }
// 	// md_heed_signals();
// }

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
