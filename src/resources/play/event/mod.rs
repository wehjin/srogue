use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::PartyType;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::one_move::OneMove;
use crate::resources::play::event::pick_up::PickUp;
use crate::resources::play::event::reg_move::RegMove;
use crate::resources::play::seed::StepSeed;
use crate::resources::play::state::RunState;
use message::Message;
use rand::Rng;
use state_action::StateAction;

pub mod message;
pub mod one_move;
pub mod pick_up;
pub mod reg_move;
pub mod state_action;

#[derive(Debug)]
pub enum RunEvent {
	Init,
	Message(Message),
	PlayerQuit(RunState),
	OneMove(OneMove),
	RegisterMove(RegMove),
	PickUp(PickUp),

	PlayerCloseModal(RunState),
	PlayerOpenHelp(RunState),
	PlayerOpenInventory(RunState),
	PrintNextAndStep(RunState, StepSeed),
}

impl RunEvent {
	pub fn dispatch<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		match self {
			RunEvent::Init => init(ctx.rng()),

			RunEvent::Message(message) => message.dispatch(ctx),
			RunEvent::OneMove(one_move) => one_move.dispatch(ctx),
			RunEvent::RegisterMove(reg_move) => reg_move.dispatch(ctx),
			RunEvent::PickUp(pickup) => pickup.dispatch(ctx),

			RunEvent::PlayerQuit(state) => player_quit(state),
			RunEvent::PlayerCloseModal(state) => player_close_modal(state),
			RunEvent::PlayerOpenHelp(state) => player_open_help(state),
			RunEvent::PlayerOpenInventory(state) => player_open_inventory(state),
			RunEvent::PrintNextAndStep(mut state, step_seed) => {
				state.diary.message_line = state.diary.next_message_line.take();
				step_seed.create_step(state)
			}
		}
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

fn init(rng: &mut impl Rng) -> RunStep {
	let state = RunState::init(rng);
	RunStep::Effect(state, RunEffect::AwaitMove)
}

pub enum RunStep {
	Exit(RunState),
	Redirect(RunEvent),
	Effect(RunState, RunEffect),
}
fn _descend(state: RunState, rng: &mut impl Rng) -> RunStep {
	let RunState { mut stats, level, visor, diary, settings } = state;
	let party_type = if stats.is_party_depth(&level.rogue.depth) {
		stats.party_depth = stats.party_depth.roll_next(&level.rogue.depth, rng);
		PartyType::PartyRollBig
	} else {
		PartyType::NoParty
	};
	let level = roll_level(party_type, level.rogue, &mut stats, rng);
	let state = RunState { stats, level, visor, diary, settings };
	RunStep::Exit(state)
}