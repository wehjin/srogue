#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{chtype, mvaddch};
use serde::{Deserialize, Serialize};
use TrapKind::NoTrap;
use crate::hit::{DamageEffect, DamageStat, get_damage, get_dir_rc};
use crate::level::constants::{DCOLS, DROWS, MAX_TRAP};
use crate::level::{CellKind, Level, new_level_message};
use crate::message::{CANCEL, check_message, message, print_stats, rgetchar, sound_bell};
use crate::play::interrupted;
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::stat_const::{STAT_HP, STAT_STRENGTH};
use crate::r#move::{is_direction, reg_move};
use crate::r#use::{take_a_nap, tele};
use crate::random::{get_rand, rand_percent};
use crate::room::{get_dungeon_char, gr_row_col};
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

pub static mut trap_door: bool = false;

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

pub unsafe fn trap_at(row: usize, col: usize, level: &Level) -> TrapKind {
	for i in 0..MAX_TRAP {
		if level.traps[i].trap_type != NoTrap {
			break;
		}
		if level.traps[i].trap_row == row && level.traps[i].trap_col == col {
			return level.traps[i].trap_type;
		}
	}
	return NoTrap;
}

pub unsafe fn trap_player(row: usize, col: usize, player: &mut Player, level: &mut Level) {
	let t = trap_at(row, col, level);
	if t == NoTrap {
		return;
	}
	level.dungeon[row][col].remove_kind(CellKind::Hidden);
	if rand_percent(player.buffed_exp() as usize) {
		message("the trap failed", 1);
		return;
	}
	match t {
		NoTrap => unreachable!("no trap"),
		TrapDoor => {
			trap_door = true;
			new_level_message = Some(trap_message(t).to_string());
		}
		BearTrap => {
			message(trap_message(t), 1);
			level.bear_trap = get_rand(4, 7);
		}
		TeleTrap => {
			mvaddch(player.rogue.row as i32, player.rogue.col as i32, chtype::from('^'));
			tele(player, level);
		}
		DartTrap => {
			message(trap_message(t), 1);
			const DART_DAMAGE: DamageStat = DamageStat { hits: 1, damage: 6 };
			player.rogue.hp_current -= get_damage(&[DART_DAMAGE], DamageEffect::Roll);
			if player.rogue.hp_current <= 0 {
				player.rogue.hp_current = 0;
			}
			if !player.ring_effects.has_sustain_strength() && rand_percent(40) && player.rogue.str_current >= 3 {
				player.rogue.str_current -= 1;
			}
			print_stats(STAT_HP | STAT_STRENGTH, player);
			if player.rogue.hp_current <= 0 {
				killed_by(Ending::PoisonDart, player);
			}
		}
		SleepingGasTrap => {
			message(trap_message(t), 1);
			take_a_nap(player, level);
		}
		RustTrap => {
			message(trap_message(t), 1);
			rust(None, player);
		}
	}
}

pub unsafe fn add_traps(player: &Player, level: &mut Level) {
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
			loop {
				row = get_rand((level.rooms[cur_party_room].top_row + 1) as usize, (level.rooms[cur_party_room].bottom_row - 1) as usize);
				col = get_rand((level.rooms[cur_party_room].left_col + 1) as usize, (level.rooms[cur_party_room].right_col - 1) as usize);
				tries += 1;
				const REPEAT_KINDS: [CellKind; 4] = [CellKind::Object, CellKind::Stairs, CellKind::Trap, CellKind::Tunnel];
				let repeat_loop = (level.dungeon[row][col].is_any_kind(&REPEAT_KINDS) || level.dungeon[row][col].is_nothing()) && tries < 15;
				if !repeat_loop {
					break;
				}
			}
			if tries < 15 {
				(row, col)
			} else {
				random_spot_with_floor_or_monster(player, level)
			}
		} else {
			random_spot_with_floor_or_monster(player, level)
		};
		level.traps[i].set_spot(row, col);
		level.dungeon[row][col].add_kind(CellKind::Trap);
		level.dungeon[row][col].add_kind(CellKind::Hidden);
	}
}

unsafe fn random_spot_with_floor_or_monster(player: &Player, level: &mut Level) -> (usize, usize) {
	let mut row = 0;
	let mut col = 0;
	gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Monster], player, level);
	(row as usize, col as usize)
}

pub unsafe fn id_trap(player: &Player, level: &Level) {
	message("direction? ", 0);
	let mut dir: char;
	loop {
		dir = rgetchar();
		if is_direction(dir) {
			break;
		}
		sound_bell();
	}
	check_message();
	if dir == CANCEL {
		return;
	}

	let mut row = player.rogue.row;
	let mut col = player.rogue.col;
	get_dir_rc(dir, &mut row, &mut col, false);
	if level.dungeon[row as usize][col as usize].is_trap() && !level.dungeon[row as usize][col as usize].is_hidden() {
		message(trap_at(row as usize, col as usize, level).name(), 0);
	} else {
		message("no trap there", 0);
	}
}


pub unsafe fn show_traps(level: &Level) {
	for i in 0..DROWS {
		for j in 0..DCOLS {
			if level.dungeon[i][j].is_trap() {
				mvaddch(i as i32, j as i32, chtype::from('^'));
			}
		}
	}
}

pub unsafe fn search(n: usize, is_auto: bool, player: &mut Player, level: &mut Level) {
	static mut reg_search: bool = false;

	let mut found = 0;
	for i in -1..=1 {
		for j in -1..=1 {
			let row = player.rogue.row + i;
			let col = player.rogue.col + j;
			if is_off_screen(row, col) {
				continue;
			}
			if level.dungeon[row as usize][col as usize].is_hidden() {
				found += 1;
			}
		}
	}

	let mut shown = 0;
	for _s in 0..n {
		for i in -1..=1 {
			for j in -1..=1 {
				let row = player.rogue.row + i;
				let col = player.rogue.col + j;
				if is_off_screen(row, col) {
					continue;
				}
				if level.dungeon[row as usize][col as usize].is_hidden() {
					if rand_percent(17 + player.buffed_exp() as usize) {
						level.dungeon[row as usize][col as usize].remove_kind(CellKind::Hidden);
						if player.blind.is_inactive() && !player.is_at(row, col) {
							mvaddch(row as i32, col as i32, get_dungeon_char(row, col, player, level));
						}
						shown += 1;
						if level.dungeon[row as usize][col as usize].is_trap() {
							message(trap_at(row as usize, col as usize, level).name(), 1);
						}
					}
				}
				if (shown == found && found > 0) || interrupted {
					return;
				}
			}
		}
		if !is_auto {
			reg_search = !reg_search;
			if reg_search {
				reg_move(player, level);
			}
		}
	}
}

pub fn is_off_screen(row: i64, col: i64) -> bool {
	row < MIN_ROW || row >= (DROWS - 1) as i64 || col < 0 || col >= DCOLS as i64
}
