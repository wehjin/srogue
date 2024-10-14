use crate::hit;
use crate::hit::DamageEffect;
use crate::init::Dungeon;
use crate::monster::mon_name;
use crate::objects::get_armor_class;
use crate::prelude::AMULET_LEVEL;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::MessageEvent;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::step_seed::StepSeed;
use crate::resources::play::state::RunState;
use crate::spec_hit::special_hit;

impl GameEventVariant for MonsterHitEvent {
	fn into_game_event(self) -> GameEvent {
		GameEvent::MonsterHit(self)
	}
}

#[derive(Debug)]
pub enum MonsterHitEvent {
	Start { mon_id: u64, flame: Option<&'static str>, after: StepSeed },
	ReportHit { mon_id: u64, flame: Option<&'static str>, after: StepSeed },
	Hit { mon_id: u64, flame: Option<&'static str>, after: StepSeed },
}

impl MonsterHitEvent {
	pub fn new(mon_id: u64, flame: Option<&'static str>, after: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self::Start {
			mon_id,
			flame,
			after: StepSeed::new("monster-hit", after),
		}
	}
}


impl Dispatch for MonsterHitEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		match self {
			Self::Start { mon_id, flame, after } => {
				if let Some(fight_id) = state.fight_to_death() {
					if mon_id != fight_id {
						state.set_fight_to_death(None);
					}
				}
				state.as_monster_mut(mon_id).clear_target_reset_stuck();
				let hit_chance = hit_chance(mon_id, flame, &state);
				if state.fight_to_death().is_none() {
					let diary = state.as_diary_mut();
					diary.interrupted = true;
				}
				let no_hit = !state.roll_chance(hit_chance);
				if no_hit {
					if state.fight_to_death().is_none() {
						let monster_name = monster_name(mon_id, flame, &state);
						let monster_miss = format!("the {} misses", monster_name);
						let fight_report = fight_report(monster_miss, &mut state);
						MessageEvent::new(
							state,
							fight_report,
							true,
							move |state| after.into_step(state),
						).into_redirect()
					} else {
						after.into_step(state)
					}
				} else {
					Self::ReportHit { mon_id, flame, after }.into_redirect(state)
				}
			}
			Self::ReportHit { mon_id, flame, after } => {
				if state.fight_to_death().is_none() {
					let monster_name = monster_name(mon_id, flame, &state);
					let monster_hit = format!("the {} hit", monster_name);
					let fight_report = fight_report(monster_hit, &mut state);
					MessageEvent::new(
						state,
						fight_report,
						true,
						move |state| Self::Hit { mon_id, flame, after }.into_redirect(state),
					).into_redirect()
				} else {
					Self::Hit { mon_id, flame, after }.into_redirect(state)
				}
			}
			Self::Hit { mon_id, flame, after } => {
				let mut damage: isize = if !state.as_monster_flags(mon_id).stationary {
					let mut damage = hit::get_damage(state.as_monster(mon_id).m_damage(), DamageEffect::Roll);
					if flame.is_some() {
						damage -= get_armor_class(state.armor());
						if damage < 0 {
							damage = 1;
						}
					}
					let rogue_depth = state.rogue_depth();
					let minus: isize = if rogue_depth >= (AMULET_LEVEL * 2) {
						((AMULET_LEVEL * 2) - rogue_depth) as isize
					} else {
						let mut minus = get_armor_class(state.armor()) * 3;
						minus = (minus as f64 / 100.0 * damage as f64) as isize;
						minus
					};
					damage -= minus;
					damage
				} else {
					let original = state.as_monster(mon_id).stationary_damage;
					state.as_monster_mut(mon_id).stationary_damage += 1;
					original
				};
				if state.wizard() {
					damage /= 3;
				}
				if damage > 0 {
					hit::rogue_damage(damage, mon_id, &mut state);
				}

				if state.as_monster_flags(mon_id).special_hit() {
					special_hit(mon_id, &mut state);
				}
				after.into_step(state)
			}
		}
	}
}

fn monster_name(mon_id: u64, flame: Option<&'static str>, state: &RunState) -> &'static str {
	let base_monster_name = mon_name(mon_id, state);
	let monster_name = if let Some(name) = flame { name } else { &base_monster_name };
	monster_name
}

fn hit_chance(mon_id: u64, flame: Option<&str>, state: &RunState) -> usize {
	let mut hit_chance: usize = if state.rogue_depth() >= (AMULET_LEVEL * 2) {
		100
	} else {
		let reduction = (2 * state.buffed_exp()) - state.debuf_exp();
		reduce_chance(state.as_monster(mon_id).m_hit_chance(), reduction)
	};
	if state.wizard() {
		hit_chance /= 2;
	}
	if flame.is_some() {
		hit_chance = reduce_chance(hit_chance, state.buffed_exp() - state.debuf_exp());
	}
	hit_chance
}

pub fn fight_report(monster_message: String, game: &mut impl Dungeon) -> String {
	let player_message = game.as_diary_mut().hit_message.take().unwrap_or_else(String::new);
	if player_message.is_empty() {
		format!("{monster_message}")
	} else {
		format!("{player_message}  {monster_message}")
	}
}

fn reduce_chance(chance: usize, reduction: isize) -> usize {
	let reduction: usize = reduction.max(0) as usize;
	if chance <= reduction { 0 } else { chance - reduction }
}
