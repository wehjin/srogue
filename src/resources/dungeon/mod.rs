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
		let Step { state, effect } = handle_event(event, rng);
		draw_state(&state);
		next_event = Some(match effect {
			Effect::Exit => break,
			Effect::AwaitPlayerMove => {
				let input = get_input(InputMode::Any);
				match input {
					PlayerInput::Close => DungeonEvent::PlayerQuit(state),
					PlayerInput::Help => DungeonEvent::PlayerOpenHelp(state),
					PlayerInput::Menu => DungeonEvent::PlayerOpenInventory(state),
				}
			}
			Effect::AwaitCloseDialog => {
				let _input = get_input(InputMode::Alert);
				DungeonEvent::PlayerCloseDialog(state)
			}
		});
	}
}

fn handle_event(event: DungeonEvent, rng: &mut impl Rng) -> Step {
	let step = match event {
		DungeonEvent::Init => {
			Step { state: DungeonState::roll(rng), effect: Effect::AwaitPlayerMove }
		}
		DungeonEvent::PlayerQuit(state) => {
			Step { state, effect: Effect::Exit }
		}
		DungeonEvent::PlayerCloseDialog(mut state) => {
			state.visor = DungeonVisor::Map;
			Step { state, effect: Effect::AwaitPlayerMove }
		}
		DungeonEvent::PlayerOpenHelp(mut state) => {
			state.visor = DungeonVisor::Help;
			Step { state, effect: Effect::AwaitCloseDialog }
		}
		DungeonEvent::PlayerOpenInventory(mut state) => {
			state.visor = DungeonVisor::Inventory;
			Step { state, effect: Effect::AwaitCloseDialog }
		}
	};
	step
}

pub enum Effect {
	Exit,
	AwaitPlayerMove,
	AwaitCloseDialog,
}

pub struct Step {
	pub state: DungeonState,
	pub effect: Effect,
}

pub enum DungeonEvent {
	Init,
	PlayerQuit(DungeonState),
	PlayerCloseDialog(DungeonState),
	PlayerOpenHelp(DungeonState),
	PlayerOpenInventory(DungeonState),
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
		let rogue = Rogue::new(1).outfit(rng);
		let party_type = PartyType::NoParty;
		let mut level = roll_level(party_type, rogue, &mut stats, rng);
		level.lighting_enabled = true;
		Self { stats, level, visor: DungeonVisor::Map }
	}
}

pub enum DungeonVisor {
	Map,
	Help,
	Inventory,
}
