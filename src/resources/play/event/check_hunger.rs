use crate::init::Dungeon;
use crate::motion::YOU_CAN_MOVE_AGAIN;
use crate::prelude::ending::Ending;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::message::Message;
use crate::resources::play::event::move_monsters::MoveMonstersEvent;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::seed::event_seed::EventSeed;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;
use crate::score::killed_by;
use rand::Rng;

impl GameEventVariant for CheckHungerEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::CheckHunger(self) }
}

#[derive(Debug)]
pub enum CheckHungerEvent {
	Start { after_check: EventSeed },
	InspectEnergy { after_check: EventSeed },
	EnactFaint { duration: Option<usize>, after_check: EventSeed },
}

impl CheckHungerEvent {
	pub fn new(after_check: impl FnOnce(RunState) -> RunEvent + 'static) -> Self {
		Self::Start { after_check: EventSeed::new(after_check) }
	}
}

impl Dispatch for CheckHungerEvent {
	fn dispatch(self, mut state: RunState, _ctx: &mut RunContext) -> RunStep {
		// Update the rogue's energy state.
		match self {
			Self::Start { after_check } => {
				let calorie_burn = get_rogue_calorie_burn(&state);
				match calorie_burn == 0 {
					true => after_check.into_redirect(state),
					false => {
						let old_energy = state.rogue_energy();
						state.as_fighter_mut().moves_left -= calorie_burn;
						let energy = state.rogue_energy();
						match energy == old_energy {
							true => {
								// Go directly to performing energy effects.
								Self::InspectEnergy { after_check }.into_redirect(state)
							}
							false => {
								// Send a report before performing energy effects.
								let diary = state.as_diary_mut();
								diary.stats_changed = true;
								let report = energy.as_report();
								let interrupt = match energy {
									RogueEnergy::Normal | RogueEnergy::Hungry => false,
									RogueEnergy::Weak | RogueEnergy::Faint | RogueEnergy::Starved => true,
								};
								let post_report = |state| {
									// After reporting new energy, perform any effects resulting
									// from the current energy level.
									Self::InspectEnergy { after_check }.into_redirect(state)
								};
								Message::new(state, report, interrupt, post_report).into_redirect()
							}
						}
					}
				}
			}
			Self::InspectEnergy { after_check } => match state.rogue_energy() {
				RogueEnergy::Normal |
				RogueEnergy::Hungry |
				RogueEnergy::Weak => {
					// No energy effects. Do whatever comes next.
					after_check.into_redirect(state)
				}
				RogueEnergy::Faint => {
					// Rogue may experience fainting.
					Self::EnactFaint { duration: None, after_check }.into_redirect(state)
				}
				RogueEnergy::Starved => {
					// Rogue already defeated.
					killed_by(Ending::Starvation, &mut state);
					after_check.into_redirect(state)
				}
			},
			Self::EnactFaint { duration, after_check } => match duration {
				None => {
					assert_eq!(RogueEnergy::Faint, state.rogue_energy());
					let duration = roll_faint_duration(&mut state);
					if 0 == duration {
						// No faint. Do whatever comes next.
						after_check.into_redirect(state)
					} else {
						// Small chance of energy recovery.
						if state.roll_chance(40) {
							state.as_fighter_mut().moves_left += 1;
						}
						// Report and implement the faint.
						let report = "you faint";
						let post_report = move |state| {
							// After reporting the faint, execute [duration] rounds of fainting.
							Self::EnactFaint { duration: Some(duration), after_check }.into_redirect(state)
						};
						Message::new(state, report, true, post_report).into_redirect()
					}
				}
				Some(duration) => if duration == 0 {
					// Fainting spell is over.
					let report = YOU_CAN_MOVE_AGAIN;
					let post_report = |state| {
						// After report our rogue's recovery, Do whatever comes next.
						after_check.into_redirect(state)
					};
					Message::new(state, report, true, post_report).into_redirect()
				} else {
					// Fainting continues. Toss for monster movement.
					let coin_toss = state.rng.gen_bool(0.5);
					if coin_toss {
						// Monsters won the toss. Let them go wild this round.
						let after_move = move |state: RunState| {
							// After monsters move, execute the next fainting round unless the rogue was defeated.
							match state.cleaned_up() {
								None => {
									// Rogue still standing. Execute the next round.
									Self::EnactFaint { duration: Some(duration - 1), after_check }.into_redirect(state)
								}
								Some(_) => {
									// Rogue defeated while fainted. Do whatever comes next.
									after_check.into_redirect(state)
								}
							}
						};
						MoveMonstersEvent::new(after_move).into_redirect(state)
					} else {
						// Rogue won the toss so skip this round.
						Self::EnactFaint { duration: Some(duration - 1), after_check }.into_redirect(state)
					}
				},
			},
		}
	}
}

fn roll_faint_duration(state: &mut RunState) -> usize {
	let max_duration = (RogueEnergy::MAX_FAINT - state.as_fighter().moves_left).max(0) as usize;
	let duration = state.rng.gen_range(0..=max_duration);
	duration
}

fn get_rogue_calorie_burn(game: &RunState) -> isize {
	let calorie_burn = match game.as_ring_effects().calorie_burn() {
		-2 => 0,
		-1 => game.as_fighter().moves_left % 2,
		0 => 1,
		1 => 1 + (game.as_fighter().moves_left % 2),
		2 => 2,
		_ => panic!("invalid calorie burn")
	};
	calorie_burn
}