#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{mvaddch, mvinch};
use crate::monster::flags::{MONSTERS};
use crate::prelude::*;
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Wands;
use crate::prelude::SpotFlag::{Floor, HorWall, Monster, Nothing, Object, Stairs, Tunnel, VertWall};
use crate::prelude::wand_kind::WandKind;
use crate::settings::set_score_only;

pub static mut wizard: bool = false;

pub unsafe fn zapp() {
	let dir = get_dir_or_cancel();
	check_message();
	if dir == CANCEL {
		return;
	}

	let wch = pack_letter("zap with what?", Wands);
	if wch == CANCEL {
		return;
	}
	check_message();

	let wand = get_letter_object(wch);
	if wand.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*wand).what_is != Wand {
		message("you can't zap with that", 0);
		return;
	}

	if (*wand).class <= 0 {
		message("nothing happens", 0);
	} else {
		(*wand).class -= 1;
		let mut row = rogue.row;
		let mut col = rogue.col;
		let monster = get_zapped_monster(dir, &mut row, &mut col);
		if !monster.is_null() {
			wake_up(&mut *monster);
			zap_monster(&mut *monster, (*wand).which_kind);
			relight();
		}
	}
	reg_move();
}

pub unsafe fn get_zapped_monster(dir: char, row: &mut i64, col: &mut i64) -> *mut object {
	loop {
		let orow = *row;
		let ocol = *col;
		get_dir_rc(dir, row, col, false);
		if (*row == orow && *col == ocol)
			|| HorWall.is_set(dungeon[*row as usize][*col as usize])
			|| VertWall.is_set(dungeon[*row as usize][*col as usize])
			|| Nothing.is_set(dungeon[*row as usize][*col as usize]) {
			return 0 as *mut object;
		}
		if Monster.is_set(dungeon[*row as usize][*col as usize]) {
			if !imitating(*row, *col) {
				return object_at(&mut level_monsters, *row, *col);
			}
		}
	}
}

pub unsafe fn zap_monster(monster: &mut obj, which_kind: u16) {
	let row = monster.row;
	let col = monster.col;
	match WandKind::from_index(which_kind as usize) {
		WandKind::SlowMonster => {
			if monster.m_flags.hasted {
				monster.m_flags.hasted = false;
			} else {
				monster.set_slowed_toggle(false);
				monster.m_flags.slowed = true;
			}
		}
		WandKind::HasteMonster => {
			if monster.m_flags.slowed {
				monster.m_flags.slowed = false;
			} else {
				monster.m_flags.hasted = true;
			}
		}
		WandKind::TeleAway => {
			tele_away(monster);
		}
		WandKind::ConfuseMonster => {
			monster.m_flags.confused = true;
			monster.set_moves_confused(monster.moves_confused() + get_rand(12, 22));
		}
		WandKind::Invisibility => {
			monster.m_flags.invisible = true;
		}
		WandKind::Polymorph => unsafe {
			if monster.m_flags.holds {
				being_held = false;
			}
			let nm = monster.next_monster();
			let tc = monster.trail_char();
			gr_monster(monster, get_rand(0, MONSTERS - 1));
			monster.row = row;
			monster.col = col;
			monster.set_next_monster(nm);
			monster.set_trail_char(tc);
			if !monster.m_flags.imitates {
				wake_up(monster);
			}
		}
		WandKind::PutToSleep => {
			monster.m_flags.asleep = true;
			monster.m_flags.napping = true;
			monster.set_nap_length(get_rand(3, 6));
		}
		WandKind::MagicMissile => {
			rogue_hit(monster, true);
		}
		WandKind::Cancellation => {
			if monster.m_flags.holds {
				being_held = false;
			}
			if monster.m_flags.steals_item {
				monster.set_drop_percent(0);
			}
			monster.m_flags.flies = false;
			monster.m_flags.flits = false;
			monster.m_flags.set_special_hit(false);
			monster.m_flags.invisible = false;
			monster.m_flags.flames = false;
			monster.m_flags.imitates = false;
			monster.m_flags.confuses = false;
			monster.m_flags.seeks_gold = false;
			monster.m_flags.holds = false;
		}
		WandKind::DoNothing => {
			message("nothing happens", 0);
		}
	}
}

unsafe fn tele_away(monster: &mut obj) {
	if monster.m_flags.holds {
		being_held = false;
	}
	let (row, col) = {
		let mut row = 0;
		let mut col = 0;
		gr_row_col(&mut row, &mut col, vec![Floor, Tunnel, Stairs, Object]);
		(row, col)
	};

	mvaddch(monster.row as i32, monster.col as i32, monster.trail_char());
	Monster.clear(&mut dungeon[monster.row as usize][monster.col as usize]);

	monster.row = row;
	monster.col = col;
	Monster.set(&mut dungeon[row as usize][col as usize]);
	monster.set_trail_char(mvinch(row as i32, col as i32));

	if detect_monster || rogue_can_see(row, col) {
		mvaddch(row as i32, col as i32, gmc(monster));
	}
}

pub unsafe fn wizardize() {
	if wizard {
		wizard = false;
		message("not wizard anymore", 0);
	} else {
		let line = get_input_line::<String>("wizard's password:", None, None, false, false);
		if !line.is_empty() {
			xxx(true);
			if line == "\u{A7}DV\u{BA}M\u{A3}\u{17}" {
				wizard = true;
				set_score_only(true);
				message("Welcome, mighty wizard!", 0);
			} else {
				message("sorry", 0);
			}
		}
	}
}
