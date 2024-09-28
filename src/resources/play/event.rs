use crate::motion::{dispatch_move_event, MoveDirection, MoveEffect, MoveEvent};
use crate::resources::dungeon::DungeonVisor;
use crate::resources::level::setup::roll_level;
use crate::resources::level::PartyType;
use crate::resources::play::effect::RunEffect;
use crate::resources::play::state::RunState;
use rand::Rng;

pub enum RunEvent {
	Init,
	PlayerQuit(RunState),
	PlayerCloseModal(RunState),
	PlayerOpenHelp(RunState),
	PlayerOpenInventory(RunState),
	PlayerMove(RunState, MoveDirection),
}
impl RunEvent {
	pub fn dispatch(self, rng: &mut impl Rng) -> RunStep {
		match self {
			RunEvent::Init => init(rng),
			RunEvent::PlayerQuit(state) => player_quit(state),
			RunEvent::PlayerCloseModal(state) => player_close_modal(state),
			RunEvent::PlayerOpenHelp(state) => player_open_help(state),
			RunEvent::PlayerOpenInventory(state) => player_open_inventory(state),
			RunEvent::PlayerMove(state, direction) => player_move(direction, state, rng),
		}
	}
}

fn player_move(direction: MoveDirection, mut state: RunState, rng: &mut impl Rng) -> RunStep {
	let mut next_event = Some(MoveEvent::Start { direction, pickup: true });
	while let Some(event) = next_event.take() {
		match dispatch_move_event(event, &mut state, rng) {
			MoveEffect::Fail { consume_time } => {
				if consume_time {
					unimplemented!()
				} else {
					// TODO Deal with diary.
					let step = RunStep { state, effect: RunEffect::AwaitPlayerMove };
					return step;
				}
			}
			MoveEffect::PrepWithinRoom { row, col, rogue_row, rogue_col } => {
				next_event = Some(MoveEvent::Continue { row, col, rogue_row, rogue_col })
			}
			MoveEffect::Done { .. } => {
				let step = RunStep { state, effect: RunEffect::AwaitPlayerMove };
				return step;
			}
			_ => unimplemented!(),
		}
	}
	RunStep { state, effect: RunEffect::Exit }
}

fn player_open_inventory(mut state: RunState) -> RunStep {
	state.visor = DungeonVisor::Inventory;
	RunStep { state, effect: RunEffect::AwaitModalClose }
}

fn player_open_help(mut state: RunState) -> RunStep {
	state.visor = DungeonVisor::Help;
	RunStep { state, effect: RunEffect::AwaitModalClose }
}

fn player_close_modal(mut state: RunState) -> RunStep {
	state.visor = DungeonVisor::Map;
	RunStep { state, effect: RunEffect::AwaitPlayerMove }
}

fn player_quit(state: RunState) -> RunStep {
	RunStep { state, effect: RunEffect::Exit }
}

fn init(rng: &mut impl Rng) -> RunStep {
	let state = RunState::init(rng);
	RunStep { state, effect: RunEffect::AwaitPlayerMove }
}
pub struct RunStep {
	pub state: RunState,
	pub effect: RunEffect,
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
	RunStep { state, effect: RunEffect::Exit }
}