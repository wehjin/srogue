use rand::thread_rng;
use wand_kind::WandKind;

use crate::hit::rogue_hit;
use crate::init::GameState;
use crate::level::Level;
use crate::monster::{MonsterIndex, MonsterMash};
use crate::motion::{get_dir_or_cancel, MoveDirection};
use crate::pack::pack_letter;
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Wands;
use crate::r#use::relight;
use crate::random::get_rand;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::resources::level::setup::npc::roll_monster;
use crate::room::gr_spot;
use crate::spec_hit::imitating;

pub(crate) mod constants;
pub(crate) mod wand_kind;
pub(crate) mod wand_materials;

pub fn zapp(game: &mut GameState) {
	let dir = get_dir_or_cancel(game);
	if dir == CANCEL_CHAR {
		return;
	}
	let ch = pack_letter("zap with what?", Wands, game);
	if ch == CANCEL_CHAR {
		return;
	}

	match game.player.object_id_with_letter(ch) {
		None => {
			game.diary.add_entry("no such item.");
			return;
		}
		Some(obj_id) => {
			if game.player.object_what(obj_id) != Wand {
				game.diary.add_entry("you can't zap with that");
				return;
			}
			if game.player.expect_object(obj_id).class <= 0 {
				game.diary.add_entry("nothing happens");
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
			todo!("reg_move(game);");
		}
	}
}

pub fn get_zapped_monster(dir: char, row: &mut i64, col: &mut i64, mash: &mut MonsterMash, level: &Level) -> Option<u64> {
	loop {
		let orow = *row as usize;
		let ocol = *col as usize;
		let (to_row, to_col) = MoveDirection::from(dir).apply_confined(*row, *col);
		*row = to_row as i64;
		*col = to_col as i64;

		if (to_row == orow && to_col == ocol)
			|| level.dungeon[to_row][to_col].is_any_wall()
			|| level.dungeon[to_row][to_col].is_nothing() {
			return None;
		}

		if level.dungeon[to_row][to_row].has_monster() && !imitating(to_row, to_col, mash, level) {
			return mash.monster_at_spot(to_row as i64, to_col as i64).map(|m| m.id());
		}
	}
}

pub fn zap_monster(mon_id: u64, which_kind: u16, game: &mut GameState) {
	let row;
	let col;
	{
		let monster = game.mash.monster(mon_id);
		row = monster.spot.row;
		col = monster.spot.col;
	}
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
			if game.mash.monster(mon_id).m_flags.holds {
				let health = game.as_health_mut();
				health.being_held = false;
			}
			let mut morph_monster = roll_monster(game.player.cur_depth, 0, &mut thread_rng());
			morph_monster.set_spot(row, col);
			if !morph_monster.m_flags.imitates {
				morph_monster.wake_up();
			}
			if let Some(fight_id) = game.player.fight_monster {
				if fight_id == game.mash.monster(mon_id).id() {
					game.player.fight_monster = Some(morph_monster.id());
				}
			}
			game.mash.remove_monster(game.mash.monster(mon_id).id());
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
			if game.mash.monster(mon_id).m_flags.holds {
				let health = game.as_health_mut();
				health.being_held = false;
			}
			if game.mash.monster(mon_id).m_flags.steals_item {
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
			game.diary.add_entry("nothing happens");
		}
	}
}

fn tele_away(mon_id: MonsterIndex, game: &mut GameState) {
	if game.as_monster_flags(mon_id).holds {
		let health = game.as_health_mut();
		health.being_held = false;
	}
	let tele_from = game.as_monster(mon_id).spot;
	let tele_to = gr_spot(
		|cell| cell.is_any_floor() || cell.is_any_tunnel() || cell.is_stairs() || cell.has_object(),
		&game.player,
		&game.level,
	);
	game.level.cell_mut(tele_from).set_monster(false);
	let monster = game.as_monster_mut(mon_id);
	monster.spot = tele_to;
	game.level.cell_mut(tele_to).set_monster(true);
	game.render_spot(tele_from);
	game.render_spot(tele_to);
}
