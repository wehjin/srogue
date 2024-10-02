use crate::init::Dungeon;
use crate::monster::{mv_mons, put_wanderer};
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::Dispatch;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::reg_move::stage4_update_health::Stage4UpdateHealth;
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

#[derive(Debug)]
pub(super) struct Stage3MoveMonsters {
	old_energy: RogueEnergy,
}
impl Stage3MoveMonsters {
	pub fn new(old_energy: RogueEnergy) -> Self { Self { old_energy } }
}
impl RegMoveStage for Stage3MoveMonsters {
	fn into_reg_move_event(self) -> RegMoveEvent { RegMoveEvent::MoveMonsters(self) }
}

impl Dispatch for Stage3MoveMonsters {
	fn dispatch(self, mut state: RunState, ctx: &mut RunContext) -> RunStep {
		let Self { old_energy } = self;
		state = mv_mons(state, ctx);
		state = update_wanderers(state);
		Stage4UpdateHealth::new(old_energy).into_redirect(state)
	}
}

fn update_wanderers(mut game: RunState) -> RunState {
	// Every 120 moves, add a wanderer.
	let next_m_move = game.m_moves() + 1;
	if next_m_move >= 120 {
		*game.m_moves_mut() = 0;
		put_wanderer(&mut game);
	} else {
		*game.m_moves_mut() = next_m_move;
	}
	game
}
