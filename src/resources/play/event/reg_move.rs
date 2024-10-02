use crate::init::Dungeon;
use crate::monster::{mv_mons, put_wanderer};
use crate::motion::MoveResult;
use crate::prelude::ending::Ending;
use crate::resources::arena::Arena;
use crate::resources::avatar::Avatar;
use crate::resources::cofx::random_faint::RandomFaint;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::message::Message;
use crate::resources::play::event::state_action::StateAction;
use crate::resources::play::event::{RunEvent, RunStep};
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;
use crate::score::killed_by;

#[derive(Debug, Clone)]
pub struct RegMove(pub RunState);

impl StateAction for RegMove {
	fn into_event(self) -> RunEvent {
		RunEvent::RegisterMove(self)
	}

	fn dispatch(self, ctx: &mut RunContext) -> RunStep {
		let Self(mut state) = self;
		let old_energy = state.rogue_energy();
		state = reg_move(state, old_energy, ctx);

		let new_energy = state.rogue_energy();
		if RogueEnergy::Starved == new_energy {
			RunStep::Exit(state)
		} else {
			RunStep::Effect(state, RunEffect::AwaitMove)
		}
	}
}


/// Sets [game.rogue_energy] to some value of [RogueEnergy] before returning.
fn reg_move(mut game: RunState, old_energy: RogueEnergy, ctx: &mut RunContext) -> RunState {
	// Take care of hunger.
	if game.is_max_depth() || game.as_fighter().moves_left <= RogueEnergy::MAX_HUNGRY {
		check_hunger_etc(game, old_energy, ctx)
	} else {
		move_monsters_etc(game, old_energy, ctx)
	}
}

fn check_hunger_etc(mut game: RunState, old_energy: RogueEnergy, ctx: &mut RunContext) -> RunState {
	game = check_hunger(game, ctx);
	if RogueEnergy::Starved == game.rogue_energy() {
		update_move_result(game, old_energy)
	} else {
		move_monsters_etc(game, old_energy, ctx)
	}
}

fn move_monsters_etc(mut game: RunState, old_energy: RogueEnergy, ctx: &mut RunContext) -> RunState {
	// Move monsters.
	game = mv_mons(game, ctx);
	game = update_wanderers(game);
	game = update_health(game);
	game = update_move_result(game, old_energy);
	game
}

fn update_move_result(mut state: RunState, old_energy: RogueEnergy) -> RunState {
	state.move_result = match state.move_result {
		None => {
			let energy = state.rogue_energy();
			let energy_changed = energy != old_energy;
			let starved = energy == RogueEnergy::Starved;
			let confused = state.as_health().confused.is_active();
			let interrupted = state.diary.interrupted;
			if energy_changed || starved || confused || interrupted {
				Some(MoveResult::StoppedOnSomething)
			} else {
				Some(MoveResult::Moved)
			}
		}
		Some(value) => Some(value)
	};
	state
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