use wand_kind::WandKind;

use crate::hit::{get_dir_rc, rogue_hit};
use crate::init::GameState;
use crate::level::Level;
use crate::message::get_input_line;
use crate::monster::{gr_monster, MonsterIndex, MonsterKind, MonsterMash};
use crate::motion::{get_dir_or_cancel, reg_move};
use crate::pack::pack_letter;
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Wands;
use crate::r#use::relight;
use crate::random::get_rand;
use crate::resources::keyboard::CANCEL;
use crate::room::gr_spot;
use crate::spec_hit::imitating;

pub(crate) mod constants;
pub(crate) mod wand_kind;
pub(crate) mod wand_materials;

pub fn zapp(game: &mut GameState) {
	let dir = get_dir_or_cancel(game);
	game.dialog.clear_message();
	if dir == CANCEL {
		return;
	}
	let ch = pack_letter("zap with what?", Wands, game);
	if ch == CANCEL {
		return;
	}

	game.dialog.clear_message();
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			if game.player.object_what(obj_id) != Wand {
				game.dialog.message("you can't zap with that", 0);
				return;
			}
			if game.player.expect_object(obj_id).class <= 0 {
				game.dialog.message("nothing happens", 0);
			} else {
				game.player.expect_object_mut(obj_id).class -= 1;
				let mut row = game.player.rogue.row;
				let mut col = game.player.rogue.col;
				if let Some(mon_id) = get_zapped_monster(dir, &mut row, &mut col, &mut game.mash, &game.level) {
					let monster = game.mash.monster_mut(mon_id);
					let obj_kind = game.player.object_kind(obj_id);
					monster.wake_up();
					zap_monster(monster.id(), obj_kind, game);
					relight(game);
				}
			}
			reg_move(game);
		}
	}
}

pub fn get_zapped_monster(dir: char, row: &mut i64, col: &mut i64, mash: &mut MonsterMash, level: &Level) -> Option<u64> {
	loop {
		let orow = *row;
		let ocol = *col;
		get_dir_rc(dir, row, col, false);
		if (*row == orow && *col == ocol)
			|| level.dungeon[*row as usize][*col as usize].is_any_wall()
			|| level.dungeon[*row as usize][*col as usize].is_nothing() {
			return None;
		}
		if level.dungeon[*row as usize][*col as usize].has_monster() {
			if !imitating(*row, *col, mash, level) {
				return mash.monster_at_spot(*row, *col).map(|m| m.id());
			}
		}
	}
}

pub fn zap_monster(mon_id: u64, which_kind: u16, game: &mut GameState) {
	let monster = game.mash.monster(mon_id);
	let row = monster.spot.row;
	let col = monster.spot.col;
	match WandKind::from_index(which_kind as usize) {
		WandKind::SlowMonster => {
			let monster = game.mash.monster_mut(mon_id);
			if monster.m_flags.hasted {
				monster.m_flags.hasted = false;
			} else {
				monster.slowed_toggle = false;
				monster.m_flags.slowed = true;
			}
		}
		WandKind::HasteMonster => {
			let monster = game.mash.monster_mut(mon_id);
			if monster.m_flags.slowed {
				monster.m_flags.slowed = false;
			} else {
				monster.m_flags.hasted = true;
			}
		}
		WandKind::TeleAway => {
			tele_away(mon_id, game);
		}
		WandKind::ConfuseMonster => {
			let monster = game.mash.monster_mut(mon_id);
			monster.m_flags.confused = true;
			monster.moves_confused += get_rand(12, 22);
		}
		WandKind::Invisibility => {
			let monster = game.mash.monster_mut(mon_id);
			monster.m_flags.invisible = true;
		}
		WandKind::Polymorph => {
			if monster.m_flags.holds {
				game.level.being_held = false;
			}
			let mut morph_monster = gr_monster(game.player.cur_depth, 0, Some(MonsterKind::random_any()));
			morph_monster.set_spot(row, col);
			if !morph_monster.m_flags.imitates {
				morph_monster.wake_up();
			}
			if let Some(fight_id) = game.player.fight_monster {
				if fight_id == monster.id() {
					game.player.fight_monster = Some(morph_monster.id());
				}
			}
			game.mash.remove_monster(monster.id());
			game.mash.add_monster(morph_monster);
		}
		WandKind::PutToSleep => {
			let monster = game.mash.monster_mut(mon_id);
			monster.m_flags.asleep = true;
			monster.m_flags.napping = true;
			monster.nap_length = get_rand(3, 6);
		}
		WandKind::MagicMissile => {
			rogue_hit(mon_id, true, game);
		}
		WandKind::Cancellation => {
			if monster.m_flags.holds {
				game.level.being_held = false;
			}
			if monster.m_flags.steals_item {
				let monster = game.mash.monster_mut(mon_id);
				monster.drop_percent = 0;
			}
			let monster = game.mash.monster_mut(mon_id);
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
			game.dialog.message("nothing happens", 0);
		}
	}
}

fn tele_away(mon_id: MonsterIndex, game: &mut GameState) {
	if game.mash.monster_flags(mon_id).holds {
		game.level.being_held = false;
	}
	let tele_from = game.mash.monster(mon_id).spot;
	let tele_to =
		gr_spot(
			|cell| cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.has_object(),
			&game.player,
			&game.level,
		);
	game.level.cell_mut(tele_from).set_monster(false);
	game.mash.monster_mut(mon_id).spot = tele_to;
	game.level.cell_mut(tele_to).set_monster(true);
	game.render_spot(tele_from);
	game.render_spot(tele_to);
}

pub fn wizardize(game: &mut GameState) {
	if game.player.wizard {
		game.player.wizard = false;
		game.dialog.message("not wizard anymore", 0);
	} else {
		let line = get_input_line::<String>("wizard's password:", None, None, false, false, game);
		if !line.is_empty() {
			//const PW: &str = "\u{A7}DV\u{BA}M\u{A3}\u{17}";
			const PW: &str = "neko?";
			if line == PW {
				game.player.wizard = true;
				game.player.settings.score_only = true;
				game.dialog.message("Welcome, mighty wizard!", 0);
			} else {
				game.dialog.message("sorry", 0);
			}
		}
	}
}
