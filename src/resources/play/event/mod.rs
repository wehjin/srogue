use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::PartyType;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::game::{Dispatch, GameEvent};
use crate::resources::play::event::pick_up::PickUpRegMove;
use crate::resources::play::seed::step_seed::StepSeed;
use crate::resources::play::state::RunState;
use message::MessageEvent;
use rand_chacha::ChaCha8Rng;
use state_action::StateAction;

pub mod check_hunger;
pub mod game;
pub mod message;
pub mod monster_hit;
pub mod move_monsters;
pub mod move_monster_step;
pub mod one_move;
pub mod pick_up;
pub mod reg_move;
pub mod state_action;
pub mod upgrade_rogue;

#[derive(Debug)]
pub enum RunEvent {
	Init(ChaCha8Rng),
	Game(RunState, GameEvent),
	PlayerQuit(RunState),
	Message(MessageEvent),
	PickUp(PickUpRegMove),

	PlayerCloseModal(RunState),
	PlayerOpenHelp(RunState),
	PlayerOpenInventory(RunState),
	PrintNextAndStep(RunState, StepSeed),
}

impl RunEvent {
	pub fn dispatch(self, ctx: &mut RunContext) -> RunStep {
		match self {
			RunEvent::Init(rng) => init(rng),
			RunEvent::Game(state, game_event) => game_event.dispatch(state, ctx),
			RunEvent::Message(message) => message.dispatch(ctx),

			RunEvent::PickUp(pickup) => pickup.dispatch(ctx),
			RunEvent::PlayerQuit(state) => player_quit(state),
			RunEvent::PlayerCloseModal(state) => player_close_modal(state),
			RunEvent::PlayerOpenHelp(state) => player_open_help(state),
			RunEvent::PlayerOpenInventory(state) => player_open_inventory(state),
			RunEvent::PrintNextAndStep(mut state, step_seed) => {
				state.diary.message_line = state.diary.next_message_line.take();
				step_seed.into_step(state)
			}
		}
	}
}

pub trait RunEventVariant: Dispatch {
	fn into_run_event(self, state: RunState) -> RunEvent;
	fn into_redirect(self, state: RunState) -> RunStep
	where
		Self: Sized,
	{
		RunStep::Redirect(self.into_run_event(state))
	}
}

fn player_open_inventory(mut state: RunState) -> RunStep {
	state.visor = DungeonVisor::Inventory;
	RunStep::Effect(state, RunEffect::AwaitModalClose)
}

fn player_open_help(mut state: RunState) -> RunStep {
	state.visor = DungeonVisor::Help;
	RunStep::Effect(state, RunEffect::AwaitModalClose)
}

fn player_close_modal(mut state: RunState) -> RunStep {
	state.visor = DungeonVisor::Map;
	RunStep::Effect(state, RunEffect::AwaitMove)
}

fn player_quit(state: RunState) -> RunStep {
	RunStep::Exit(state)
}

fn init(rng: ChaCha8Rng) -> RunStep {
	let state = RunState::init(rng);
	RunStep::Effect(state, RunEffect::AwaitMove)
}

#[derive(Debug)]
pub enum RunStep {
	Exit(RunState),
	Redirect(RunEvent),
	Effect(RunState, RunEffect),
}
fn _descend(mut state: RunState) -> RunStep {
	let party_type = if state.stats.is_party_depth(&state.level.rogue.depth) {
		let rogue_depth = state.level.rogue.depth;
		state.stats.party_depth = state.stats.party_depth.roll_next(&rogue_depth, state.rng());
		PartyType::PartyRollBig
	} else {
		PartyType::NoParty
	};
	let (level, stats, rng) = roll_level(party_type, state.level.rogue, state.stats, state.rng);
	state.level = level;
	state.stats = stats;
	state.rng = rng;
	RunStep::Exit(state)
}