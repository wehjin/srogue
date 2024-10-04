use crate::init::Dungeon;
use crate::monster::put_wanderer;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::move_monsters;
use crate::resources::play::state::RunState;

pub fn update_wanderers(mut game: RunState) -> RunState {
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

pub fn mv_mons(mut state: RunState, ctx: &mut RunContext) -> RunState {
	for mon_id in move_monsters::get_monster_ids_for_movement(&state) {
		if state.cleaned_up().is_some() {
			break;
		}
		state = move_monsters::mv_one_mon(mon_id, state, ctx);
	}
	state
}

