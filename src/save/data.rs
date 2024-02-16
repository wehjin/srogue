use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::init::GameState;
use crate::level::Level;
use crate::machdep::{get_current_time, RogueTime};
use crate::message::hunger_str;
use crate::monster::MonsterMash;
use crate::objects::{foods, ObjectPack};
use crate::player::Player;
use crate::r#move::m_moves;
use crate::zap::wizard;

pub fn from_file(path: &str) -> Result<SaveData, Box<dyn Error>> {
	let json = fs::read_to_string(path)?;
	let data = serde_json::from_str::<SaveData>(&json)?;
	Ok(data)
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
	pub player: Player,
	pub hunger_str: String,
	pub mash: MonsterMash,
	pub ground: ObjectPack,
	pub file_id: i64,
	pub foods: i16,
	pub level: Level,
	pub wizard: bool,
	pub m_moves: i16,
	pub saved_time: RogueTime,
}

impl SaveData {
	pub unsafe fn read_from_statics(file_id: i64, game: &GameState) -> Self {
		SaveData {
			player: game.player.clone(),
			hunger_str: hunger_str.clone(),
			mash: game.mash.clone(),
			ground: game.ground.clone(),
			file_id,
			foods,
			level: game.level.clone(),
			wizard,
			m_moves,
			saved_time: get_current_time().add_seconds(10),
		}
	}
	pub unsafe fn write_to_statics(&self) {
		hunger_str = self.hunger_str.clone();
		foods = self.foods;
		wizard = self.wizard;
		m_moves = self.m_moves;
	}
}
