use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::setup::roll_level;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::resources::player::{InputMode, PlayerInput};
use crate::resources::rogue::Rogue;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

pub mod stats;
pub fn run(get_input: impl Fn(InputMode) -> PlayerInput, mut draw_state: impl FnMut(&DungeonState)) {
	let rng = &mut ChaChaRng::from_entropy();
	let mut next_event = Some(DungeonEvent::Init);
	while let Some(event) = next_event.take() {
		let Step { state, effect } = match event {
			DungeonEvent::Init => {
				let state = DungeonState::roll(rng);
				Step { state, effect: Effect::AwaitPlayerMove }
			}
			DungeonEvent::PlayerQuit(state) => {
				Step { state, effect: Effect::Exit }
			}
			DungeonEvent::OpenInstructions(mut state) => {
				state.visor = DungeonVisor::Help;
				Step { state, effect: Effect::AwaitCloseInstructions }
			}
			DungeonEvent::CloseInstructions(mut state) => {
				state.visor = DungeonVisor::Map;
				Step { state, effect: Effect::AwaitPlayerMove }
			}
		};
		draw_state(&state);
		let new_event = match effect {
			Effect::Exit => break,
			Effect::AwaitPlayerMove => match get_input(InputMode::Any) {
				PlayerInput::Close => DungeonEvent::PlayerQuit(state),
				PlayerInput::Help => DungeonEvent::OpenInstructions(state),
			},
			Effect::AwaitCloseInstructions => {
				let _input = get_input(InputMode::Alert);
				DungeonEvent::CloseInstructions(state)
			}
		};
		next_event = Some(new_event);
	}
}

pub enum Effect {
	Exit,
	AwaitPlayerMove,
	AwaitCloseInstructions,
}

pub struct Step {
	pub state: DungeonState,
	pub effect: Effect,
}

pub enum DungeonEvent {
	Init,
	PlayerQuit(DungeonState),
	OpenInstructions(DungeonState),
	CloseInstructions(DungeonState),
}

fn _descend(state: DungeonState, rng: &mut impl Rng) -> Step {
	let DungeonState { mut stats, level, visor } = state;
	let party_type = if stats.is_party_depth(&level.rogue.depth) {
		stats.party_depth = stats.party_depth.roll_next(&level.rogue.depth, rng);
		PartyType::PartyRollBig
	} else {
		PartyType::NoParty
	};
	let level = roll_level(party_type, level.rogue, &mut stats, rng);
	let state = DungeonState { stats, level, visor };
	Step { state, effect: Effect::Exit }
}

pub struct DungeonState {
	pub stats: DungeonStats,
	pub level: DungeonLevel,
	pub visor: DungeonVisor,
}

impl DungeonState {
	pub fn roll(rng: &mut impl Rng) -> Self {
		let mut stats = DungeonStats::new(rng);
		let mut level = roll_level(PartyType::NoParty, Rogue::new(1), &mut stats, rng);
		level.lighting_enabled = true;
		Self { stats, level, visor: DungeonVisor::Map }
	}
}

pub enum DungeonVisor {
	Map,
	Help,
}
