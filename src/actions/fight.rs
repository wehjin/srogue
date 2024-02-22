use crate::actions::PlayerAction;
use crate::hit::fight;
use crate::init::GameState;

pub struct FightLight;

impl PlayerAction for FightLight {
	fn update(_input_key: char, game: &mut GameState) {
		fight(false, game);
	}
}

pub struct FightHeavy;

impl PlayerAction for FightHeavy {
	fn update(_input_key: char, game: &mut GameState) {
		fight(true, game);
	}
}