use crate::components::hunger::FAINT_MOVES_LEFT;
use crate::init::Dungeon;
use crate::monster::mv_mons;
use crate::motion::YOU_CAN_MOVE_AGAIN;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::message::Message;
use crate::resources::play::state::RunState;
use rand::Rng;
use FaintState::{End, MoveMonsters, Start};

pub struct RandomFaint {
	pub did_faint: bool,
	pub state: RunState,
}
impl RandomFaint {
	pub fn run(game: RunState, ctx: &mut RunContext) -> Self {
		let mut faint_state = Start(game);
		loop {
			let next_faint = step(faint_state, ctx);
			faint_state = if let End(state, fainted) = next_faint {
				return RandomFaint { did_faint: fainted, state }
			} else {
				next_faint
			};
		}
	}
}
enum FaintState {
	Start(RunState),
	MoveMonsters(RunState, usize),
	End(RunState, bool),
}

fn step(state: FaintState, ctx: &mut RunContext) -> FaintState {
	match state {
		Start(mut state) => {
			let n_max = (FAINT_MOVES_LEFT - state.as_fighter().moves_left).max(0) as usize;
			let n = state.rng.gen_range(0..=n_max);
			if n == 0 {
				return End(state, false);
			}
			if state.roll_chance(40) {
				state.as_fighter_mut().moves_left += 1;
			}
			state = Message::run_await_exit(state, "you faint", true, ctx);
			MoveMonsters(state, n)
		}
		MoveMonsters(mut state, n) => {
			if n > 0 {
				if roll_move_monsters(&mut state) {
					state = mv_mons(state, ctx);
				}
				if state.cleaned_up().is_none() {
					MoveMonsters(state, n - 1)
				} else {
					End(state, true)
				}
			} else {
				state = Message::run_await_exit(state, YOU_CAN_MOVE_AGAIN, true, ctx);
				End(state, true)
			}
		}
		End(_, _) => {
			panic!("Do not step the end state!")
		}
	}
}

fn roll_move_monsters(state: &mut RunState) -> bool {
	state.rng.gen_bool(0.5)
}
