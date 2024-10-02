use crate::init::Dungeon;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::Dispatch;
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::stage5_update_move_result::Stage5UpdateMoveResult;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;

#[derive(Debug)]
pub(super) struct Stage4UpdateHealth {
	old_energy: RogueEnergy,
}
impl Stage4UpdateHealth {
	pub fn new(old_energy: RogueEnergy) -> Self {
		Self { old_energy }
	}
}

impl RegMoveStage for Stage4UpdateHealth {
	fn into_reg_move_event(self) -> RegMoveEvent {
		RegMoveEvent::UpdateHealth(self)
	}
}

impl Dispatch for Stage4UpdateHealth {
	fn dispatch(self, state: RunState, _ctx: &mut RunContext) -> RunStep {
		let Self { old_energy } = self;
		let state = update_health(state);
		Stage5UpdateMoveResult::new(old_energy).into_redirect(state)
	}
}

fn update_health(mut game: RunState) -> RunState {
	// Take care of hallucinations.
	if game.as_health().halluc.is_active() {
		game.as_health_mut().halluc.decr();
		if game.as_health().halluc.is_active() {
			// TODO show_hallucination(game);
		} else {
			// TODO unhallucinate(game);
		}
	}
	// Take care of blindness.
	if game.as_health().blind.is_active() {
		game.as_health_mut().blind.decr();
		if game.as_health().blind.is_inactive() {
			//TODO unblind(game);
		}
	}
	// Take care of confusion.
	if game.as_health().confused.is_active() {
		game.as_health_mut().confused.decr();
		if game.as_health().confused.is_inactive() {
			// TODO unconfuse(game);
		}
	}
	// Take care of bear traps.
	if game.as_health().bear_trap > 0 {
		game.as_health_mut().bear_trap -= 1;
	}
	// Take care of levitation.
	if game.as_health().levitate.is_active() {
		game.as_health_mut().levitate.decr();
		if game.as_health().levitate.is_inactive() {
			game.interrupt_and_slurp();
			game.as_diary_mut().add_entry("you float gently to the ground");
			let rogue_row = game.rogue_row();
			let rogue_col = game.rogue_col();
			if game.is_any_tunnel_at(rogue_row, rogue_col) {
				// TODO trap_player(rogue_row as usize, rogue_col as usize, game);
			}
		}
	}
	// Take care of haste effect.
	if game.as_health().haste_self.is_active() {
		game.as_health_mut().haste_self.decr();
		if game.as_health().haste_self.is_inactive() {
			game.as_diary_mut().add_entry("you feel yourself slowing down");
		}
	}
	// Take care of healing.
	// TODO game.heal_player();
	// Take care of searching.
	{
		let auto_search = game.as_ring_effects().auto_search();
		if auto_search > 0 {
			// TODO search(SearchKind::Auto { n: auto_search as usize }, game);
		}
	}
	game
}
