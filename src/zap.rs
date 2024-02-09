#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{mvaddch, mvinch};
use crate::prelude::*;
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Wands;
use crate::prelude::wand_kind::WandKind;
use crate::settings::set_score_only;

pub static mut wizard: bool = false;

pub unsafe fn zapp(player: &Player, level: &mut Level) {
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
		if let Some(monster_id) = get_zapped_monster(dir, &mut row, &mut col, level) {
			if let Some(monster) = MASH.monster_with_id_mut(monster_id) {
				monster.wake_up();
				zap_monster(monster, (*wand).which_kind, player, level);
				relight(level);
			}
		}
	}
	reg_move(player, level);
}

pub unsafe fn get_zapped_monster(dir: char, row: &mut i64, col: &mut i64, level: &Level) -> Option<u64> {
	loop {
		let orow = *row;
		let ocol = *col;
		get_dir_rc(dir, row, col, false);
		if (*row == orow && *col == ocol)
			|| level.dungeon[*row as usize][*col as usize].is_any_kind(&[CellKind::HorizontalWall, CellKind::VerticalWall])
			|| level.dungeon[*row as usize][*col as usize].is_nothing() {
			return None;
		}
		if level.dungeon[*row as usize][*col as usize].is_monster() {
			if !imitating(*row, *col, level) {
				return MASH.monster_at_spot(*row, *col).map(|m| m.id());
			}
		}
	}
}

pub unsafe fn zap_monster(monster: &mut Monster, which_kind: u16, player: &Player, level: &mut Level) {
	let row = monster.spot.row;
	let col = monster.spot.col;
	match WandKind::from_index(which_kind as usize) {
		WandKind::SlowMonster => {
			if monster.m_flags.hasted {
				monster.m_flags.hasted = false;
			} else {
				monster.slowed_toggle = false;
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
			tele_away(monster, level);
		}
		WandKind::ConfuseMonster => {
			monster.m_flags.confused = true;
			monster.moves_confused += get_rand(12, 22);
		}
		WandKind::Invisibility => {
			monster.m_flags.invisible = true;
		}
		WandKind::Polymorph => unsafe {
			if monster.m_flags.holds {
				level.being_held = false;
			}
			let mut morph_monster = gr_monster(player.cur_depth, 0, Some(MonsterKind::random_any()));
			morph_monster.set_spot(row, col);
			morph_monster.trail_char = monster.trail_char;
			if !morph_monster.m_flags.imitates {
				morph_monster.wake_up();
			}
			if let Some(id) = FIGHT_MONSTER {
				if id == monster.id() {
					FIGHT_MONSTER = Some(morph_monster.id());
				}
			}
			MASH.remove_monster(monster.id());
			MASH.add_monster(morph_monster);
		}
		WandKind::PutToSleep => {
			monster.m_flags.asleep = true;
			monster.m_flags.napping = true;
			monster.nap_length = get_rand(3, 6);
		}
		WandKind::MagicMissile => {
			rogue_hit(monster, true, player, level);
		}
		WandKind::Cancellation => {
			if monster.m_flags.holds {
				level.being_held = false;
			}
			if monster.m_flags.steals_item {
				monster.drop_percent = 0;
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

unsafe fn tele_away(monster: &mut Monster, level: &mut Level) {
	if monster.m_flags.holds {
		level.being_held = false;
	}
	let (row, col) = {
		let mut row = 0;
		let mut col = 0;
		gr_row_col(&mut row, &mut col, &[CellKind::Floor, CellKind::Tunnel, CellKind::Stairs, CellKind::Object], level);
		(row, col)
	};

	mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.trail_char);
	level.dungeon[monster.spot.row as usize][monster.spot.col as usize].remove_kind(CellKind::Monster);
	monster.spot.row = row;
	monster.spot.col = col;
	level.dungeon[row as usize][col as usize].add_kind(CellKind::Monster);
	monster.trail_char = mvinch(row as i32, col as i32);

	if level.detect_monster || rogue_can_see(row, col, level) {
		mvaddch(row as i32, col as i32, gmc(monster, level));
	}
}

pub unsafe fn wizardize() {
	if wizard {
		wizard = false;
		message("not wizard anymore", 0);
	} else {
		let line = get_input_line::<String>("wizard's password:", None, None, false, false);
		if !line.is_empty() {
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
