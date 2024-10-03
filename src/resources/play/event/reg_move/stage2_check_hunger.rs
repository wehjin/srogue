use crate::init::Dungeon;
use crate::prelude::ending::Ending;
use crate::resources::avatar::Avatar;
use crate::resources::cofx::random_faint::RandomFaint;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::game::{Dispatch, GameEventVariant};
use crate::resources::play::event::message::Message;
use crate::resources::play::event::reg_move::stage::RegMoveStage;
use crate::resources::play::event::reg_move::stage5_update_move_result::Stage5UpdateMoveResult;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::RunStep;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;
use crate::score::killed_by;

#[derive(Debug)]
pub(super) struct Stage2CheckHunger {
	old_energy: RogueEnergy,
}
impl RegMoveStage for Stage2CheckHunger {
	fn into_reg_move_event(self) -> RegMoveEvent { RegMoveEvent::CheckHunger(self) }
}
impl Stage2CheckHunger {
	pub fn new(old_energy: RogueEnergy) -> Self { Self { old_energy } }
}
impl Dispatch for Stage2CheckHunger {
	fn dispatch(self, mut state: RunState, ctx: &mut RunContext) -> RunStep {
		let Self { old_energy } = self;
		state = check_hunger(state, ctx);
		if RogueEnergy::Starved == state.rogue_energy() {
			Stage5UpdateMoveResult::new(old_energy).into_redirect(state)
		} else {
			RegMoveEvent::Stage3MoveMonsters { old_energy, mon_ids: None }.into_redirect(state)
		}
	}
}

fn check_hunger(mut game: RunState, ctx: &mut RunContext) -> RunState {
	let calorie_burn = match game.as_ring_effects().calorie_burn() {
		-2 => 0,
		-1 => game.as_fighter().moves_left % 2,
		0 => 1,
		1 => 1 + (game.as_fighter().moves_left % 2),
		2 => 2,
		_ => panic!("invalid calorie burn")
	};
	if calorie_burn == 0 {
		// No change
		return game;
	}

	let old_energy = game.rogue_energy();
	game.as_fighter_mut().moves_left -= calorie_burn;

	let new_energy = game.rogue_energy();
	if new_energy != old_energy {
		let diary = game.as_diary_mut();
		diary.stats_changed = true;

		let report = new_energy.as_report();
		let interrupt = match new_energy {
			RogueEnergy::Normal | RogueEnergy::Hungry => false,
			RogueEnergy::Weak | RogueEnergy::Faint | RogueEnergy::Starved => true,
		};
		game = Message::run_await_exit(game, report, interrupt, ctx);
	}
	match new_energy {
		RogueEnergy::Normal | RogueEnergy::Hungry | RogueEnergy::Weak => game,
		RogueEnergy::Faint => RandomFaint::run(game, ctx),
		RogueEnergy::Starved => {
			killed_by(Ending::Starvation, &mut game);
			game
		}
	}
}