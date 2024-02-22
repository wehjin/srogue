use lazy_static::lazy_static;

use crate::actions::action_set::PlayerActionSet;
use crate::actions::instruct::Instruct;
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::rest::Rest;
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::init::GameState;

pub mod put_on_ring;
pub mod remove_ring;
pub mod rest;
pub mod take_off;
pub mod wear;
pub mod wield;
pub mod instruct;
pub mod action_set;

pub trait PlayerAction {
	fn commit(&self, game: &mut GameState);
}

lazy_static! {
	pub static ref PLAYER_ACTIONS: PlayerActionSet = PlayerActionSet::new(vec![
		('P', Box::new(PutOnRing)),
		('R', Box::new(RemoveRing)),
		('T', Box::new(TakeOff)),
		('W', Box::new(Wear)),
		('w', Box::new(Wield)),
		('?', Box::new(Instruct)),
		('.', Box::new(Rest)),
	]);
}