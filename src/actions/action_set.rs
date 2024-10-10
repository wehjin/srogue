use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::actions::action_set::PlayerEvent::{MoveRogue, Update};
use crate::actions::fight::{FightHeavy, FightLight, Throw, Zap};
use crate::actions::ground::KickIntoPack;
use crate::actions::ground::MoveOnto;
use crate::actions::inventory::{InventoryArmor, InventoryGround, InventoryOne, InventoryRings, InventoryWeapons};
use crate::actions::motion::{Ascend, Descend};
use crate::actions::put_on_ring::PutOnRing;
use crate::actions::read_scroll::ReadScroll;
use crate::actions::remove_ring::RemoveRing;
use crate::actions::rest::{CallIt, Ignore, Quit, Rest, SaveGame, ShowAverageHp, Version};
use crate::actions::screen::ReMessage;
use crate::actions::search::{IdentifyTrap, Search};
use crate::actions::take_off::TakeOff;
use crate::actions::wear::Wear;
use crate::actions::wield::Wield;
use crate::actions::wizard::{DrawMagicMap, NewObjectForWizard, ShowMonsters, ShowObjects, ShowTraps, Wizardize};
use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::motion::{MoveDirection, MoveUntil};
use crate::resources::keyboard;
use crate::resources::keyboard::{CTRL_A, CTRL_C, CTRL_I, CTRL_M, CTRL_O, CTRL_S, CTRL_T, CTRL_W};
use crate::systems::play_level::{LevelResult, UNKNOWN_COMMAND};
use keyboard::{CTRL_B, CTRL_H, CTRL_J, CTRL_K, CTRL_L, CTRL_N, CTRL_P, CTRL_U, CTRL_Y};

const ROGUE_ACTIONS: [(&[char], UpdateGameFn); 35] = [
	(&['<'], Ascend::update),
	(&['c'], CallIt::update),
	(&['>'], Descend::update),
	(&[CTRL_S], DrawMagicMap::update),
	(&['F'], FightHeavy::update),
	(&['f'], FightLight::update),
	(&['^'], IdentifyTrap::update),
	(&[' '], Ignore::update),
	(&[']'], InventoryArmor::update),
	(&[CTRL_I], InventoryGround::update),
	(&['I'], InventoryOne::update),
	(&['='], InventoryRings::update),
	(&[')'], InventoryWeapons::update),
	(&[','], KickIntoPack::update),
	(&['m'], MoveOnto::update),
	(&[CTRL_C], NewObjectForWizard::update),
	(&['P'], PutOnRing::update),
	(&['Q'], Quit::update),
	(&['r'], ReadScroll::update),
	(&[CTRL_P], ReMessage::update),
	(&['R'], RemoveRing::update),
	(&['.'], Rest::update),
	(&['S'], SaveGame::update),
	(&['s'], Search::update),
	(&[CTRL_A], ShowAverageHp::update),
	(&[CTRL_O], ShowObjects::update),
	(&[CTRL_M], ShowMonsters::update),
	(&[CTRL_T], ShowTraps::update),
	(&['T'], TakeOff::update),
	(&['t'], Throw::update),
	(&['v'], Version::update),
	(&['W'], Wear::update),
	(&['w'], Wield::update),
	(&[CTRL_W], Wizardize::update),
	(&['z'], Zap::update),
];

lazy_static! {
	static ref ACTION_UPDATES: HashMap<char, UpdateGameFn> = {
		let mut actions = HashMap::new();
		for (key_set, handler) in &ROGUE_ACTIONS {
			for key in *key_set {
				actions.insert(*key, *handler);
			}
		}
		actions
	};
}

pub enum PlayerEvent {
	MoveRogue(MoveDirection, Option<MoveUntil>),
	Update(&'static UpdateGameFn),
}

impl TryFrom<char> for PlayerEvent {
	type Error = anyhow::Error;

	fn try_from(value: char) -> Result<Self, Self::Error> {
		match value {
			'h' | 'j' | 'k' | 'l' | 'y' | 'u' | 'n' | 'b' => {
				Ok(MoveRogue(MoveDirection::from(value), None))
			}
			'H' | 'J' | 'K' | 'L' | 'Y' | 'U' | 'N' | 'B' => {
				Ok(MoveRogue(MoveDirection::from(value), Some(MoveUntil::Obstacle)))
			}
			CTRL_H | CTRL_J | CTRL_K | CTRL_L | CTRL_Y | CTRL_U | CTRL_N | CTRL_B => {
				Ok(MoveRogue(MoveDirection::from(value), Some(MoveUntil::NearSomething)))
			}
			_ => match ACTION_UPDATES.get(&value) {
				None => Err(anyhow::anyhow!("{} {}", UNKNOWN_COMMAND, value)),
				Some(update_game) => Ok(Update(update_game)),
			}
		}
	}
}

pub type UpdateGameFn = fn(&mut GameState) -> Option<LevelResult>;
