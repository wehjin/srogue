use std::error::Error;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::init::GameState;
use crate::inventory::IS_WOOD;
use crate::level::{cur_room, Level};
use crate::machdep::{get_current_time, RogueTime};
use crate::message::hunger_str;
use crate::monster::{MASH, MonsterMash};
use crate::objects::{foods, id, id_potions, id_rings, id_scrolls, id_wands, level_objects, ObjectPack};
use crate::player::Player;
use crate::r#move::m_moves;
use crate::r#use::{confused};
use crate::settings;
use crate::settings::{login_name, score_only};
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
	pub login_name: String,
	pub level_monsters: MonsterMash,
	pub level_objects: ObjectPack,
	pub file_id: i64,
	pub foods: i16,
	pub id_potions: SaveIdTable,
	pub id_scrolls: SaveIdTable,
	pub id_wands: SaveIdTable,
	pub id_rings: SaveIdTable,
	pub is_wood: Vec<bool>,
	pub cur_room: i64,
	pub level: Level,
	pub confused: usize,
	pub wizard: bool,
	pub score_only: bool,
	pub m_moves: i16,
	pub saved_time: RogueTime,
}

impl SaveData {
	pub unsafe fn read_from_statics(file_id: i64, game: &GameState) -> Self {
		SaveData {
			player: game.player.clone(),
			hunger_str: hunger_str.clone(),
			login_name: login_name().to_string(),
			level_monsters: MASH.clone(),
			level_objects: level_objects.clone(),
			file_id,
			foods,
			id_potions: SaveIdTable::from_array(&id_potions),
			id_scrolls: SaveIdTable::from_array(&id_scrolls),
			id_wands: SaveIdTable::from_array(&id_wands),
			id_rings: SaveIdTable::from_array(&id_rings),
			is_wood: IS_WOOD.to_vec(),
			cur_room,
			level: game.level.clone(),
			confused,
			wizard,
			score_only: score_only(),
			m_moves,
			saved_time: get_current_time().add_seconds(10),
		}
	}
	pub unsafe fn write_to_statics(&self) {
		hunger_str = self.hunger_str.clone();
		settings::set_login_name(&self.login_name);
		MASH = self.level_monsters.clone();
		level_objects = self.level_objects.clone();
		foods = self.foods;
		load_array(&mut id_potions, &self.id_potions.ids);
		load_array(&mut id_scrolls, &self.id_scrolls.ids);
		load_array(&mut id_wands, &self.id_wands.ids);
		load_array(&mut id_rings, &self.id_rings.ids);
		load_array(&mut IS_WOOD, &self.is_wood);
		cur_room = self.cur_room;
		confused = self.confused;
		wizard = self.wizard;
		settings::set_score_only(self.score_only);
		m_moves = self.m_moves;
	}
}


fn load_array<T: Clone, const N: usize>(dest: &mut [T; N], src: &Vec<T>) {
	for i in 0..N {
		dest[i] = src[i].clone()
	}
}

#[derive(Serialize, Deserialize)]
pub struct SaveIdTable {
	ids: Vec<id>,
}

impl SaveIdTable {
	pub fn from_array(table: &[id]) -> SaveIdTable {
		let mut ids = Vec::new();
		for i in 0..table.len() {
			ids.push(table[i].clone());
		}
		SaveIdTable { ids }
	}
}