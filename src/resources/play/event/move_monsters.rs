use crate::init::Dungeon;
use crate::monster;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEvent, GameEventVariant};
use crate::resources::play::event::RunStep;
use crate::resources::play::seed::StepSeed;
use crate::resources::play::state::RunState;

impl GameEventVariant for MoveMonstersEvent {
	fn into_game_event(self) -> GameEvent { GameEvent::MoveMonsters(self) }
}

#[derive(Debug)]
pub struct MoveMonstersEvent {
	mon_ids: Option<Vec<u64>>,
	after_move: StepSeed,
}

impl MoveMonstersEvent {
	pub fn new(after_move: impl FnOnce(RunState) -> RunStep + 'static) -> Self {
		Self { mon_ids: None, after_move: StepSeed::new("move-monster", after_move) }
	}
}

impl Dispatch for MoveMonstersEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		let Self { mon_ids, after_move } = self;
		match mon_ids {
			None => {
				let mon_ids = get_monster_ids_for_movement(&state);
				Self { mon_ids: Some(mon_ids), after_move }.into_redirect(state)
			}
			Some(mut mon_ids) => match state.cleaned_up().is_some() {
				true => state.into_exit(),
				false => match mon_ids.pop() {
					None => after_move.into_step(state),
					Some(mon_id) => {
						let state = match state.as_monster(mon_id).is_defeated() {
							true => state,
							false => mv_one_mon(mon_id, state, ctx),
						};
						Self { mon_ids: Some(mon_ids), after_move }.into_redirect(state)
					}
				},
			},
		}
	}
}

pub fn get_monster_ids_for_movement(state: &RunState) -> Vec<u64> {
	if state.cleaned_up().is_some() || state.as_health().haste_self.is_half_active() {
		vec![]
	} else {
		state.monster_ids()
	}
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