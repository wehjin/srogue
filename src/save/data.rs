use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::init::GameState;
use crate::level::Level;
use crate::machdep::{get_current_time, RogueTime};
use crate::monster::MonsterMash;
use crate::objects::ObjectPack;
use crate::player::Player;

pub fn from_file(path: &str) -> Result<SaveData, Box<dyn Error>> {
	let json = fs::read_to_string(path)?;
	let data = serde_json::from_str::<SaveData>(&json)?;
	Ok(data)
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
	pub mash: MonsterMash,
	pub player: Player,
	pub level: Level,
	pub ground: ObjectPack,
	pub file_id: i64,
	pub saved_time: RogueTime,
}

impl SaveData {
	pub unsafe fn read_from_statics(file_id: i64, game: &GameState) -> Self {
		SaveData {
			mash: game.mash.clone(),
			player: game.player.clone(),
			ground: game.ground.clone(),
			level: game.level.clone(),
			file_id,
			saved_time: get_current_time().add_seconds(10),
		}
	}
	pub unsafe fn write_to_statics(&self) {}
}
