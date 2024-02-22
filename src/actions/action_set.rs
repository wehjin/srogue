use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::actions::drop_item::DropItem;
use crate::actions::eat::Eat;
use crate::actions::fight::{FightHeavy, FightLight};
use crate::actions::instruct::Instruct;
use crate::actions::inventory::Inventory;
use crate::actions::move_once::MoveOnce;
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

const ROGUE_ACTIONS: [(&[char], fn(char, &mut GameState)); 17] = [
	(&['d'], DropItem::update),
	(&['e'], Eat::update),
	(&['F'], FightHeavy::update),
	(&['f'], FightLight::update),
	(&['?'], Instruct::update),
	(&['i'], Inventory::update),
	(&MOTION_KEYS, MoveOnce::update),
	(&['m'], MoveOnto::update),
	(&['P'], PutOnRing::update),
	(&['q'], Quaff::update),
	(&['r'], ReadScroll::update),
	(&['R'], RemoveRing::update),
	(&['.'], Rest::update),
	(&['s'], Search::update),
	(&['T'], TakeOff::update),
	(&['W'], Wear::update),
	(&['w'], Wield::update),
];

const MOTION_KEYS: [char; 8] = ['h', 'j', 'k', 'l', 'y', 'u', 'n', 'b'];


lazy_static! {
	pub static ref ACTION_UPDATES: HashMap<char, fn(char,&mut GameState)> = {
		let mut actions = HashMap::new();
		for (key_set, handler) in &ROGUE_ACTIONS {
			for key in *key_set {
				actions.insert(*key, *handler);
			}
		}
		actions
	};
}
