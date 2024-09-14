use crate::actions::PlayerAction;
use crate::hit::fight;
use crate::init::GameState;
use crate::systems::play_level::LevelResult;
use crate::throw::throw;
use crate::zap::zapp;

pub struct Throw;

impl PlayerAction for Throw {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		throw(game);
		None
	}
}

pub struct Zap;

impl PlayerAction for Zap {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		zapp(game);
		None
	}
}

pub struct FightLight;

impl PlayerAction for FightLight {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		fight(false, game);
		None
	}
}

pub struct FightHeavy;

impl PlayerAction for FightHeavy {
	fn update(_input_key: char, game: &mut GameState) -> Option<LevelResult> {
		fight(true, game);
		None
	}
}