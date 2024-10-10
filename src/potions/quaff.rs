use crate::actions::quaff::STRANGE_FEELING;
use crate::init::{Dungeon, GameState};
use crate::potions::kind::PotionKind;
use crate::prelude::MAX_STRENGTH;
use crate::resources::avatar::Avatar;
pub enum PotionEffect {
	None,
	Report(Vec<String>),
	Upgrade,
}
impl PotionEffect {
	pub fn none() -> Self { Self::None }
	pub fn report(report: impl AsRef<str>) -> Self { Self::Report(vec![report.as_ref().to_string()]) }
	pub fn upgrade() -> Self { Self::Upgrade }
}

pub fn quaff_potion(potion_kind: PotionKind, game: &mut impl Dungeon) -> PotionEffect {
	match potion_kind {
		PotionKind::IncreaseStrength => {
			let fighter = game.as_fighter_mut();
			fighter.str_current = (fighter.str_current + 1).min(MAX_STRENGTH);
			fighter.str_max = fighter.str_max.max(fighter.str_current);
			PotionEffect::report("you feel stronger now, what bulging muscles!")
		}
		PotionKind::RestoreStrength => {
			let fighter = game.as_fighter_mut();
			fighter.str_current = fighter.str_max;
			PotionEffect::report("this tastes great, you feel warm all over")
		}
		PotionKind::Healing => {
			// TODO potion_heal(false, game);
			PotionEffect::report("you begin to feel better")
		}
		PotionKind::ExtraHealing => {
			// TODO potion_heal(true, game);
			PotionEffect::report("you begin to feel much better")
		}
		PotionKind::Poison => {
			if !game.as_ring_effects().has_sustain_strength() {
				let amount = game.roll_range(1..=3);
				let fighter = game.as_fighter_mut();
				fighter.str_current = (fighter.str_current - amount).max(1);
			}
			let poison_report = "you feel very u now".to_string();
			if game.as_health().halluc.is_active() {
				// TODO crate::r#use::unhallucinate(game);
			}
			let unhalluc_report = "".to_string();  // TODO set unhalloc report.
			PotionEffect::Report(vec![poison_report, unhalluc_report])
		}
		PotionKind::RaiseLevel => {
			let fighter = game.as_fighter_mut();
			fighter.exp.raise_level();
			fighter.exp.add_points(1);
			PotionEffect::upgrade()
		}
		PotionKind::Blindness => {
			let reports = if game.as_health().blind.is_inactive() {
				vec!["a cloak of darkness falls around you".to_string()]
			} else {
				vec![]
			};
			let amount = game.roll_range(500..=800);
			game.as_health_mut().blind.extend(amount);
			// TODO Make sure blind is rendered correctly. show_blind(game)
			PotionEffect::Report(reports)
		}
		PotionKind::Hallucination => {
			let amount = game.roll_range(500..=800);
			game.as_health_mut().halluc.extend(amount);
			PotionEffect::report("oh wow, everything seems so cosmic")
		}
		PotionKind::DetectMonster => {
			// todo!("Mark the level or rogue as showing monsters and respect value when printing. show_monsters(game);");
			if game.monster_ids().is_empty() {
				PotionEffect::report(STRANGE_FEELING)
			} else {
				PotionEffect::none()
			}
		}
		PotionKind::DetectObjects => {
			if game.object_ids().is_empty() {
				PotionEffect::report(STRANGE_FEELING)
			} else {
				if game.as_health().blind.is_inactive() {
					// todo!("Make sure objects are visible. show_objects(game);");
				}
				PotionEffect::none()
			}
		}
		PotionKind::Confusion => {
			crate::r#use::confuse(game);
			let report = if game.as_health().halluc.is_active() { "what a trippy feeling" } else { "you feel confused" };
			PotionEffect::report(report)
		}
		PotionKind::Levitation => {
			let amount = game.roll_range(15..=30);
			let health = game.as_health_mut();
			health.levitate.extend(amount);
			health.bear_trap = 0;
			health.being_held = false;
			PotionEffect::report("you start to float in the air")
		}
		PotionKind::HasteSelf => {
			let amount = game.roll_range(11..=21);
			let health = game.as_health_mut();
			health.haste_self.extend(amount);
			health.haste_self.ensure_half_active();
			PotionEffect::report("you feel yourself moving much faster")
		}
		PotionKind::SeeInvisible => {
			let mut reports = vec![format!("hmm, this potion tastes like {} juice", game.as_settings().fruit.trim())];
			if game.as_health().blind.is_active() {
				// TODO crate::r#use::unblind(game);
				let unblind_report = "".to_string(); // TODO Get report from unblind"
				reports.push(unblind_report)
			}
			game.set_see_invisible(true);
			// TODO Make sure room and . crate::r#use::relight(game);
			PotionEffect::Report(reports)
		}
	}
}

fn potion_heal(extra: bool, game: &mut GameState) {
	game.as_fighter_mut().hp_current += game.as_fighter().exp.level as isize;
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
	if game.player.health.blind.is_active() {
		crate::r#use::unblind(game);
	}
	if game.player.health.confused.is_active() {
		if extra {
			crate::r#use::unconfuse(game);
		} else {
			game.player.health.confused.halve();
		}
	}
	if game.player.health.halluc.is_active() {
		if extra {
			crate::r#use::unhallucinate(game);
		} else {
			game.player.health.halluc.halve();
		}
	}
}
