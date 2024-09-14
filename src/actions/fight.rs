use crate::actions::GameUpdater;
use crate::hit::fight;
use crate::init::GameState;
use crate::systems::play_level::LevelResult;
use crate::throw::throw;
use crate::zap::zapp;

pub struct Throw;

impl GameUpdater for Throw {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		throw(game);
		None
	}
}

pub struct Zap;

impl GameUpdater for Zap {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		zapp(game);
		None
	}
}

pub struct FightLight;

impl GameUpdater for FightLight {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		fight(false, game);
		None
	}
}

pub struct FightHeavy;

impl GameUpdater for FightHeavy {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		fight(true, game);
		None
	}
}