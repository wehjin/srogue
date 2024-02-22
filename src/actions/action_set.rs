use std::collections::HashMap;

use lazy_static::lazy_static;

use keyboard::CTRL_P;

use crate::actions::drop_item::DropItem;
use crate::actions::eat::Eat;
use crate::actions::fight::{FightHeavy, FightLight, Throw, Zap};
use crate::actions::instruct::Instruct;
use crate::actions::inventory::{Inventory, InventoryArmor, InventoryGround, InventoryOne, InventoryRings, InventoryWeapons};
use crate::actions::motion::{Ascend, Descend, MoveMultiple, MoveOnce};
use crate::actions::move_onto::MoveOnto;
use crate::actions::PlayerAction;
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::quaff::Quaff;
use crate::actions::read_scroll::ReadScroll;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::rest::{CallIt, Ignore, Quit, Rest, ShowAverageHp, Version};
use crate::actions::screen::ReMessage;
use crate::actions::search::{IdentifyTrap, Search};
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::actions::wizard::{DrawMagicMap, NewObjectForWizard, ShowObjects, ShowTraps, Wizardize};
use crate::init::GameState;
use crate::resources::keyboard;
use crate::resources::keyboard::{CTRL_A, CTRL_C, CTRL_I, CTRL_O, CTRL_S, CTRL_T, CTRL_W};
use crate::systems::play_level::PlayResult;

const ROGUE_ACTIONS: [(&[char], fn(char, &mut GameState) -> Option<PlayResult>); 40] = [
	(&['<'], Ascend::update),
	(&['c'], CallIt::update),
	(&['>'], Descend::update),
	(&[CTRL_S], DrawMagicMap::update),
	(&['d'], DropItem::update),
	(&['e'], Eat::update),
	(&['F'], FightHeavy::update),
	(&['f'], FightLight::update),
	(&['^'], IdentifyTrap::update),
	(&[' '], Ignore::update),
	(&['?'], Instruct::update),
	(&['i'], Inventory::update),
	(&[']'], InventoryArmor::update),
	(&[CTRL_I], InventoryGround::update),
	(&['I'], InventoryOne::update),
	(&['='], InventoryRings::update),
	(&[')'], InventoryWeapons::update),
	(&SHIFT_MOTION_KEYS, MoveMultiple::update),
	(&CTRL_MOTION_KEYS, MoveMultiple::update),
	(&MOTION_KEYS, MoveOnce::update),
	(&['m'], MoveOnto::update),
	(&[CTRL_C], NewObjectForWizard::update),
	(&['P'], PutOnRing::update),
	(&['q'], Quaff::update),
	(&['Q'], Quit::update),
	(&['r'], ReadScroll::update),
	(&[CTRL_P], ReMessage::update),
	(&['R'], RemoveRing::update),
	(&['.'], Rest::update),
	(&['s'], Search::update),
	(&[CTRL_A], ShowAverageHp::update),
	(&[CTRL_O], ShowObjects::update),
	(&[CTRL_T], ShowTraps::update),
	(&['T'], TakeOff::update),
	(&['t'], Throw::update),
	(&['v'], Version::update),
	(&['W'], Wear::update),
	(&['w'], Wield::update),
	(&[CTRL_W], Wizardize::update),
	(&['z'], Zap::update),
];

const MOTION_KEYS: [char; 8] = ['h', 'j', 'k', 'l', 'y', 'u', 'n', 'b'];
const SHIFT_MOTION_KEYS: [char; 8] = ['H', 'J', 'K', 'L', 'B', 'Y', 'U', 'N'];
const CTRL_MOTION_KEYS: [char; 8] = [
	keyboard::CTRL_H, keyboard::CTRL_J, keyboard::CTRL_K, keyboard::CTRL_L,
	keyboard::CTRL_Y, keyboard::CTRL_U, keyboard::CTRL_N, keyboard::CTRL_B
];

lazy_static! {
	pub static ref ACTION_UPDATES: HashMap<char, fn(char,&mut GameState) -> Option<PlayResult>> = {
		let mut actions = HashMap::new();
		for (key_set, handler) in &ROGUE_ACTIONS {
			for key in *key_set {
				actions.insert(*key, *handler);
			}
		}
		actions
	};
}
