use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::setup::roll_level;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::{DungeonLevel, PartyType};
use crate::resources::rogue::Rogue;
use crossterm::event::KeyCode;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

pub mod stats;
pub type PlayerInput = KeyCode;

pub fn run(get_input: impl Fn() -> PlayerInput, mut draw_level: impl FnMut(&DungeonLevel)) {
	let rng = &mut ChaChaRng::from_entropy();
	let mut next_event = Some(DungeonEvent::Init);
	while let Some(event) = next_event.take() {
		let Step { state, effect } = match event {
			DungeonEvent::Init => Step { state: DungeonState::roll(rng), effect: Effect::AwaitPlayerMove },
			DungeonEvent::PlayerQuit(state) => Step { state, effect: Effect::Exit },
		};
		draw_level(&state.level);
		match effect {
			Effect::Exit => break,
			Effect::AwaitPlayerMove => {
				let _ = get_input();
				next_event = Some(DungeonEvent::PlayerQuit(state));
			}
		};
	}
}

pub enum Effect {
	Exit,
	AwaitPlayerMove,
}

pub struct Step {
	pub state: DungeonState,
	pub effect: Effect,
}

pub enum DungeonEvent {
	Init,
	PlayerQuit(DungeonState),
}

impl DungeonEvent {}

fn _descend(state: DungeonState, rng: &mut impl Rng) -> Step {
	let DungeonState { mut stats, level } = state;
	let party_type = if stats.is_party_depth(&level.rogue.depth) {
		stats.party_depth = stats.party_depth.roll_next(&level.rogue.depth, rng);
		PartyType::PartyRollBig
	} else {
		PartyType::NoParty
	};
	let level = roll_level(party_type, level.rogue, &mut stats, rng);
	let state = DungeonState { stats, level };
	Step { state, effect: Effect::Exit }
}

pub struct DungeonState {
	pub stats: DungeonStats,
	pub level: DungeonLevel,
}

impl DungeonState {
	pub fn roll(rng: &mut impl Rng) -> Self {
		let mut stats = DungeonStats::new(rng);
		let mut level = roll_level(PartyType::NoParty, Rogue::new(1), &mut stats, rng);
		level.lighting_enabled = true;
		Self { stats, level }
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub enum RogueSpot {
	#[default]
	None,
	Vault(LevelSpot, RoomId),
	Passage(LevelSpot),
}

impl RogueSpot {
	pub fn is_spot(&self, spot: LevelSpot) -> bool {
		self.try_spot() == Some(&spot)
	}
	pub fn as_spot(&self) -> &LevelSpot {
		self.try_spot().expect("no rogue spot")
	}
	pub fn try_spot(&self) -> Option<&LevelSpot> {
		match self {
			RogueSpot::None => None,
			RogueSpot::Vault(spot, _) => Some(spot),
			RogueSpot::Passage(spot) => Some(spot),
		}
	}
	pub fn is_vault(&self) -> bool {
		self.as_vault().is_some()
	}
	pub fn as_vault(&self) -> Option<&RoomId> {
		match self {
			RogueSpot::None => None,
			RogueSpot::Vault(_, room) => Some(room),
			RogueSpot::Passage(_) => None,
		}
	}
	pub fn is_in_room(&self, value: RoomId) -> bool {
		match self.as_vault() {
			None => false,
			Some(room_id) => *room_id == value,
		}
	}
	pub fn try_room(&self, level: &DungeonLevel) -> Option<RoomId> {
		if let Some(spot) = self.try_spot() {
			level.room_at_spot(*spot)
		} else {
			None
		}
	}
}