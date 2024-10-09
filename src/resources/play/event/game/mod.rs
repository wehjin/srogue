use crate::actions::eat::EatMealEvent;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::check_hunger::CheckHungerEvent;
use crate::resources::play::event::game::drop_item::DropItemEvent;
use crate::resources::play::event::move_monsters::MoveMonstersEvent;
use crate::resources::play::event::one_move::OneMoveEvent;
use crate::resources::play::event::reg_move::RegMoveEvent;
use crate::resources::play::event::upgrade_rogue::UpgradeRogueEvent;
use crate::resources::play::event::{RunEvent, RunEventVariant, RunStep};
use crate::resources::play::state::RunState;
pub mod drop_item;

#[derive(Debug)]
pub enum GameEvent {
	RegMove(RegMoveEvent),
	OneMove(OneMoveEvent),
	MoveMonsters(MoveMonstersEvent),
	CheckHunger(CheckHungerEvent),
	DropItem(DropItemEvent),
	EatMeal(EatMealEvent),
	UpgradeRogue(UpgradeRogueEvent),
}
impl Dispatch for GameEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			GameEvent::CheckHunger(event) => event.dispatch(state, ctx),
			GameEvent::DropItem(event) => event.dispatch(state, ctx),
			GameEvent::MoveMonsters(event) => event.dispatch(state, ctx),
			GameEvent::OneMove(event) => event.dispatch(state, ctx),
			GameEvent::RegMove(event) => event.dispatch(state, ctx),
			GameEvent::EatMeal(event) => event.dispatch(state, ctx),
			GameEvent::UpgradeRogue(event) => event.dispatch(state, ctx),
		}
	}
}
impl RunEventVariant for GameEvent {
	fn into_run_event(self, state: RunState) -> RunEvent {
		RunEvent::Game(state, self)
	}
}

pub trait GameEventVariant: Dispatch {
	fn into_game_event(self) -> GameEvent;
	fn into_run_event(self, state: RunState) -> RunEvent
	where
		Self: Sized,
	{
		self.into_game_event().into_run_event(state)
	}
	fn into_redirect(self, state: RunState) -> RunStep
	where
		Self: Sized,
	{
		self.into_game_event().into_redirect(state)
	}
}

pub trait Dispatch {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep;
}