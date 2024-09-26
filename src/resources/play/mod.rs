use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::setup::roll_level;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::resources::player::{InputMode, PlayerInput};
use crate::resources::rogue::Rogue;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

pub enum PlayEvent {
	Init,
	PlayerQuit(PlayState),
	PlayerCloseDialog(PlayState),
	PlayerOpenHelp(PlayState),
	PlayerOpenInventory(PlayState),
}

pub enum PlayEffect {
	Exit,
	AwaitPlayerMove,
	AwaitCloseDialog,
}

pub fn run(get_input: impl Fn(InputMode) -> PlayerInput, mut draw_state: impl FnMut(&PlayState)) {
	let rng = &mut ChaChaRng::from_entropy();
	let mut next_event = Some(PlayEvent::Init);
	while let Some(event) = next_event.take() {
		let Step { state, effect } = step(event, rng);
		draw_state(&state);
		let event1 = match effect {
			PlayEffect::Exit => break,
			PlayEffect::AwaitPlayerMove => {
				let input = get_input(InputMode::Any);
				match input {
					PlayerInput::Close => PlayEvent::PlayerQuit(state),
					PlayerInput::Help => PlayEvent::PlayerOpenHelp(state),
					PlayerInput::Menu => PlayEvent::PlayerOpenInventory(state),
				}
			}
			PlayEffect::AwaitCloseDialog => {
				let _input = get_input(InputMode::Alert);
				PlayEvent::PlayerCloseDialog(state)
			}
		};
		next_event = Some(event1);
	}
}

fn step(event: PlayEvent, rng: &mut impl Rng) -> Step {
	let step = match event {
		PlayEvent::Init => {
			Step { state: PlayState::roll(rng), effect: PlayEffect::AwaitPlayerMove }
		}
		PlayEvent::PlayerQuit(state) => {
			Step { state, effect: PlayEffect::Exit }
		}
		PlayEvent::PlayerCloseDialog(mut state) => {
			state.visor = DungeonVisor::Map;
			Step { state, effect: PlayEffect::AwaitPlayerMove }
		}
		PlayEvent::PlayerOpenHelp(mut state) => {
			state.visor = DungeonVisor::Help;
			Step { state, effect: PlayEffect::AwaitCloseDialog }
		}
		PlayEvent::PlayerOpenInventory(mut state) => {
			state.visor = DungeonVisor::Inventory;
			Step { state, effect: PlayEffect::AwaitCloseDialog }
		}
	};
	step
}

pub struct Step {
	pub state: PlayState,
	pub effect: PlayEffect,
}

fn _descend(state: PlayState, rng: &mut impl Rng) -> Step {
	let PlayState { mut stats, level, visor } = state;
	let party_type = if stats.is_party_depth(&level.rogue.depth) {
		stats.party_depth = stats.party_depth.roll_next(&level.rogue.depth, rng);
		PartyType::PartyRollBig
	} else {
		PartyType::NoParty
	};
	let level = roll_level(party_type, level.rogue, &mut stats, rng);
	let state = PlayState { stats, level, visor };
	Step { state, effect: PlayEffect::Exit }
}

pub struct PlayState {
	pub stats: DungeonStats,
	pub level: DungeonLevel,
	pub visor: DungeonVisor,
}

impl PlayState {
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
