use std::collections::HashMap;

use lazy_static::lazy_static;

use keyboard::CTRL_P;

use crate::actions::drop_item::DropItem;
use crate::actions::eat::Eat;
use crate::actions::fight::{FightHeavy, FightLight};
use crate::actions::instruct::Instruct;
use crate::actions::inventory::Inventory;
use crate::actions::motion::{MoveMultiple, MoveOnce};
use crate::actions::move_onto::MoveOnto;
use crate::actions::PlayerAction;
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::quaff::Quaff;
use crate::actions::read_scroll::ReadScroll;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::rest::Rest;
use crate::actions::screen::ReMessage;
use crate::actions::search::Search;
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::actions::wizard::Wizardize;
use crate::init::GameState;
use crate::resources::keyboard;
use crate::resources::keyboard::CTRL_W;

const ROGUE_ACTIONS: [(&[char], fn(char, &mut GameState)); 21] = [
	(&['d'], DropItem::update),
	(&['e'], Eat::update),
	(&['F'], FightHeavy::update),
	(&['f'], FightLight::update),
	(&['?'], Instruct::update),
	(&['i'], Inventory::update),
	(&SHIFT_MOTION_KEYS, MoveMultiple::update),
	(&CTRL_MOTION_KEYS, MoveMultiple::update),
	(&MOTION_KEYS, MoveOnce::update),
	(&['m'], MoveOnto::update),
	(&['P'], PutOnRing::update),
	(&['q'], Quaff::update),
	(&['r'], ReadScroll::update),
	(&[CTRL_P], ReMessage::update),
	(&['R'], RemoveRing::update),
	(&['.'], Rest::update),
	(&['s'], Search::update),
	(&['T'], TakeOff::update),
	(&['W'], Wear::update),
	(&['w'], Wield::update),
	(&[CTRL_W], Wizardize::update),
];

const MOTION_KEYS: [char; 8] = ['h', 'j', 'k', 'l', 'y', 'u', 'n', 'b'];
const SHIFT_MOTION_KEYS: [char; 8] = ['H', 'J', 'K', 'L', 'B', 'Y', 'U', 'N'];
const CTRL_MOTION_KEYS: [char; 8] = [
	keyboard::CTRL_H, keyboard::CTRL_J, keyboard::CTRL_K, keyboard::CTRL_L,
	keyboard::CTRL_Y, keyboard::CTRL_U, keyboard::CTRL_N, keyboard::CTRL_B
];

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
