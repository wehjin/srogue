use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::init::GameState;
use crate::level::{cur_room, Level};
use crate::machdep::{get_current_time, RogueTime};
use crate::message::hunger_str;
use crate::monster::{MASH, MonsterMash};
use crate::objects::{foods, level_objects, ObjectPack};
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
	pub level_monsters: MonsterMash,
	pub level_objects: ObjectPack,
	pub file_id: i64,
	pub foods: i16,
	pub cur_room: i64,
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
			level_monsters: MASH.clone(),
			level_objects: level_objects.clone(),
			file_id,
			foods,
			cur_room,
			level: game.level.clone(),
			wizard,
			m_moves,
			saved_time: get_current_time().add_seconds(10),
		}
	}
	pub unsafe fn write_to_statics(&self) {
		hunger_str = self.hunger_str.clone();
		MASH = self.level_monsters.clone();
		level_objects = self.level_objects.clone();
		foods = self.foods;
		cur_room = self.cur_room;
		wizard = self.wizard;
		m_moves = self.m_moves;
	}
}
