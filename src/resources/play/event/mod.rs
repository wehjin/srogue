use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::PartyType;
use crate::resources::play::context::RunContext;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::event::one_move::OneMoveEvent;
use crate::resources::play::event::reg_move::{RegMoveEvent, StepEvent};
use crate::resources::play::seed::EventSeed;
use crate::resources::play::state::RunState;
use message::MessageEvent;
use rand::Rng;

pub mod message;
pub mod one_move;
pub mod reg_move;

#[derive(Debug)]
pub enum RunEvent {
	Init,
	Message(MessageEvent),
	PlayerQuit(RunState),
	OneMove(OneMoveEvent),
	RegisterMove(RegMoveEvent),
	PlayerCloseModal(RunState),
	PlayerOpenHelp(RunState),
	PlayerOpenInventory(RunState),
	PrintNextAndEffect(RunState, RunEffect),
	PrintNextAndRedirect(RunState, EventSeed),
}

impl RunEvent {
	pub fn dispatch<R: Rng>(self, ctx: &mut RunContext<R>) -> RunStep {
		match self {
			RunEvent::Init => init(ctx.rng()),

			RunEvent::Message(message) => message.into_step(),
			RunEvent::OneMove(one_move) => one_move.into_step(ctx),
			RunEvent::RegisterMove(reg_move) => reg_move.step(ctx),

			RunEvent::PlayerQuit(state) => player_quit(state),
			RunEvent::PlayerCloseModal(state) => player_close_modal(state),
			RunEvent::PlayerOpenHelp(state) => player_open_help(state),
			RunEvent::PlayerOpenInventory(state) => player_open_inventory(state),

			RunEvent::PrintNextAndRedirect(mut state, seed) => {
				state.diary.message_line = state.diary.next_message_line.take();
				RunStep::Redirect(seed.into_event(state))
			}
			RunEvent::PrintNextAndEffect(mut state, effect) => {
				state.diary.message_line = state.diary.next_message_line.take();
				RunStep::Effect(state, effect)
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