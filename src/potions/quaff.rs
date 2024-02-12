use ncurses::{chtype, mvaddch};
use crate::level::{add_exp, cur_room, Level, LEVEL_POINTS};
use crate::message::message;
use crate::monster::{MASH, show_monsters};
use crate::objects::{level_objects, show_objects};
use crate::player::Player;
use crate::potions::kind::PotionKind;
use crate::r#use::{confused, extra_hp, haste_self, STRANGE_FEELING};
use crate::random::get_rand;
use crate::settings::fruit;

pub unsafe fn quaff_potion(potion_kind: PotionKind, player: &mut Player, level: &mut Level) {
	match potion_kind {
		PotionKind::IncreaseStrength => {
			message("you feel stronger now, what bulging muscles!", 0);
			player.rogue.str_current += 1;
			if player.rogue.str_current > player.rogue.str_max {
				player.rogue.str_max = player.rogue.str_current;
			}
		}
		PotionKind::RestoreStrength => {
			player.rogue.str_current = player.rogue.str_max;
			message("this tastes great, you feel warm all over", 0);
		}
		PotionKind::Healing => {
			message("you begin to feel better", 0);
			potion_heal(false, player, level);
		}
		PotionKind::ExtraHealing => {
			message("you begin to feel much better", 0);
			potion_heal(true, player, level);
		}
		PotionKind::Poison => {
			if !player.ring_effects.has_sustain_strength() {
				player.rogue.str_current -= get_rand(1, 3);
				if player.rogue.str_current < 1 {
					player.rogue.str_current = 1;
				}
			}
			message("you feel very sick now", 0);
			if player.halluc.is_active() {
				crate::r#use::unhallucinate(player, level);
			}
		}
		PotionKind::RaiseLevel => {
			player.rogue.exp_points = LEVEL_POINTS[(player.rogue.exp - 1) as usize];
			add_exp(1, true, player);
		}
		PotionKind::Blindness => {
			if player.blind.is_inactive() {
				message("a cloak of darkness falls around you", 0);
			}
			player.blind.extend(get_rand(500, 800));
			show_blind(player, level);
		}
		PotionKind::Hallucination => {
			message("oh wow, everything seems so cosmic", 0);
			let amount = get_rand(500, 800);
			player.halluc.extend(amount);
		}
		PotionKind::DetectMonster => {
			show_monsters(player, level);
			if MASH.is_empty() {
				message(STRANGE_FEELING, 0);
			}
		}
		PotionKind::DetectObjects => {
			if level_objects.is_empty() {
				message(STRANGE_FEELING, 0);
			} else {
				if player.blind.is_inactive() {
					show_objects(player, level);
				}
			}
		}
		PotionKind::Confusion => {
			let msg = if player.halluc.is_active() {
				"what a trippy feeling"
			} else {
				"you feel confused"
			};
			message(msg, 0);
			crate::r#use::confuse();
		}
		PotionKind::Levitation => {
			message("you start to float in the air", 0);
			player.levitate.extend(get_rand(15, 30));
			level.bear_trap = 0;
			level.being_held = false;
		}
		PotionKind::HasteSelf => {
			message("you feel yourself moving much faster", 0);
			haste_self += get_rand(11, 21);
			if haste_self % 2 == 0 {
				haste_self += 1;
			}
		}
		PotionKind::SeeInvisible => {
			message(&format!("hmm, this potion tastes like {} juice", fruit().trim()), 0);
			if player.blind.is_active() {
				crate::r#use::unblind(player, level);
			}
			level.see_invisible = true;
			crate::r#use::relight(player, level);
		}
	}
}

unsafe fn potion_heal(extra: bool, player: &mut Player, level: &mut Level) {
	player.rogue.hp_current += player.rogue.exp;

	let mut ratio = player.rogue.hp_current as f32 / player.rogue.hp_max as f32;
	if ratio >= 1.00 {
		player.rogue.hp_max += if extra { 2 } else { 1 };
		extra_hp += if extra { 2 } else { 1 };
		player.rogue.hp_current = player.rogue.hp_max;
	} else if ratio >= 0.90 {
		player.rogue.hp_max += if extra { 1 } else { 0 };
		extra_hp += if extra { 1 } else { 0 };
		player.rogue.hp_current = player.rogue.hp_max;
	} else {
		if ratio < 0.33 {
			ratio = 0.33;
		}
		if extra {
			ratio += ratio;
		}
		let add = ratio * (player.rogue.hp_max - player.rogue.hp_current) as f32;
		player.rogue.hp_current += add as isize;
		if player.rogue.hp_current > player.rogue.hp_max {
			player.rogue.hp_current = player.rogue.hp_max;
		}
	}
	if player.blind.is_active() {
		crate::r#use::unblind(player, level);
	}
	if confused != 0 && extra {
		crate::r#use::unconfuse(player);
	} else if confused != 0 {
		confused = (confused / 2) + 1;
	}
	if player.halluc.is_active() {
		if extra {
			crate::r#use::unhallucinate(player, level);
		} else {
			player.halluc.halve();
		}
	}
}

unsafe fn show_blind(player: &Player, level: &Level) {
	if level.detect_monster {
		for monster in &MASH.monsters {
			mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.trail_char);
		}
	}
	if cur_room >= 0 {
		for i in (level.rooms[cur_room as usize].top_row as usize + 1)..level.rooms[cur_room as usize].bottom_row as usize {
			for j in (level.rooms[cur_room as usize].left_col as usize + 1)..level.rooms[cur_room as usize].right_col as usize {
				mvaddch(i as i32, j as i32, chtype::from(' '));
			}
		}
	}
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, chtype::from(player.rogue.fchar));
}
