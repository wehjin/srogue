use crate::actions::eat::EatMealEvent;
use crate::actions::quaff::QuaffPotionEvent;
use crate::resources::play::context::RunContext;
use crate::resources::play::event::check_hunger::CheckHungerEvent;
use crate::resources::play::event::game::drop_item::DropItemEvent;
use crate::resources::play::event::move_monsters::{MoveMonsterFullEvent, MoveMonstersEvent};
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
	MoveMonsterFull(MoveMonsterFullEvent),
	CheckHunger(CheckHungerEvent),
	DropItem(DropItemEvent),
	EatMeal(EatMealEvent),
	QuaffPotion(QuaffPotionEvent),
	UpgradeRogue(UpgradeRogueEvent),
}
impl Dispatch for GameEvent {
	fn dispatch(self, state: RunState, ctx: &mut RunContext) -> RunStep {
		match self {
			Self::CheckHunger(event) => event.dispatch(state, ctx),
			Self::DropItem(event) => event.dispatch(state, ctx),
			Self::MoveMonsters(event) => event.dispatch(state, ctx),
			Self::MoveMonsterFull(event) => event.dispatch(state, ctx),
			Self::OneMove(event) => event.dispatch(state, ctx),
			Self::RegMove(event) => event.dispatch(state, ctx),
			Self::EatMeal(event) => event.dispatch(state, ctx),
			Self::QuaffPotion(event) => event.dispatch(state, ctx),
			Self::UpgradeRogue(event) => event.dispatch(state, ctx),
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