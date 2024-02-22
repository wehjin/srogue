use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::actions::drop_item::DropItem;
use crate::actions::eat::Eat;
use crate::actions::fight::{FightHeavy, FightLight};
use crate::actions::instruct::Instruct;
use crate::actions::inventory::Inventory;
use crate::actions::move_onto::MoveOnto;
use crate::actions::PlayerAction;
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::quaff::Quaff;
use crate::actions::read_scroll::ReadScroll;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::rest::Rest;
use crate::actions::search::Search;
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::init::GameState;

const ROGUE_ACTIONS: [(char, fn(&mut GameState)); 16] = [
	('?', Instruct::update),
	('.', Rest::update),
	('F', FightHeavy::update),
	('P', PutOnRing::update),
	('R', RemoveRing::update),
	('T', TakeOff::update),
	('W', Wear::update),
	('d', DropItem::update),
	('e', Eat::update),
	('f', FightLight::update),
	('i', Inventory::update),
	('m', MoveOnto::update),
	('q', Quaff::update),
	('r', ReadScroll::update),
	('s', Search::update),
	('w', Wield::update),
];

lazy_static! {
	pub static ref ACTION_UPDATES: HashMap<char, fn(&mut GameState)> = {
		let mut actions = HashMap::new();
		for (key_code, handler) in &ROGUE_ACTIONS {
			actions.insert(*key_code, *handler);
		}
		actions
	};
}
