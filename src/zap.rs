#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use libc::{c_int, strlen, strncmp};
use crate::monster::flags::{MONSTERS};
use crate::prelude::*;
use crate::prelude::object_what::PackFilter::Wands;
use crate::prelude::wand_kind::WandKind;
use crate::settings::set_score_only;

pub static mut wizard: bool = false;

#[no_mangle]
pub unsafe extern "C" fn zapp() -> i64 {
	let mut wch: libc::c_short = 0;
	let mut first_miss: libc::c_char = 1 as libc::c_char;
	let mut wand: *mut object = 0 as *mut object;
	let mut dir: libc::c_short = 0;
	let mut row: libc::c_short = 0;
	let mut col: libc::c_short = 0;
	let mut monster: *mut object = 0 as *mut object;
	loop {
		dir = rgetchar() as libc::c_short;
		if !(is_direction(dir as i32) == 0) {
			break;
		}
		sound_bell();
		if first_miss != 0 {
			message(
				b"direction? \0" as *const u8 as *const libc::c_char,
				0 as i64,
			);
			first_miss = 0 as i64 as libc::c_char;
		}
	}
	check_message();
	if dir as i64 == '\u{1b}' as i32 {
		return;
	}
	wch = pack_letter("zap with what?", Wands) as libc::c_short;
	if wch as i64 == '\u{1b}' as i32 {
		return;
	}
	check_message();
	wand = get_letter_object(wch as i64);
	if wand.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*wand).what_is as i64
		!= 0o100 as i64 as libc::c_ushort as i64
	{
		message(
			b"you can't zap with that\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*wand).class as i64 <= 0 as i64 {
		message(
			b"nothing happens\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
	} else {
		(*wand).class -= 1;
		(*wand).class;
		row = rogue.row;
		col = rogue.col;
		monster = get_zapped_monster(dir as i64, &mut row, &mut col);
		if !monster.is_null() {
			wake_up(monster);
			zap_monster(monster, (*wand).which_kind as i64);
			relight();
		}
	}
	reg_move();
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn get_zapped_monster(
	mut dir: libc::c_short,
	mut row: *mut libc::c_short,
	mut col: *mut libc::c_short,
) -> *mut object {
	let mut orow: libc::c_short = 0;
	let mut ocol: libc::c_short = 0;
	loop {
		orow = *row;
		ocol = *col;
		get_dir_rc(dir as i64, row, col, 0 as i64);
		if *row as i64 == orow as i64
			&& *col as i64 == ocol as i64
			|| dungeon[*row as usize][*col as usize] as i64
			& (0o10 as i64 as libc::c_ushort as i64
			| 0o20 as i64 as libc::c_ushort as i64) != 0
			|| dungeon[*row as usize][*col as usize] as i64
			== 0 as i64 as libc::c_ushort as i64
		{
			return 0 as *mut object;
		}
		if dungeon[*row as usize][*col as usize] as i64
			& 0o2 as i64 as libc::c_ushort as i64 != 0
		{
			if !imitating(*row as i64, *col as i64) {
				return object_at(
					&mut level_monsters,
					*row as i64,
					*col as i64,
				);
			}
		}
	};
}

pub unsafe fn zap_monster(monster: &mut obj, which_kind: u16) {
	let row = monster.row;
	let col = monster.col;
	let kind = WandKind::from_code(which_kind);
	match kind {
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
			gr_monster(monster, get_rand(0, (MONSTERS - 1) as c_int));
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

#[no_mangle]
pub unsafe extern "C" fn wizardize() {
	let mut buf: [libc::c_char; 100] = [0; 100];
	if wizard != 0 {
		wizard = 0 as i64 as libc::c_char;
		message("not wizard anymore", 0 as i64);
	} else {
		let line = get_input_line("wizard's password:", None, None, false, false);
		if !line.is_empty() {
			xxx(true);
			xxxx(buf.as_mut_ptr(), strlen(buf.as_mut_ptr()));
			if strncmp(
				buf.as_mut_ptr(),
				b"\xA7DV\xBAM\xA3\x17\0" as *const u8 as *const libc::c_char,
				7 as i64 as libc::c_ulong,
			) == 0
			{
				wizard = 1 as libc::c_char;
				set_score_only(true);
				message("Welcome, mighty wizard!", 0 as i64);
			} else {
				message("sorry", 0 as i64);
			}
		}
	}
}
