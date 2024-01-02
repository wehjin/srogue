use std::error::Error;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::level::{cur_level, cur_room, max_level, party_room};
use crate::machdep::{get_current_time, RogueTime};
use crate::monster::{fight, fighter};
use crate::objects::{dungeon, foods, id, obj, party_counter, SaveObj};
use crate::prelude::{bear_trap, being_held, blind, confused, DCOLS, detect_monster, DROWS, halluc, haste_self, levitate, m_moves, see_invisible, wizard};
use crate::room::{room, rooms};
use crate::save::{hunger_str, id_potions, id_rings, id_scrolls, id_wands, is_wood, level_monsters, level_objects, rogue, traps};
use crate::settings;
use crate::settings::{login_name, score_only};
use crate::trap::trap;

pub fn from_file(path: &str) -> Result<SaveData, Box<dyn Error>> {
	let json = fs::read_to_string(path)?;
	let data = serde_json::from_str::<SaveData>(&json)?;
	Ok(data)
}


#[derive(Serialize, Deserialize)]
pub struct SavePack {
	pub save_objs: Vec<SaveObj>,
}

impl SavePack {
	pub unsafe fn from_pack(pack: *const obj) -> SavePack {
		let mut save_objs = Vec::new();
		loop {
			let pack = (*pack).next_object;
			if pack.is_null() {
				break;
			}
			let obj = SaveObj::from_obj(&*pack);
			save_objs.push(obj);
		}
		SavePack { save_objs }
	}
	pub unsafe fn write_pack(&self, pack: &mut obj, is_rogue: bool) {
		let mut tail = pack as *mut obj;
		for save_obj in &self.save_objs {
			let new_obj = save_obj.to_obj(is_rogue);
			(*tail).next_object = new_obj;
			tail = new_obj;
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct SaveFighter {
	pub hp_current: isize,
	pub hp_max: isize,
	pub str_current: isize,
	pub str_max: isize,
	pub gold: isize,
	pub exp: isize,
	pub exp_points: isize,
	pub row: i64,
	pub col: i64,
	pub fchar: char,
	pub moves_left: i16,
}

impl SaveFighter {
	pub fn from_fighter(fighter: &fighter) -> Self {
		SaveFighter {
			hp_current: fighter.hp_current,
			hp_max: fighter.hp_max,
			str_current: fighter.str_current,
			str_max: fighter.str_max,
			gold: fighter.gold,
			exp: fighter.exp,
			exp_points: fighter.exp_points,
			row: fighter.row,
			col: fighter.col,
			fchar: fighter.fchar,
			moves_left: fighter.moves_left,
		}
	}

	pub fn to_fighter(&self) -> fight {
		fight {
			armor: 0 as *mut obj,
			weapon: 0 as *mut obj,
			left_ring: 0 as *mut obj,
			right_ring: 0 as *mut obj,
			hp_current: self.hp_current,
			hp_max: self.hp_max,
			str_current: self.str_current,
			str_max: self.str_max,
			pack: obj::default(),
			gold: self.gold,
			exp: self.exp,
			exp_points: self.exp_points,
			row: self.row,
			col: self.col,
			fchar: self.fchar,
			moves_left: self.moves_left,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct SaveDungeon {
	pub cells: Vec<Vec<u16>>,
}

impl SaveDungeon {
	unsafe fn from_dungeon(cells: &[[u16; DCOLS]; DROWS]) -> SaveDungeon {
		let mut rows = Vec::new();
		for row in 0..DROWS {
			let mut cols = Vec::new();
			for col in 0..DCOLS {
				cols.push(cells[row][col]);
			}
			rows.push(cols);
		}
		// Don't know why, but C code also saves and restores screen chars. Let's not
		// do that for now.
		SaveDungeon { cells: rows }
	}

	fn load_dungeon(&self, cells: &mut [[u16; DCOLS]; DROWS]) {
		for row in 0..DROWS {
			for col in 0..DCOLS {
				cells[row][col] = self.cells[row][col].clone();
			}
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
	pub detect_monster: bool,
	pub cur_level: isize,
	pub max_level: isize,
	pub hunger_str: String,
	pub login_name: String,
	pub party_room: i64,
	pub party_counter: isize,
	pub level_monsters: SavePack,
	pub level_objects: SavePack,
	pub file_id: i64,
	pub dungeon: SaveDungeon,
	pub foods: i16,
	pub rogue: SaveFighter,
	pub rogue_pack: SavePack,
	pub id_potions: SaveIdTable,
	pub id_scrolls: SaveIdTable,
	pub id_wands: SaveIdTable,
	pub id_rings: SaveIdTable,
	pub traps: Vec<trap>,
	pub is_wood: Vec<bool>,
	pub cur_room: i64,
	pub rooms: Vec<room>,
	pub being_held: bool,
	pub bear_trap: usize,
	pub halluc: usize,
	pub blind: usize,
	pub confused: usize,
	pub levitate: usize,
	pub haste_self: usize,
	pub see_invisible: bool,
	pub wizard: bool,
	pub score_only: bool,
	pub m_moves: i16,
	pub saved_time: RogueTime,
}

impl SaveData {
	pub unsafe fn read_from_statics(file_id: i64) -> Self {
		SaveData {
			detect_monster,
			cur_level,
			max_level,
			hunger_str: hunger_str.clone(),
			login_name: login_name().to_string(),
			party_room,
			party_counter,
			level_monsters: SavePack::from_pack(&level_monsters),
			level_objects: SavePack::from_pack(&level_objects),
			file_id,
			dungeon: SaveDungeon::from_dungeon(&dungeon),
			foods,
			rogue: SaveFighter::from_fighter(&rogue),
			rogue_pack: SavePack::from_pack(&rogue.pack),
			id_potions: SaveIdTable::from_array(&id_potions),
			id_scrolls: SaveIdTable::from_array(&id_scrolls),
			id_wands: SaveIdTable::from_array(&id_wands),
			id_rings: SaveIdTable::from_array(&id_rings),
			traps: traps.to_vec(),
			is_wood: is_wood.to_vec(),
			cur_room,
			rooms: rooms.to_vec(),
			being_held,
			bear_trap,
			halluc,
			blind,
			confused,
			levitate,
			haste_self,
			see_invisible,
			wizard,
			score_only: score_only(),
			m_moves,
			saved_time: get_current_time().add_seconds(10),
		}
	}
	pub unsafe fn write_to_statics(&self) {
		detect_monster = self.detect_monster;
		cur_level = self.cur_level;
		max_level = self.max_level;
		hunger_str = self.hunger_str.clone();
		settings::set_login_name(&self.login_name);
		party_room = self.party_room;
		party_counter = self.party_counter;
		self.level_monsters.write_pack(&mut level_monsters, false);
		self.level_objects.write_pack(&mut level_objects, false);
		self.dungeon.load_dungeon(&mut dungeon);
		foods = self.foods;
		rogue = self.rogue.to_fighter();
		self.rogue_pack.write_pack(&mut rogue.pack, true);
		load_array(&mut id_potions, &self.id_potions.ids);
		load_array(&mut id_scrolls, &self.id_scrolls.ids);
		load_array(&mut id_wands, &self.id_wands.ids);
		load_array(&mut id_rings, &self.id_rings.ids);
		load_array(&mut traps, &self.traps);
		load_array(&mut is_wood, &self.is_wood);
		cur_room = self.cur_room;
		load_array(&mut rooms, &self.rooms);
		being_held = self.being_held;
		bear_trap = self.bear_trap;
		halluc = self.halluc;
		blind = self.blind;
		confused = self.confused;
		levitate = self.levitate;
		haste_self = self.haste_self;
		see_invisible = self.see_invisible;
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