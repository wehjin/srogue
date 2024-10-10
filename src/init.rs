use crate::console;
use crate::console::{Console, ConsoleError};
use crate::init::InitError::NoConsole;
use crate::level::constants::{DCOLS, DROWS};
use crate::level::Level;
use crate::machdep::md_heed_signals;
use crate::monster::{aim_monster, Monster, MonsterIndex, MonsterMash};
use crate::motion::{can_move, is_passable};
use crate::objects::{Object, ObjectId, ObjectPack};
use crate::player::Player;
use crate::prelude::DungeonSpot;
use crate::random::get_rand;
use crate::render_system::RenderAction;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::diary::Diary;
use crate::resources::healer::Healer;
use crate::resources::infra::Infra;
use crate::resources::level::size::LevelSpot;
use crate::resources::physics;
use crate::resources::physics::rogue_sees_spot;
use crate::ring::ring_stats;
use crate::room::{dr_course_legacy, gr_spot_without_rogue, RoomBounds};
use crate::save::restore;
use crate::score::put_scores;
use crate::settings::Settings;
use rand::distributions::uniform::SampleUniform;
use rand::{thread_rng, Rng};
use std::io;
use std::io::Write;
use std::ops::RangeInclusive;

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
	};
	if settings.score_only {
		put_scores(None, &mut game);
		return Ok(InitResult::ScoreOnly(Player::new(settings.clone()), console, settings));
	}
	if let Some(rest_file) = game.player.settings.rest_file.clone() {
		return if restore(&rest_file, &mut game) {
			Ok(InitResult::Restored(game, console))
		} else {
			Err(InitError::BadRestore(game.diary.cleaned_up.clone()))
		};
	}
	game.player.notes.assign_dynamic_titles();
	player_init(&mut game.player);
	ring_stats(false, &mut game);
	Ok(InitResult::Initialized(game, console))
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
}

pub trait Dungeon: Avatar + Infra + Arena {
	fn roll_range<T: SampleUniform + PartialOrd>(&mut self, range: RangeInclusive<T>) -> T;
	fn move_mon_to(&mut self, mon_id: u64, row: i64, col: i64);
	fn set_interrupted(&mut self, value: bool);
	fn rogue_can_move(&self, row: i64, col: i64) -> bool;
	fn rogue_can_see(&self, row: i64, col: i64) -> bool { rogue_sees_spot(LevelSpot::from_i64(row, col), self, self, self) }
	fn has_monster_at(&self, row: i64, col: i64) -> bool;
	fn get_monster_at(&self, row: i64, col: i64) -> Option<u64>;
	fn get_monster(&self, mon_id: u64) -> Option<&Monster>;
	fn interrupt_and_slurp(&mut self);
	fn as_diary(&self) -> &Diary;
	fn as_diary_mut(&mut self) -> &mut Diary;
	fn is_max_depth(&self) -> bool;
	fn m_moves(&self) -> usize;
	fn m_moves_mut(&mut self) -> &mut usize;
	fn is_any_door_at(&self, row: i64, col: i64) -> bool;
	fn is_any_tunnel_at(&self, row: i64, col: i64) -> bool;
	fn is_any_trap_at(&self, row: i64, col: i64) -> bool;
	fn is_no_feature_at(&self, row: i64, col: i64) -> bool;
	fn is_passable_at(&self, row: i64, col: i64) -> bool;
	fn has_object_at(&self, row: i64, col: i64) -> bool;
	fn try_object_at(&self, row: i64, col: i64) -> Option<&Object>;
	fn object_ids(&self) -> Vec<ObjectId>;
	fn shows_skull(&self) -> bool;
	fn player_name(&self) -> String;
	fn monster_ids(&self) -> Vec<u64>;
	fn cleaned_up(&self) -> Option<String>;
	fn monster_sees_rogue(&self, mon_id: MonsterIndex) -> bool {
		physics::monster_sees_rogue(mon_id, self, self)
	}
	fn roll_wanderer_spot(&self, rng: &mut impl Rng) -> Option<LevelSpot>;

	fn airdrop_monster_at(&mut self, row: i64, col: i64, monster: Monster);
}

impl GameState {
	pub fn render<T: AsRef<[RenderAction]>>(&mut self, actions: T) {
		for action in actions.as_ref() {
			self.render_queue.push(*action)
		}
	}
}

impl Dungeon for GameState {
	fn roll_range<T: SampleUniform + PartialOrd>(&mut self, range: RangeInclusive<T>) -> T {
		thread_rng().gen_range(range)
	}
	fn move_mon_to(&mut self, mon_id: u64, row: i64, col: i64) {
		let to_spot = DungeonSpot { row, col };
		let from_spot = self.as_monster(mon_id).spot;
		self.cell_at_mut(from_spot).set_monster(false);
		self.cell_at_mut(to_spot).set_monster(true);
		self.render(&[RenderAction::Spot(from_spot), RenderAction::Spot(to_spot)]);
		if self.cell_at(to_spot).is_any_door() {
			let entering = self.cell_at(from_spot).is_any_tunnel();
			dr_course_legacy(mon_id, entering, row, col, self);
		} else {
			let monster = self.as_monster_mut(mon_id);
			monster.spot = to_spot;
		}
	}

	fn set_interrupted(&mut self, value: bool) { self.player.interrupted = value; }
	fn rogue_can_move(&self, row: i64, col: i64) -> bool { can_move(self.rogue_row(), self.rogue_col(), row, col, &self.level) }
	fn has_monster_at(&self, row: i64, col: i64) -> bool { self.level.dungeon[row as usize][col as usize].has_monster() }
	fn get_monster_at(&self, row: i64, col: i64) -> Option<u64> {
		match self.mash.monster_at_spot(row, col) {
			None => None,
			Some(monster) => Some(monster.id())
		}
	}
	fn get_monster(&self, mon_id: u64) -> Option<&Monster> { self.mash.try_monster(mon_id) }
	fn interrupt_and_slurp(&mut self) { self.player.interrupt_and_slurp(); }
	fn as_diary(&self) -> &Diary { &self.diary }
	fn as_diary_mut(&mut self) -> &mut Diary { &mut self.diary }
	fn is_max_depth(&self) -> bool { self.player.cur_depth >= self.player.max_depth }
	fn m_moves(&self) -> usize { self.mash.m_moves }
	fn m_moves_mut(&mut self) -> &mut usize { &mut self.mash.m_moves }
	fn is_any_door_at(&self, row: i64, col: i64) -> bool { self.level.cell(DungeonSpot { row, col }).is_any_door() }
	fn is_any_tunnel_at(&self, row: i64, col: i64) -> bool { self.level.cell(DungeonSpot { row, col }).is_any_tunnel() }
	fn is_any_trap_at(&self, row: i64, col: i64) -> bool { self.level.dungeon[row as usize][col as usize].is_any_trap() }
	fn is_no_feature_at(&self, row: i64, col: i64) -> bool { self.level.dungeon[row as usize][col as usize].is_nothing() }
	fn is_passable_at(&self, row: i64, col: i64) -> bool { is_passable(row, col, &self.level) }
	fn has_object_at(&self, row: i64, col: i64) -> bool { self.level.dungeon[row as usize][col as usize].has_object() }
	fn try_object_at(&self, row: i64, col: i64) -> Option<&Object> { self.ground.find_object_at(row, col) }
	fn object_ids(&self) -> Vec<ObjectId> { self.ground.object_ids() }
	fn shows_skull(&self) -> bool { self.player.settings.show_skull }
	fn player_name(&self) -> String { self.player.settings.player_name().to_string() }
	fn monster_ids(&self) -> Vec<u64> { self.mash.monster_ids() }
	fn cleaned_up(&self) -> Option<String> { self.diary.cleaned_up.clone() }

	fn roll_wanderer_spot(&self, _rng: &mut impl Rng) -> Option<LevelSpot> {
		let rogue_row = self.rogue_row();
		let rogue_col = self.rogue_col();
		for _ in 0..25 {
			let spot = gr_spot_without_rogue(rogue_row, rogue_col, &self.level, |cell| {
				cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.has_object()
			});
			let rogue_cannot_see = !self.player.can_see(spot.row, spot.col, &self.level);
			if rogue_cannot_see {
				return Some(spot.into());
			}
		}
		None
	}

	fn airdrop_monster_at(&mut self, row: i64, col: i64, mut monster: Monster) {
		let mon_id = monster.id;
		monster.set_spot(row, col);
		self.level.dungeon[row as usize][col as usize].set_monster(true);
		self.mash.add_monster(monster);
		aim_monster(mon_id, self);
	}
}

impl GameState {
	pub fn dungeon_bounds(&self) -> RoomBounds {
		RoomBounds { top: 1, right: DCOLS as i64 - 1, bottom: DROWS as i64 - 2, left: 0 }
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

pub fn clean_up(estr: &str, game: &mut impl Dungeon) {
	game.set_interrupted(true);
	let diary = game.as_diary_mut();
	diary.cleaned_up = Some(estr.to_string());
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