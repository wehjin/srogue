use crate::init::GameState;
use crate::level::{add_exp, LEVEL_POINTS};
use crate::monster::show_monsters;
use crate::objects::show_objects;
use crate::potions::kind::PotionKind;
use crate::actions::quaff::STRANGE_FEELING;
use crate::random::get_rand;
use crate::render_system::RenderAction::MonstersFloorAndPlayer;

pub fn quaff_potion(potion_kind: PotionKind, game: &mut GameState) {
	match potion_kind {
		PotionKind::IncreaseStrength => {
			game.dialog.message("you feel stronger now, what bulging muscles!", 0);
			game.player.raise_strength();
		}
		PotionKind::RestoreStrength => {
			game.player.rogue.str_current = game.player.rogue.str_max;
			game.dialog.message("this tastes great, you feel warm all over", 0);
		}
		PotionKind::Healing => {
			game.dialog.message("you begin to feel better", 0);
			potion_heal(false, game);
		}
		PotionKind::ExtraHealing => {
			game.dialog.message("you begin to feel much better", 0);
			potion_heal(true, game);
		}
		PotionKind::Poison => {
			if !game.player.ring_effects.has_sustain_strength() {
				game.player.rogue.str_current -= get_rand(1, 3);
				if game.player.rogue.str_current < 1 {
					game.player.rogue.str_current = 1;
				}
			}
			game.dialog.message("you feel very sick now", 0);
			if game.player.halluc.is_active() {
				crate::r#use::unhallucinate(game);
			}
		}
		PotionKind::RaiseLevel => {
			game.player.rogue.exp_points = LEVEL_POINTS[(game.player.rogue.exp - 1) as usize];
			add_exp(1, true, game);
		}
		PotionKind::Blindness => {
			if game.player.blind.is_inactive() {
				game.dialog.message("a cloak of darkness falls around you", 0);
			}
			game.player.blind.extend(get_rand(500, 800));
			show_blind(game);
		}
		PotionKind::Hallucination => {
			game.dialog.message("oh wow, everything seems so cosmic", 0);
			let amount = get_rand(500, 800);
			game.player.halluc.extend(amount);
		}
		PotionKind::DetectMonster => {
			show_monsters(game);
			if game.mash.is_empty() {
				game.dialog.message(STRANGE_FEELING, 0);
			}
		}
		PotionKind::DetectObjects => {
			if game.ground.is_empty() {
				game.dialog.message(STRANGE_FEELING, 0);
			} else {
				if game.player.blind.is_inactive() {
					show_objects(game);
				}
			}
		}
		PotionKind::Confusion => {
			let msg = if game.player.halluc.is_active() {
				"what a trippy feeling"
			} else {
				"you feel confused"
			};
			game.dialog.message(msg, 0);
			crate::r#use::confuse(&mut game.player);
		}
		PotionKind::Levitation => {
			game.dialog.message("you start to float in the air", 0);
			game.player.levitate.extend(get_rand(15, 30));
			game.level.bear_trap = 0;
			game.level.being_held = false;
		}
		PotionKind::HasteSelf => {
			game.dialog.message("you feel yourself moving much faster", 0);
			game.player.haste_self.extend(get_rand(11, 21));
			game.player.haste_self.ensure_half_active();
		}
		PotionKind::SeeInvisible => {
			game.dialog.message(&format!("hmm, this potion tastes like {} juice", game.player.settings.fruit.trim()), 0);
			if game.player.blind.is_active() {
				crate::r#use::unblind(game);
			}
			game.level.see_invisible = true;
			crate::r#use::relight(game);
		}
	}
}

fn potion_heal(extra: bool, game: &mut GameState) {
	game.player.rogue.hp_current += game.player.rogue.exp;
	let mut ratio = game.player.rogue.hp_current as f32 / game.player.rogue.hp_max as f32;
	if ratio >= 1.00 {
		let raise_max = if extra { 2 } else { 1 };
		game.player.raise_hp_max(raise_max);
		game.player.extra_hp += raise_max;
		game.player.rogue.hp_current = game.player.rogue.hp_max;
	} else if ratio >= 0.90 {
		let raise_max = if extra { 1 } else { 0 };
		game.player.raise_hp_max(raise_max);
		game.player.extra_hp += raise_max;
		game.player.rogue.hp_current = game.player.rogue.hp_max;
	} else {
		ratio = ratio.max(0.33);
		if extra {
			ratio += ratio;
		}
		let missing_hp = game.player.rogue.hp_max - game.player.rogue.hp_current;
		let restore = ratio * (missing_hp as f32);
		game.player.rogue.hp_current += restore as isize;
		game.player.rogue.hp_current = game.player.rogue.hp_current.min(game.player.rogue.hp_max);
	}
	if game.player.blind.is_active() {
		crate::r#use::unblind(game);
	}
	if game.player.confused.is_active() {
		if extra {
			crate::r#use::unconfuse(game);
		} else {
			game.player.confused.halve();
		}
	}
	if game.player.halluc.is_active() {
		if extra {
			crate::r#use::unhallucinate(game);
		} else {
			game.player.halluc.halve();
		}
	}
}

fn show_blind(game: &mut GameState) {
	game.render(&[MonstersFloorAndPlayer]);
}
