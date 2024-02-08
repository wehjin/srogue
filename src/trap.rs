#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{chtype, mvaddch};
use serde::{Deserialize, Serialize};
use TrapKind::NoTrap;
use crate::prelude::*;
use crate::prelude::ending::Ending;
use crate::prelude::SpotFlag::{Floor, Hidden, Monster, Object, Stairs, Tunnel};
use crate::prelude::stat_const::{STAT_HP, STAT_STRENGTH};
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

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Trap {
	pub trap_type: TrapKind,
	pub trap_row: usize,
	pub trap_col: usize,
}

pub const MAX_TRAP: usize = 10;
pub static mut TRAPS: [Trap; MAX_TRAP] = [Trap { trap_type: NoTrap, trap_row: 0, trap_col: 0 }; MAX_TRAP];
pub static mut trap_door: bool = false;
pub static mut bear_trap: usize = 0;

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

pub unsafe fn trap_at(row: usize, col: usize) -> TrapKind {
	for i in 0..MAX_TRAP {
		if TRAPS[i].trap_type != NoTrap {
			break;
		}
		if TRAPS[i].trap_row == row && TRAPS[i].trap_col == col {
			return TRAPS[i].trap_type;
		}
	}
	return NoTrap;
}

pub unsafe fn trap_player(row: usize, col: usize, depth: &RogueDepth, level: &Level) {
	let t = trap_at(row, col);
	if t == NoTrap {
		return;
	}
	Hidden.clear(&mut dungeon[row][col]);
	if rand_percent((rogue.exp + ring_exp) as usize) {
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
			bear_trap = get_rand(4, 7);
		}
		TeleTrap => {
			mvaddch(rogue.row as i32, rogue.col as i32, chtype::from('^'));
			tele(level);
		}
		DartTrap => {
			message(trap_message(t), 1);
			rogue.hp_current -= get_damage("1d6", DamageEffect::Roll);
			if rogue.hp_current <= 0 {
				rogue.hp_current = 0;
			}
			if !sustain_strength && rand_percent(40) && rogue.str_current >= 3 {
				rogue.str_current -= 1;
			}
			print_stats(STAT_HP | STAT_STRENGTH, depth.cur);
			if rogue.hp_current <= 0 {
				killed_by(Ending::PoisonDart, depth.max);
			}
		}
		SleepingGasTrap => {
			message(trap_message(t), 1);
			take_a_nap(depth, level);
		}
		RustTrap => {
			message(trap_message(t), 1);
			rust(None, depth.cur);
		}
	}
}

pub unsafe fn add_traps(cur_level: usize, level: &Level) {
	let n: usize;
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
		TRAPS[i].trap_type = TrapKind::random();
		let (row, col) = if i == 0 && party_room.is_some() {
			let cur_party_room = party_room.expect("party room is some");
			let mut row: usize;
			let mut col: usize;
			let mut tries = 0;
			loop {
				row = get_rand((level.rooms[cur_party_room].top_row + 1) as usize, (level.rooms[cur_party_room].bottom_row - 1) as usize);
				col = get_rand((level.rooms[cur_party_room].left_col + 1) as usize, (level.rooms[cur_party_room].right_col - 1) as usize);
				tries += 1;
				let try_again = (SpotFlag::is_any_set(&vec![Object, Stairs, SpotFlag::Trap, Tunnel], dungeon[row][col]) || SpotFlag::is_nothing(dungeon[row][col]))
					&& tries < 15;
				if !try_again {
					break;
				}
			}
			if tries >= 15 {
				let mut row = 0;
				let mut col = 0;
				gr_row_col(&mut row, &mut col, vec![Floor, Monster], level);
				(row as usize, col as usize)
			} else {
				(row, col)
			}
		} else {
			let mut row = 0;
			let mut col = 0;
			gr_row_col(&mut row, &mut col, vec![Floor, Monster], level);
			(row as usize, col as usize)
		};
		TRAPS[i].trap_row = row;
		TRAPS[i].trap_col = col;
		SpotFlag::Trap.set(&mut dungeon[row][col]);
		Hidden.set(&mut dungeon[row][col]);
	}
}

pub unsafe fn id_trap() {
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

	let mut row = rogue.row;
	let mut col = rogue.col;
	get_dir_rc(dir, &mut row, &mut col, false);
	if SpotFlag::Trap.is_set(dungeon[row as usize][col as usize]) && !Hidden.is_set(dungeon[row as usize][col as usize]) {
		let t = trap_at(row as usize, col as usize);
		message(t.name(), 0);
	} else {
		message("no trap there", 0);
	}
}


pub unsafe fn show_traps() {
	for i in 0..DROWS {
		for j in 0..DCOLS {
			if SpotFlag::Trap.is_set(dungeon[i][j]) {
				mvaddch(i as i32, j as i32, chtype::from('^'));
			}
		}
	}
}

pub unsafe fn search(n: usize, is_auto: bool, depth: &RogueDepth, level: &Level) {
	static mut reg_search: bool = false;

	let mut found = 0;
	for i in -1..=1 {
		for j in -1..=1 {
			let row = rogue.row + i;
			let col = rogue.col + j;
			if is_off_screen(row, col) {
				continue;
			}
			if Hidden.is_set(dungeon[row as usize][col as usize]) {
				found += 1;
			}
		}
	}

	let mut shown = 0;
	for _s in 0..n {
		for i in -1..=1 {
			for j in -1..=1 {
				let row = rogue.row + i;
				let col = rogue.col + j;
				if is_off_screen(row, col) {
					continue;
				}
				if Hidden.is_set(dungeon[row as usize][col as usize]) {
					if rand_percent(17 + (rogue.exp + ring_exp) as usize) {
						Hidden.clear(&mut dungeon[row as usize][col as usize]);
						if not_blind() && no_rogue(row, col) {
							mvaddch(row as i32, col as i32, get_dungeon_char(row, col));
						}
						shown += 1;
						if SpotFlag::Trap.is_set(dungeon[row as usize][col as usize]) {
							let t = trap_at(row as usize, col as usize);
							message(t.name(), 1);
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
				reg_move(depth, level);
			}
		}
	}
}

pub unsafe fn no_rogue(row: i64, col: i64) -> bool {
	let no_rogue = row != rogue.row || col != rogue.col;
	no_rogue
}

pub unsafe fn not_blind() -> bool {
	let not_blind = blind == 0;
	not_blind
}

pub fn is_off_screen(row: i64, col: i64) -> bool {
	row < MIN_ROW || row >= (DROWS - 1) as i64 || col < 0 || col >= DCOLS as i64
}
