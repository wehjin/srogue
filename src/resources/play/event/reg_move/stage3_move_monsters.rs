use crate::init::Dungeon;
use crate::monster;
use crate::monster::put_wanderer;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
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
	for mon_id in get_monster_ids_for_movement(&state) {
		if state.cleaned_up().is_some() {
			break;
		}
		state = mv_one_mon(mon_id, state, ctx);
	}
	state
}

pub fn mv_one_mon(mon_id: u64, mut game: RunState, ctx: &mut RunContext) -> RunState {
	let mut done_with_monster = false;
	if game.as_monster(mon_id).is_hasted() {
		let rogue_row = game.rogue_row();
		let rogue_col = game.rogue_col();
		game = monster::mv_monster(game, mon_id, rogue_row, rogue_col, false, ctx);
		if game.get_monster(mon_id).is_none() {
			done_with_monster = true;
		}
	} else if game.as_monster(mon_id).is_slowed() {
		game.as_monster_mut(mon_id).flip_slowed_toggle();
		if game.as_monster(mon_id).is_slowed_toggle() {
			done_with_monster = true;
		}
	}
	if !done_with_monster && game.as_monster(mon_id).is_confused() {
		if monster::move_confused(mon_id, false, &mut game) {
			done_with_monster = true;
		}
	}
	if !done_with_monster {
		let mut flew = false;
		let rogue_row = game.rogue_row();
		let rogue_col = game.rogue_col();

		if game.as_monster(mon_id).flies()
			&& !game.as_monster(mon_id).is_napping()
			&& !monster::mon_can_go_and_reach(mon_id, rogue_row, rogue_col, false, &game) {
			flew = true;
			game = monster::mv_monster(game, mon_id, rogue_row, rogue_col, false, ctx);
		}
		if !(flew && monster::mon_can_go_and_reach(mon_id, rogue_row, rogue_col, false, &game)) {
			game = monster::mv_monster(game, mon_id, rogue_row, rogue_col, false, ctx);
		}
	}
	game
}

pub fn get_monster_ids_for_movement(state: &RunState) -> Vec<u64> {
	if state.cleaned_up().is_some() || state.as_health().haste_self.is_half_active() {
		vec![]
	} else {
		state.monster_ids()
	}
}
