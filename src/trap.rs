use serde::{Deserialize, Serialize};

use TrapKind::NoTrap;

use crate::hit::{get_damage, DamageEffect, DamageStat};
use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS, MAX_TRAP};
use crate::level::Level;
use crate::message::sound_bell;
use crate::motion::{is_direction, MoveDirection};
use crate::player::Player;
use crate::prelude::ending::Ending;
use crate::prelude::*;
use crate::r#use::{take_a_nap, tele};
use crate::random::{get_rand, rand_percent};
use crate::resources::diary;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::room::gr_spot;
use crate::score::killed_by;
use crate::spec_hit::rust;
use crate::trap::trap_kind::TrapKind;
use crate::trap::trap_kind::TrapKind::{BearTrap, DartTrap, RustTrap, SleepingGasTrap, TeleTrap, TrapDoor};

pub mod trap_kind {
	use serde::{Deserialize, Serialize};

	use crate::random::get_rand;
	use crate::trap::trap_kind::TrapKind::{BearTrap, DartTrap, NoTrap, RustTrap, SleepingGasTrap, TeleTrap, TrapDoor};

	#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
	pub enum TrapKind {
		NoTrap,
		TrapDoor,
		BearTrap,
		TeleTrap,
		DartTrap,
		SleepingGasTrap,
		RustTrap,
	}

	impl Default for TrapKind {
		fn default() -> Self { NoTrap }
	}

	impl TrapKind {
		pub fn name(&self) -> &'static str {
			match self {
				NoTrap => "no trap",
				TrapDoor => "trap door",
				BearTrap => "bear trap",
				TeleTrap => "teleport trap",
				DartTrap => "poison dart trap",
				SleepingGasTrap => "sleeping gas trap",
				RustTrap => "rust trap",
			}
		}
		pub const ALL_KINDS: [TrapKind; 6] = [TrapDoor, BearTrap, TeleTrap, DartTrap, SleepingGasTrap, RustTrap];
		pub fn random() -> Self {
			let index = get_rand(0, 5) as usize;
			Self::ALL_KINDS[index]
		}
	}
}

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct Trap {
	pub trap_type: TrapKind,
	pub trap_row: usize,
	pub trap_col: usize,
}

impl Trap {
	pub fn clear(&mut self) {
		self.trap_type = NoTrap;
	}
	pub fn set_spot(&mut self, row: usize, col: usize) {
		self.trap_row = row;
		self.trap_col = col;
	}
}


pub fn trap_message(trap: TrapKind) -> &'static str {
	match trap {
		NoTrap => "a trap and an anti-trap collide",
		TrapDoor => "you fell down a trap",
		BearTrap => "you are caught in a bear trap",
		TeleTrap => "teleport",
		DartTrap => "a small dart just hit you in the shoulder",
		SleepingGasTrap => "a strange white mist envelops you and you fall asleep",
		RustTrap => "a gush of water hits you on the head"
	}
}

pub fn trap_at(row: usize, col: usize, level: &Level) -> TrapKind {
	for i in 0..MAX_TRAP {
		if level.traps[i].trap_type != NoTrap {
			break;
		}
		if level.traps[i].trap_row == row && level.traps[i].trap_col == col {
			return level.traps[i].trap_type;
		}
	}
	NoTrap
}

pub fn trap_player(row: usize, col: usize, game: &mut GameState) {
	let t = trap_at(row, col, &game.level);
	if t == NoTrap {
		return;
	}
	game.level.dungeon[row][col].set_visible();
	if rand_percent(game.player.buffed_exp() as usize) {
		game.player.interrupt_and_slurp();
		game.diary.add_entry("the trap failed");
		return;
	}
	match t {
		NoTrap => unreachable!("no trap"),
		TrapDoor => {
			game.level.trap_door = true;
			game.level.new_level_message = Some(trap_message(t).to_string());
		}
		BearTrap => {
			game.player.interrupt_and_slurp();
			game.diary.add_entry(trap_message(t));
			game.level.bear_trap = get_rand(4, 7);
		}
		TeleTrap => {
			game.render_spot(game.player.to_spot());
			tele(game);
		}
		DartTrap => {
			game.player.interrupt_and_slurp();
			game.diary.add_entry(trap_message(t));
			const DART_DAMAGE: DamageStat = DamageStat { hits: 1, damage: 6 };
			game.player.rogue.hp_current -= get_damage(&[DART_DAMAGE], DamageEffect::Roll);
			if game.player.rogue.hp_current <= 0 {
				game.player.rogue.hp_current = 0;
			}
			if !game.player.ring_effects.has_sustain_strength() && rand_percent(40) && game.player.rogue.str_current >= 3 {
				game.player.rogue.str_current -= 1;
			}
			game.stats_changed = true;
			if game.player.rogue.hp_current <= 0 {
				killed_by(Ending::PoisonDart, game);
			}
		}
		SleepingGasTrap => {
			game.player.interrupt_and_slurp();
			game.diary.add_entry(trap_message(t));
			take_a_nap(game);
		}
		RustTrap => {
			game.player.interrupt_and_slurp();
			game.diary.add_entry(trap_message(t));
			rust(None, game);
		}
	}
}

pub fn add_traps(player: &Player, level: &mut Level) {
	let n: usize;
	let cur_level = player.cur_depth;
	if cur_level <= 2 {
		n = 0;
	} else if cur_level <= 7 {
		n = get_rand(0, 2);
	} else if cur_level <= 11 {
		n = get_rand(1, 2);
	} else if cur_level <= 16 {
		n = get_rand(2, 3);
	} else if cur_level <= 21 {
		n = get_rand(2, 4);
	} else if cur_level <= (AMULET_LEVEL + 2) {
		n = get_rand(3, 5);
	} else {
		n = get_rand(5, MAX_TRAP);
	}
	for i in 0..n {
		level.traps[i].trap_type = TrapKind::random();
		let (row, col) = if i == 0 && level.party_room.is_some() {
			let cur_party_room = level.party_room.expect("some party room");
			let mut row: usize;
			let mut col: usize;
			let mut tries = 0;
			const MAX_TRIES: i32 = 15;
			loop {
				row = get_rand((level.rooms[cur_party_room].top_row + 1) as usize, (level.rooms[cur_party_room].bottom_row - 1) as usize);
				col = get_rand((level.rooms[cur_party_room].left_col + 1) as usize, (level.rooms[cur_party_room].right_col - 1) as usize);
				tries += 1;
				let cell = level.dungeon[row][col];
				let keep_looking = (cell.has_object() || cell.is_stairs() || cell.is_any_trap() || cell.is_any_tunnel() || cell.is_nothing())
					&& tries < MAX_TRIES;
				if !keep_looking {
					break;
				}
			}
			if tries < MAX_TRIES {
				(row, col)
			} else {
				random_spot_with_floor_or_monster(player, level)
			}
		} else {
			random_spot_with_floor_or_monster(player, level)
		};
		level.traps[i].set_spot(row, col);
		level.dungeon[row][col].add_hidden_trap();
	}
}

fn random_spot_with_floor_or_monster(player: &Player, level: &mut Level) -> (usize, usize) {
	let spot = gr_spot(|cell| cell.is_any_floor() || cell.has_monster(), player, level);
	(spot.row as usize, spot.col as usize)
}

pub fn id_trap(game: &mut GameState) {
	diary::show_prompt("direction? ", &mut game.diary);
	let mut dir: char;
	loop {
		dir = rgetchar();
		if is_direction(dir) {
			break;
		}
		sound_bell();
	}
	if dir == CANCEL_CHAR {
		return;
	}

	let (look_row, look_col) = MoveDirection::from(dir).apply_confined(game.player.rogue.row, game.player.rogue.col);
	if game.level.dungeon[look_row][look_col].is_any_trap() && !game.level.dungeon[look_row][look_col].is_any_hidden() {
		game.diary.add_entry(trap_at(look_row, look_col, &game.level).name());
	} else {
		game.diary.add_entry("no trap there");
	}
}

pub fn show_traps(game: &mut GameState) {
	let bounds = game.dungeon_bounds();
	for row in bounds.rows() {
		for col in bounds.cols() {
			let spot = DungeonSpot { row, col };
			if game.cell_at(spot).is_any_trap() {
				game.cell_at_mut(spot).set_visible();
				game.render_spot(spot);
			}
		}
	}
}

pub(crate) fn is_off_screen(row: i64, col: i64) -> bool {
	row < MIN_ROW || row >= ((DROWS as i64) - 1) || col < 0 || col >= (DCOLS as i64)
}
