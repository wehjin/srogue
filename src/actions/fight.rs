use crate::actions::PlayerAction;
use crate::hit::fight;
use crate::init::GameState;
use crate::systems::play_level::PlayResult;
use crate::zap::zapp;

pub struct Zap;

impl PlayerAction for Zap {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		zapp(game);
		None
	}
}

pub struct FightLight;

impl PlayerAction for FightLight {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		fight(false, game);
		None
	}
}

pub struct FightHeavy;

impl PlayerAction for FightHeavy {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		fight(true, game);
		None
	}
}