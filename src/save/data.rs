use chrono::{DateTime, Duration, Utc};
use ncurses::chtype;
use serde::Serialize;
use crate::level::{cur_level, cur_room, max_level, party_room};
use crate::monster::fighter;
use crate::objects::{foods, id, obj, party_counter, SaveObj};
use crate::prelude::{bear_trap, being_held, blind, confused, DCOLS, detect_monster, DROWS, halluc, haste_self, levitate, m_moves, see_invisible, wizard};
use crate::room::{room, rooms};
use crate::save::{hunger_str, id_potions, id_rings, id_scrolls, id_wands, is_wood, level_monsters, level_objects, rogue, traps};
use crate::settings::{login_name, score_only};
use crate::trap::trap;

#[derive(Serialize)]
pub struct SavePack {
	pub save_objs: Vec<SaveObj>,
}

impl SavePack {
	pub unsafe fn from_pack(pack: *mut obj) -> SavePack {
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
}

#[derive(Serialize)]
pub struct SaveFighter {
	pub armor: Option<SaveObj>,
	pub weapon: Option<SaveObj>,
	pub left_ring: Option<SaveObj>,
	pub right_ring: Option<SaveObj>,
	pub hp_current: i16,
	pub hp_max: i16,
	pub str_current: i16,
	pub str_max: i16,
	pub pack: SavePack,
	pub gold: i64,
	pub exp: i16,
	pub exp_points: i64,
	pub row: i16,
	pub col: i16,
	pub fchar: i16,
	pub moves_left: i16,
}

impl SaveFighter {
	pub unsafe fn from_fighter(fighter: &fighter) -> SaveFighter {
		SaveFighter {
			armor: SaveObj::option_save_obj(fighter.armor),
			weapon: SaveObj::option_save_obj(fighter.weapon),
			left_ring: SaveObj::option_save_obj(fighter.left_ring),
			right_ring: SaveObj::option_save_obj(fighter.right_ring),
			hp_current: fighter.hp_current,
			hp_max: fighter.hp_max,
			str_current: fighter.str_current,
			str_max: fighter.str_max,
			pack: SavePack::from_pack(*fighter.pack),
			gold: fighter.gold,
			exp: fighter.exp,
			exp_points: fighter.exp_points,
			row: fighter.row,
			col: fighter.col,
			fchar: fighter.fchar,
			moves_left: fighter.moves_left,
		}
	}
}

#[derive(Serialize)]
pub struct SaveDungeon {
	pub dungeon: [[chtype; DCOLS]; DROWS],
}

impl SaveDungeon {
	pub fn from_statics() -> SaveDungeon {
		let mut dungeon = [[0; DCOLS]; DROWS];
		for row in 0..DROWS {
			for col in 0..DCOLS {
				dungeon[row][col] = ncurses::mvinch(row as i32, col as i32);
			}
		}
		SaveDungeon { dungeon }
	}
}

#[derive(Serialize)]
pub struct SaveData {
	pub detect_monster: bool,
	pub cur_level: i16,
	pub max_level: i16,
	pub hunger_str: String,
	pub login_name: String,
	pub party_room: i16,
	pub party_counter: i16,
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
	pub cur_room: i16,
	pub rooms: Vec<room>,
	pub being_held: bool,
	pub bear_trap: bool,
	pub halluc: bool,
	pub blind: bool,
	pub confused: bool,
	pub levitate: bool,
	pub haste_self: bool,
	pub see_invisible: bool,
	pub wizard: bool,
	pub score_only: bool,
	pub m_moves: i16,
	pub saved_time: DateTime<Utc>,
}

pub unsafe fn from_statics(file_id: i64) -> SaveData {
	SaveData {
		detect_monster,
		cur_level,
		max_level,
		hunger_str: hunger_str.clone(),
		login_name: login_name().to_string(),
		party_room,
		party_counter,
		level_monsters: SavePack::from_pack(&*level_monsters),
		level_objects: SavePack::from_pack(&*level_objects),
		file_id,
		dungeon: SaveDungeon::from_statics(),
		foods,
		rogue: SaveFighter::from_fighter(&rogue),
		rogue_pack: SavePack::from_pack(&*rogue.pack),
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
		saved_time: Utc::now() + Duration::seconds(10),
	}
}

#[derive(Serialize)]
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