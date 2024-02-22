use lazy_static::lazy_static;

use crate::actions::action_set::PlayerActionSet;
use crate::actions::eat::Eat;
use crate::actions::fight::{FightHeavy, FightLight};
use crate::actions::instruct::Instruct;
use crate::actions::inventory::Inventory;
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::quaff::Quaff;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::rest::Rest;
use crate::actions::search::Search;
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::init::GameState;

pub mod action_set;
pub mod eat;
pub mod fight;
pub mod instruct;
pub mod inventory;
pub mod put_on_ring;
pub mod quaff;
pub mod remove_ring;
pub mod rest;
pub mod search;
pub mod take_off;
pub mod wear;
pub mod wield;

pub trait PlayerAction {
	fn commit(&self, game: &mut GameState);
}

lazy_static! {
	pub static ref PLAYER_ACTIONS: PlayerActionSet = PlayerActionSet::new(vec![
		('?', Box::new(Instruct)),
		('.', Box::new(Rest)),
		('F', Box::new(FightHeavy)),
		('P', Box::new(PutOnRing)),
		('R', Box::new(RemoveRing)),
		('T', Box::new(TakeOff)),
		('W', Box::new(Wear)),
		('e' ,Box::new(Eat)),
		('f', Box::new(FightLight)),
		('i', Box::new(Inventory)),
		('q' ,Box::new(Quaff)),
		('s', Box::new(Search)),
		('w', Box::new(Wield)),
	]);
}