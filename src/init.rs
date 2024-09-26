use std::io;
use std::io::Write;

use crate::console;
use crate::console::{Console, ConsoleError};
use crate::init::InitError::NoConsole;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::machdep::md_heed_signals;
use crate::monster::{Monster, MonsterMash};
use crate::motion::can_move;
use crate::objects::ObjectPack;
use crate::player::{Avatar, Player, RogueHealth};
use crate::prelude::DungeonSpot;
use crate::random::get_rand;
use crate::render_system::RenderAction;
use crate::resources::diary::Diary;
use crate::resources::healer::Healer;
use crate::resources::rogue::fighter::Fighter;
use crate::ring::effects::RingEffects;
use crate::ring::ring_stats;
use crate::room::RoomBounds;
use crate::save::restore;
use crate::score::put_scores;
use crate::settings::Settings;
use rand::thread_rng;

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
		diary: Diary::default(),
		healer: Healer::default(),
		player: Player::new(settings.clone()),
		level: Level::new(),
		mash: MonsterMash::default(),
		ground: ObjectPack::new(),
		turn: GameTurn::Player,
		render_queue: Vec::new(),
		stats_changed: true,
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
pub enum GameTurn {
	Player,
	Monsters,
}

pub struct GameState {
	pub diary: Diary,
	pub healer: Healer,
	pub player: Player,
	pub level: Level,
	pub mash: MonsterMash,
	pub ground: ObjectPack,
	pub turn: GameTurn,
	pub render_queue: Vec<RenderAction>,
	pub stats_changed: bool,
}

pub trait Dungeon: Avatar {
	fn rogue_can_move(&self, row: i64, col: i64) -> bool;
	fn has_monster(&self, row: i64, col: i64) -> bool;
	fn as_monster_mut(&mut self, mon_id: u64) -> &mut Monster;
	fn interrupt_and_slurp(&mut self);
	fn as_diary_mut(&mut self) -> &mut Diary;
	fn as_fighter(&self) -> &Fighter;
	fn is_max_depth(&self) -> bool;
	fn m_moves(&self) -> usize;
	fn m_moves_mut(&mut self) -> &mut usize;
	fn as_ring_effects(&self) -> &RingEffects;
	fn is_any_door_at(&self, row: i64, col: i64) -> bool;
	fn is_any_tunnel_at(&self, row: i64, col: i64) -> bool;
}

impl Dungeon for GameState {
	fn rogue_can_move(&self, row: i64, col: i64) -> bool { can_move(self.rogue_row(), self.rogue_col(), row, col, &self.level) }
	fn has_monster(&self, row: i64, col: i64) -> bool { self.level.dungeon[row as usize][col as usize].has_monster() }
	fn as_monster_mut(&mut self, mon_id: u64) -> &mut Monster { self.mash.monster_mut(mon_id) }
	fn interrupt_and_slurp(&mut self) { self.player.interrupt_and_slurp(); }
	fn as_diary_mut(&mut self) -> &mut Diary { &mut self.diary }
	fn as_fighter(&self) -> &Fighter { &self.player.rogue }
	fn is_max_depth(&self) -> bool { self.player.cur_depth >= self.player.max_depth }
	fn m_moves(&self) -> usize { self.mash.m_moves }
	fn m_moves_mut(&mut self) -> &mut usize { &mut self.mash.m_moves }
	fn as_ring_effects(&self) -> &RingEffects { &self.player.ring_effects }
	fn is_any_door_at(&self, row: i64, col: i64) -> bool { self.level.cell(DungeonSpot { row, col }).is_any_door() }
	fn is_any_tunnel_at(&self, row: i64, col: i64) -> bool { self.level.cell(DungeonSpot { row, col }).is_any_tunnel() }
}

impl Avatar for GameState {
	fn as_health(&self) -> &RogueHealth { self.player.as_health() }
	fn as_health_mut(&mut self) -> &mut RogueHealth { self.player.as_health_mut() }
	fn rogue_row(&self) -> i64 { self.player.rogue_row() }
	fn rogue_col(&self) -> i64 { self.player.rogue_col() }
	fn set_rogue_row_col(&mut self, row: i64, col: i64) { self.player.set_rogue_row_col(row, col) }
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
		self.turn = GameTurn::Player;
	}
	pub fn yield_turn_to_monsters(&mut self) {
		self.turn = GameTurn::Monsters;
	}
}

fn player_init(player: &mut Player) {
	player.rogue.pack.clear();
	let rng = &mut thread_rng();
	player.rogue.provision(rng);
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

// pub fn onintr() {
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
// }

// pub fn error_save() {
// 	save_is_interactive = false;
// 	save_into_file(error_file);
// 	clean_up("");
// }