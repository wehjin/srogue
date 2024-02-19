use std::io;
use std::io::Write;

use libc::c_short;

use crate::armors::constants::RINGMAIL;
use crate::console;
use crate::console::{Console, ConsoleError};
use crate::init::InitError::NoConsole;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::machdep::md_heed_signals;
use crate::monster::MonsterMash;
use crate::objects::{alloc_object, get_food, ObjectPack};
use crate::pack::{do_wear, do_wield};
use crate::player::Player;
use crate::prelude::DungeonSpot;
use crate::prelude::object_what::ObjectWhat::{Armor, Weapon};
use crate::random::get_rand;
use crate::render_system::RenderAction;
use crate::resources::dialog::PlayerDialog;
use crate::resources::healer::Healer;
use crate::ring::ring_stats;
use crate::room::RoomBounds;
use crate::save::restore;
use crate::score::put_scores;
use crate::settings::Settings;
use crate::weapons::constants::{ARROW, BOW, MACE};

//pub static mut save_is_interactive: bool = true;
//pub const ERROR_FILE: &'static str = "player.rogue.esave";
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

pub fn init(settings: Settings) -> Result<InitResult, InitError> {
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

	let mut game = GameState {
		healer: Healer::default(),
		dialog: PlayerDialog::default(),
		player: Player::new(settings.clone()),
		level: Level::new(),
		mash: MonsterMash::default(),
		ground: ObjectPack::new(),
		next_system: GameSystem::PlayerActions,
		render_queue: Vec::new(),
	};
	if settings.score_only {
		put_scores(None, &mut game);
		return Ok(InitResult::ScoreOnly(Player::new(settings.clone()), console, settings));
	}
	if let Some(rest_file) = game.player.settings.rest_file.clone() {
		return if restore(&rest_file, &mut game) {
			Ok(InitResult::Restored(game, console))
		} else {
			Err(InitError::BadRestore(game.player.cleaned_up.clone()))
		};
	}
	game.player.notes.assign_dynamic_titles();
	player_init(&mut game.player);
	ring_stats(false, &mut game);
	return Ok(InitResult::Initialized(game, console));
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameSystem {
	PlayerActions,
	MonsterActions,
}

pub struct GameState {
	pub healer: Healer,
	pub dialog: PlayerDialog,
	pub player: Player,
	pub level: Level,
	pub mash: MonsterMash,
	pub ground: ObjectPack,
	pub next_system: GameSystem,
	pub render_queue: Vec<RenderAction>,
}

impl GameState {
	pub fn dungeon_bounds(&self) -> RoomBounds {
		RoomBounds { top: 1, right: DCOLS as i64 - 1, bottom: DROWS as i64 - 2, left: 0 }
	}
	pub fn render<T: AsRef<[RenderAction]>>(&mut self, actions: T) {
		for action in actions.as_ref() {
			self.render_queue.push(*action)
		}
	}
	pub fn render_spot(&mut self, spot: DungeonSpot) {
		self.render(&[RenderAction::Spot(spot)])
	}
}

impl GameState {
	pub fn start_player_actions(&mut self) {
		self.next_system = GameSystem::PlayerActions;
	}
	pub fn commit_player_turn(&mut self) {
		self.next_system = GameSystem::MonsterActions;
	}
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

pub fn clean_up(estr: &str, player: &mut Player) {
	player.interrupted = true;
	player.cleaned_up = Some(estr.to_string());
}

// pub fn byebye(_ask_quit: bool, _player: &mut Player) {
// 	unimplemented!("bye bye");
// 	// md_ignore_signals();
// 	// if ask_quit {
// 	// 	ask_quit(true, player);
// 	// } else {
// 	// 	clean_up(BYEBYE_STRING);
// 	// }
// 	// md_heed_signals();
// }

pub fn onintr() {
	// TODO Will need to restructure this code to use message passing to interrupt the Player and clear the PlayerDialog.
	// md_ignore_signals();
	// if cant_int {
	// 	did_int = true;
	// } else {
	//  game.diary.clear_message();
	//  game.player.interrupt_and_slurp();
	//  game.diary.message("interrupt", 1);
	// }
	// md_heed_signals();
}

// pub fn error_save() {
// 	save_is_interactive = false;
// 	save_into_file(error_file);
// 	clean_up("");
// }