use crate::init::Dungeon;
use crate::motion::YOU_CAN_MOVE_AGAIN;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::message::Message;
use crate::resources::play::event::reg_move::stage3_move_monsters::mv_mons;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;
use rand::Rng;
use FaintState::{End, MoveMonsters, Start};

pub struct RandomFaint;

impl RandomFaint {
	pub fn run(game: RunState, ctx: &mut RunContext) -> RunState {
		let mut current = Start(game);
		loop {
			let next = step(current, ctx);
			if let End(state) = next {
				return state;
			}
			current = next
		}
	}
}
enum FaintState {
	Start(RunState),
	MoveMonsters(RunState, usize),
	End(RunState),
}

fn step(state: FaintState, ctx: &mut RunContext) -> FaintState {
	match state {
		Start(mut state) => {
			let rogue_energy = state.rogue_energy();
			if rogue_energy != RogueEnergy::Faint {
				End(state)
			} else {
				let max_duration = (RogueEnergy::MAX_FAINT - state.as_fighter().moves_left).max(0) as usize;
				let duration = state.rng.gen_range(0..=max_duration);
				if duration == 0 {
					return End(state);
				}
				if state.roll_chance(40) {
					state.as_fighter_mut().moves_left += 1;
				}
				state = Message::run_await_exit(state, "you faint", true, ctx);
				MoveMonsters(state, duration)
			}
		}
		MoveMonsters(mut state, duration) => {
			if duration <= 0 {
				state = Message::run_await_exit(state, YOU_CAN_MOVE_AGAIN, true, ctx);
				End(state)
			} else {
				let coin_toss = state.rng.gen_bool(0.5);
				if coin_toss {
					state = mv_mons(state, ctx);
					if state.cleaned_up().is_some() {
						return End(state);
					}
				}
				MoveMonsters(state, duration - 1)
			}
		}
		End(_) => {
			panic!("Do not step the end state!")
		}
	}
}
