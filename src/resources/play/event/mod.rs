use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::PartyType;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::one_move::OneMoveEvent;
use crate::resources::play::state::RunState;
use message::MessageEvent;
use rand::Rng;

pub mod message;
pub mod one_move;

#[derive(Debug)]
pub enum RunEvent {
	Init,
	Message(MessageEvent),
	PlayerQuit(RunState),
	PlayerCloseModal(RunState),
	PlayerOpenHelp(RunState),
	PlayerOpenInventory(RunState),
	OneMove(OneMoveEvent),
}

impl RunEvent {
	pub fn dispatch<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		match self {
			RunEvent::Init => init(ctx.rng()),
			RunEvent::Message(message_event) => message_event.into_step(),
			RunEvent::PlayerQuit(state) => player_quit(state),
			RunEvent::PlayerCloseModal(state) => player_close_modal(state),
			RunEvent::PlayerOpenHelp(state) => player_open_help(state),
			RunEvent::PlayerOpenInventory(state) => player_open_inventory(state),
			RunEvent::OneMove(one_move_event) => one_move_event.into_step(ctx),
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
	RunStep::Effect(state, RunEffect::AwaitPlayerMove)
}

fn player_quit(state: RunState) -> RunStep {
	RunStep::Exit(state)
}

fn init(rng: &mut impl Rng) -> RunStep {
	let state = RunState::init(rng);
	RunStep::Effect(state, RunEffect::AwaitPlayerMove)
}

pub enum RunStep {
	Exit(RunState),
	Forward(RunEvent),
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