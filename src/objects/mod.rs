use std::clone::Clone;
use std::string::ToString;

use serde::{Deserialize, Serialize};

pub use object_id::*;
pub use object_pack::*;
use ObjectWhat::{Armor, Potion, Scroll, Weapon};

use crate::armors::constants::{ARMORS, PLATE, SPLINT};
use crate::armors::ArmorKind;
use crate::hit::DamageStat;
use crate::init::GameState;
use crate::inventory::get_obj_desc;
use crate::level::constants::MAX_ROOM;
use crate::level::materials::{CellMaterial, FloorFixture, TunnelFixture, Visibility};
use crate::level::Level;
use crate::message::sound_bell;
use crate::monster::party_monsters;
use crate::objects::note_tables::NoteTables;
use crate::odds::GOLD_PERCENT;
use crate::pack::MAX_PACK_COUNT;
use crate::player::Player;
use crate::potions::colors::PotionColor;
use crate::potions::kind::PotionKind::{Blindness, Confusion, DetectMonster, DetectObjects, ExtraHealing, Hallucination, Healing, IncreaseStrength, Levitation, Poison, RaiseLevel, RestoreStrength, SeeInvisible};
use crate::potions::kind::{PotionKind, POTIONS};
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::item_usage::{BEING_USED, BEING_WIELDED, BEING_WORN, NOT_USED, ON_EITHER_HAND, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::ObjectWhat::{Amulet, Food, Gold, Ring, Wand};
use crate::prelude::{DungeonSpot, MAX_ARMOR};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::resources::diary;
use crate::resources::input_line::get_input_line;
use crate::resources::keyboard::{rgetchar, CANCEL_CHAR};
use crate::ring::constants::RINGS;
use crate::ring::gr_ring;
use crate::ring::ring_gem::RingGem;
use crate::room::{gr_room, gr_spot, party_objects, RoomType};
use crate::scrolls::constants::SCROLLS;
use crate::scrolls::ScrollKind;
use crate::weapons::constants::{ARROW, DAGGER, DART, SHURIKEN, WEAPONS};
use crate::weapons::kind::WeaponKind;
use crate::zap::constants::WANDS;
use crate::zap::wand_kind::WandKind;
use crate::zap::wand_materials::WandMaterial;

mod object_id;
mod object_pack;
mod potions;
mod scrolls;
mod weapons;
mod armors;
mod kinds;

pub(crate) mod note_tables;

#[derive(Clone, Serialize, Deserialize)]
pub enum Title {
	None,
	WeaponName(WeaponKind),
	ArmorName(ArmorKind),
	PotionColor(PotionColor),
	SyllableString(String),
	WandMaterial(WandMaterial),
	RingGem(RingGem),
	UserString(String),
}

impl Title {
	pub fn to_string(&self) -> String {
		self.as_str().to_string()
	}
	pub fn as_str(&self) -> &str {
		match self {
			Title::None => &"",
			Title::WeaponName(kind) => kind.name(),
			Title::ArmorName(kind) => kind.name(),
			Title::PotionColor(color) => color.name(),
			Title::SyllableString(string) => string.as_str(),
			Title::WandMaterial(mat) => mat.name(),
			Title::RingGem(gem) => gem.name(),
			Title::UserString(string) => string.as_str(),
		}
	}
}

impl Default for Title {
	fn default() -> Self {
		Title::None
	}
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Note {
	pub title: Title,
	pub status: NoteStatus,
	pub is_wood: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NoteStatus {
	Unidentified,
	Identified,
	Called,
}

impl Default for NoteStatus {
	fn default() -> Self { NoteStatus::Unidentified }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct
Object {
	id: ObjectId,
	pub what_is: ObjectWhat,
	pub which_kind: u16,
	pub in_use_flags: u16,
	pub quantity: i16,
	pub ichar: char,
	pub is_protected: i16,
	pub is_cursed: i16,
	pub class: isize,
	pub identified: bool,
	pub spot: DungeonSpot,
	pub hit_enchant: i16,
	pub d_enchant: isize,
	pub quiver: i16,
	pub picked_up: i16,
}

impl Object {
	pub fn new(what: ObjectWhat) -> Self {
		let mut obj = empty_obj();
		obj.what_is = what;
		obj.quantity = 1;
		obj.ichar = 'L';
		obj.is_cursed = 0;
		obj.picked_up = 0;
		obj.in_use_flags = NOT_USED;
		obj.identified = false;
		obj
	}
}

pub fn empty_obj() -> Object {
	Object {
		id: ObjectId::random(),
		quantity: 0,
		ichar: '\x00',
		is_protected: 0,
		is_cursed: 0,
		class: 0,
		identified: false,
		which_kind: 0,
		spot: DungeonSpot::default(),
		d_enchant: 0,
		quiver: 0,
		hit_enchant: 0,
		what_is: ObjectWhat::None,
		picked_up: 0,
		in_use_flags: 0,
	}
}

impl Object {
	pub fn clone_with_new_id(&self) -> Self {
		let mut new = self.clone();
		new.id = ObjectId::random();
		new
	}
	pub fn to_name_with_new_quantity(&self, quantity: i16, fruit: String, notes: &NoteTables) -> String {
		let mut temp_obj = self.clone();
		temp_obj.quantity = quantity;
		name_of(&temp_obj, fruit, notes)
	}
	pub fn can_join_existing_pack_object(&self, existing_pack_obj: &Self) -> bool {
		self.is_same_kind(existing_pack_obj) &&
			(!self.is_weapon() || (self.is_arrow_or_throwing_weapon() && self.quiver == existing_pack_obj.quiver))
	}
	pub fn is_same_kind(&self, other: &Self) -> bool { self.what_is == other.what_is && self.which_kind == other.which_kind }
	pub fn is_cursed(&self) -> bool { self.is_cursed != 0 }
	pub fn is_being_used(&self) -> bool { self.in_use_flags & BEING_USED != 0 }
	pub fn is_being_wielded(&self) -> bool { self.in_use_flags & BEING_WIELDED != 0 }
	pub fn is_being_worn(&self) -> bool { self.in_use_flags & BEING_WORN != 0 }
	pub fn is_on_either_hand(&self) -> bool { self.in_use_flags & ON_EITHER_HAND != 0 }
	pub fn is_on_left_hand(&self) -> bool { self.in_use_flags & ON_LEFT_HAND != 0 }
	pub fn is_on_right_hand(&self) -> bool { self.in_use_flags & ON_RIGHT_HAND != 0 }
	pub fn is_at(&self, row: i64, col: i64) -> bool {
		self.spot.row == row && self.spot.col == col
	}
	pub fn gold_quantity(&self) -> Option<usize> {
		if self.what_is == Gold {
			Some(self.quantity as usize)
		} else {
			None
		}
	}
	pub fn raise_armor_enchant(&mut self, raise: isize) {
		self.d_enchant = (self.d_enchant + raise).min(MAX_ARMOR);
	}
	pub fn base_damage(&self) -> DamageStat {
		if let Some(kind) = self.weapon_kind() {
			kind.damage()
		} else {
			DamageStat { hits: 1, damage: 1 }
		}
	}
	pub fn enhanced_damage(&self) -> DamageStat {
		let DamageStat { hits, damage } = self.base_damage();
		let hits = hits + self.hit_enchant as usize;
		let damage = damage + self.d_enchant as usize;
		DamageStat { hits, damage }
	}
	pub fn to_spot(&self) -> DungeonSpot { self.spot }
	pub fn id(&self) -> ObjectId { self.id }
}

pub fn put_objects(game: &mut GameState) {
	if game.player.cur_depth < game.player.max_depth {
		return;
	}

	let mut n = if coin_toss() { get_rand(2, 4) } else { get_rand(3, 5) };
	while rand_percent(33) {
		n += 1;
	}
	if game.player.cur_depth == game.player.party_counter {
		make_party(game.player.cur_depth, game);
		game.player.party_counter = next_party(game.player.cur_depth);
	}
	for _i in 0..n {
		let obj = gr_object(&mut game.player);
		rand_place(obj, game);
	}
	put_gold(game.player.cur_depth, &mut game.level, &mut game.ground);
}

pub fn put_gold(level_depth: isize, level: &mut Level, ground: &mut ObjectPack) {
	for i in 0..MAX_ROOM {
		let is_maze = level.rooms[i].room_type == RoomType::Maze;
		let is_room = level.rooms[i].room_type == RoomType::Room;
		if !(is_room || is_maze) {
			continue;
		}
		if is_maze || rand_percent(GOLD_PERCENT) {
			for _j in 0..50 {
				let row = get_rand(level.rooms[i].top_row + 1, level.rooms[i].bottom_row - 1);
				let col = get_rand(level.rooms[i].left_col + 1, level.rooms[i].right_col - 1);
				if level.dungeon[row as usize][col as usize].is_material_no_others(CellMaterial::Floor(FloorFixture::None))
					|| level.dungeon[row as usize][col as usize].is_material_no_others(CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::None)) {
					plant_gold(row, col, is_maze, level_depth, level, ground);
					break;
				}
			}
		}
	}
}

pub fn plant_gold(row: i64, col: i64, is_maze: bool, cur_level: isize, level: &mut Level, ground: &mut ObjectPack) {
	let mut obj = alloc_object();
	obj.spot.set(row, col);
	obj.what_is = Gold;
	obj.quantity = get_rand((2 * cur_level) as i16, (16 * cur_level) as i16);
	if is_maze {
		obj.quantity += obj.quantity / 2;
	}
	level.dungeon[row as usize][col as usize].set_object(Gold);
	ground.add(obj);
}


pub fn place_at(mut obj: Object, row: i64, col: i64, level: &mut Level, ground: &mut ObjectPack) {
	obj.spot.set(row, col);
	level.dungeon[row as usize][col as usize].set_object(obj.what_is);
	ground.add(obj);
}

impl Player {
	pub fn object_id_with_letter(&self, ch: char) -> Option<ObjectId> {
		self.obj_id_if(|obj| obj.ichar == ch)
	}
}

impl Player {
	pub fn object_what(&self, obj_id: ObjectId) -> ObjectWhat {
		if let Some(obj) = self.object(obj_id) { obj.what_is } else { ObjectWhat::None }
	}
	pub fn object_kind(&self, obj_id: ObjectId) -> u16 {
		if let Some(obj) = self.object(obj_id) { obj.which_kind } else { 0 }
	}
	pub fn check_object(&self, obj_id: ObjectId, f: impl Fn(&Object) -> bool) -> bool {
		self.pack().check_object(obj_id, f)
	}
	pub fn obj_id_if(&self, f: impl Fn(&Object) -> bool) -> Option<ObjectId> {
		self.pack().find_id(f)
	}
	pub fn pack(&self) -> &ObjectPack { &self.rogue.pack }

	pub fn object_with_letter(&self, ch: char) -> Option<&Object> {
		self.find_pack_obj(|obj| obj.ichar == ch)
	}
	pub fn object_with_letter_mut(&mut self, ch: char) -> Option<&mut Object> {
		self.find_pack_obj_mut(|obj| obj.ichar == ch)
	}
	pub fn name_of(&self, obj_id: ObjectId) -> String {
		let obj = self.expect_object(obj_id);
		name_of(obj, self.settings.fruit.to_string(), &self.notes)
	}
}

pub fn name_of(obj: &Object, fruit: String, notes: &NoteTables) -> String {
	let what = obj.what_is;
	match what {
		Armor => "armor ".to_string(),
		Weapon => {
			let kind = obj.which_kind;
			match kind {
				DART => if obj.quantity > 1 { "darts " } else { "dart " }.to_string(),
				ARROW => if obj.quantity > 1 { "arrows " } else { "arrow " }.to_string(),
				DAGGER => if obj.quantity > 1 { "daggers " } else { "dagger " }.to_string(),
				SHURIKEN => if obj.quantity > 1 { "shurikens " } else { "shuriken " }.to_string(),
				_ => {
					notes.title(what, kind as usize).to_string()
				}
			}
		}
		Scroll => if obj.quantity > 1 { "scrolls " } else { "scroll " }.to_string(),
		Potion => if obj.quantity > 1 { "potions " } else { "potion " }.to_string(),
		Food => if obj.which_kind == RATION { "food ".to_string() } else { fruit }
		Wand => {
			let is_wood = notes.note(Wand, obj.which_kind as usize).is_wood;
			if is_wood { "staff " } else { "wand " }.to_string()
		}
		Ring => "ring ".to_string(),
		Amulet => "amulet ".to_string(),
		_ => "unknown ".to_string(),
	}
}

pub fn gr_object(player: &mut Player) -> Object {
	let mut obj = alloc_object();
	if player.foods < (player.cur_depth / 2) {
		obj.what_is = Food;
		player.foods += 1;
	} else {
		obj.what_is = gr_what_is();
	}
	match obj.what_is {
		Scroll => {
			gr_scroll(&mut obj);
		}
		Potion => {
			gr_potion(&mut obj);
		}
		Weapon => {
			gr_weapon(&mut obj, true);
		}
		Armor => {
			gr_armor(&mut obj);
		}
		Wand => {
			gr_wand(&mut obj);
		}
		Food => {
			get_food(&mut obj, false);
		}
		Ring => {
			gr_ring(&mut obj, true);
		}
		_ => {}
	}
	obj
}

pub fn gr_what_is() -> ObjectWhat {
	let percent = get_rand(1, 91);
	if percent <= 30 {
		Scroll
	} else if percent <= 60 {
		Potion
	} else if percent <= 64 {
		Wand
	} else if percent <= 74 {
		Weapon
	} else if percent <= 83 {
		Armor
	} else if percent <= 88 {
		Food
	} else {
		Ring
	}
}

impl Object {
	pub fn roll_scroll() -> Object {
		let mut object = Object::new(Scroll);
		gr_scroll(&mut object);
		object
	}
	pub fn roll_potion() -> Object {
		let mut object = Object::new(Potion);
		gr_potion(&mut object);
		object
	}
	pub fn roll_weapon(assign_kind: bool) -> Object {
		let mut object = Object::new(Weapon);
		gr_weapon(&mut object, assign_kind);
		object
	}
	pub fn roll_armor() -> Object {
		let mut object = Object::new(Armor);
		gr_armor(&mut object);
		object
	}
	pub fn roll_wand() -> Object {
		let mut object = Object::new(Wand);
		gr_wand(&mut object);
		object
	}
	pub fn roll_food(force_ration: bool) -> Object {
		let mut object = Object::new(Food);
		get_food(&mut object, force_ration);
		object
	}
	pub fn roll_ring(assign_kind: bool) -> Object {
		let mut object = Object::new(Ring);
		gr_ring(&mut object, assign_kind);
		object
	}
}

pub fn gr_scroll(obj: &mut Object) {
	let percent = get_rand(0, 85);
	(*obj).what_is = Scroll;

	let kind = if percent <= 5 {
		ScrollKind::ProtectArmor
	} else if percent <= 11 {
		ScrollKind::HoldMonster
	} else if percent <= 20 {
		ScrollKind::CreateMonster
	} else if percent <= 35 {
		ScrollKind::Identify
	} else if percent <= 43 {
		ScrollKind::Teleport
	} else if percent <= 50 {
		ScrollKind::Sleep
	} else if percent <= 55 {
		ScrollKind::ScareMonster
	} else if percent <= 64 {
		ScrollKind::RemoveCurse
	} else if percent <= 69 {
		ScrollKind::EnchArmor
	} else if percent <= 74 {
		ScrollKind::EnchWeapon
	} else if percent <= 80 {
		ScrollKind::AggravateMonster
	} else {
		ScrollKind::MagicMapping
	};
	(*obj).which_kind = kind.to_index() as u16;
}

pub fn gr_potion(obj: &mut Object) {
	(*obj).what_is = Potion;
	(*obj).which_kind = gr_potion_kind().to_index() as u16;
}

fn gr_potion_kind() -> PotionKind {
	let percent = get_rand(1, 118);
	let kind = if percent <= 5 {
		RaiseLevel
	} else if percent <= 15 {
		DetectObjects
	} else if percent <= 25 {
		DetectMonster
	} else if percent <= 35 {
		IncreaseStrength
	} else if percent <= 45 {
		RestoreStrength
	} else if percent <= 55 {
		Healing
	} else if percent <= 65 {
		ExtraHealing
	} else if percent <= 75 {
		Blindness
	} else if percent <= 85 {
		Hallucination
	} else if percent <= 95 {
		Confusion
	} else if percent <= 105 {
		Poison
	} else if percent <= 110 {
		Levitation
	} else if percent <= 114 {
		Hallucination
	} else {
		SeeInvisible
	};
	kind
}

pub fn gr_weapon(obj: &mut Object, assign_wk: bool) {
	(*obj).what_is = Weapon;
	if assign_wk {
		(*obj).which_kind = get_rand(0, (WEAPONS - 1) as u16);
	}
	if ((*obj).which_kind == ARROW) || ((*obj).which_kind == DAGGER) || ((*obj).which_kind == SHURIKEN) | ((*obj).which_kind == DART) {
		(*obj).quantity = get_rand(3, 15);
		(*obj).quiver = get_rand(0, 126);
	} else {
		(*obj).quantity = 1;
	}
	(*obj).hit_enchant = 0;
	(*obj).d_enchant = 0;

	let percent = get_rand(1, 96);
	let blessing = get_rand(1, 3);

	let mut increment = 0;
	if percent <= 16 {
		increment = 1;
	} else if percent <= 32 {
		increment = -1;
		(*obj).is_cursed = 1;
	}
	if percent <= 32 {
		for _ in 0..blessing {
			if coin_toss() {
				(*obj).hit_enchant += increment;
			} else {
				(*obj).d_enchant += increment as isize;
			}
		}
	}
}

pub fn gr_armor(obj: &mut Object) {
	obj.what_is = Armor;
	obj.which_kind = get_rand(0, (ARMORS - 1) as u16);
	obj.class = (obj.which_kind + 2) as isize;
	if obj.which_kind == PLATE || obj.which_kind == SPLINT {
		obj.class -= 1;
	}
	obj.is_protected = 0;
	obj.d_enchant = 0;

	let percent = get_rand(1, 100);
	let blessing = get_rand(1, 3);

	if percent <= 16 {
		obj.is_cursed = 1;
		obj.d_enchant -= blessing;
	} else if percent <= 33 {
		obj.raise_armor_enchant(blessing);
	}
}

pub fn gr_wand(obj: &mut Object) {
	obj.what_is = Wand;
	obj.which_kind = get_rand(0, (WANDS - 1) as u16);
	if obj.which_kind == WandKind::MagicMissile.to_index() as u16 {
		obj.class = get_rand(6, 12);
	} else if obj.which_kind == WandKind::Cancellation.to_index() as u16 {
		obj.class = get_rand(5, 9);
	} else {
		obj.class = get_rand(3, 6);
	}
}

pub fn get_food(obj: &mut Object, force_ration: bool) {
	obj.what_is = Food;
	if force_ration || rand_percent(80) {
		obj.which_kind = RATION;
	} else {
		obj.which_kind = FRUIT;
	}
}

pub fn put_stairs(player: &Player, level: &mut Level) {
	let spot = gr_spot(|cell| cell.is_any_floor() || cell.is_any_tunnel(), player, level);
	level.dungeon[spot.row as usize][spot.col as usize].add_stairs();
}

pub fn get_armor_class(obj: Option<&Object>) -> isize {
	if let Some(armor) = obj {
		armor.class + armor.d_enchant
	} else { 0 }
}

pub fn alloc_object() -> Object {
	let mut obj = empty_obj();
	obj.quantity = 1;
	obj.ichar = 'L';
	obj.is_cursed = 0;
	obj.picked_up = 0;
	obj.in_use_flags = NOT_USED;
	obj.identified = false;
	obj
}

pub fn make_party(level_depth: isize, game: &mut GameState) {
	let party_room = gr_room(&game.level);
	game.level.party_room = Some(party_room);
	let n = if rand_percent(99) { party_objects(party_room, game) } else { 11 };
	if rand_percent(99) {
		party_monsters(party_room, n, level_depth, &mut game.mash, &mut game.level);
	}
}

pub fn show_objects(game: &mut GameState) {
	let obj_spots = game.ground.objects().iter().map(|it| it.spot).collect::<Vec<_>>();
	for obj_spot in obj_spots {
		if !game.cell_at(obj_spot).is_visited() {
			game.cell_at_mut(obj_spot).visit();
			game.render_spot(obj_spot);
		}
	}
	let imitating_mon_spots = game.mash.monsters().iter()
		.filter(|it| it.imitates())
		.map(|it| it.spot)
		.collect::<Vec<_>>();
	for mon_spot in imitating_mon_spots {
		if !game.cell_at(mon_spot).is_visited() {
			game.cell_at_mut(mon_spot).visit();
			game.render_spot(mon_spot);
		}
	}
}

pub fn put_amulet(game: &mut GameState) {
	let mut obj = alloc_object();
	obj.what_is = Amulet;
	rand_place(obj, game);
}

pub fn rand_place(obj: Object, game: &mut GameState) {
	let spot = gr_spot(|cell| cell.is_any_floor() || cell.is_any_tunnel(), &game.player, &game.level);
	place_at(obj, spot.row, spot.col, &mut game.level, &mut game.ground);
}

pub fn new_object_for_wizard(game: &mut GameState) {
	if game.player.pack_weight_with_new_object(None) >= MAX_PACK_COUNT {
		game.diary.add_entry("pack full");
		return;
	}
	diary::show_prompt("type of object?", &mut game.diary);
	let ch = {
		const CHOICES: &'static str = "!?:)]=/,\x1B";
		let mut ch: char;
		loop {
			ch = rgetchar();
			match CHOICES.find(ch) {
				None => {
					sound_bell();
				}
				Some(_) => {
					break;
				}
			}
		}
		ch
	};
	if ch == CANCEL_CHAR {
		return;
	}
	let mut obj = alloc_object();
	let max_kind = match ch {
		'!' => {
			obj.what_is = Potion;
			Some(POTIONS - 1)
		}
		'?' => {
			obj.what_is = Scroll;
			Some(SCROLLS - 1)
		}
		',' => {
			obj.what_is = Amulet;
			None
		}
		':' => {
			get_food(&mut obj, false);
			None
		}
		')' => {
			gr_weapon(&mut obj, false);
			Some(WEAPONS - 1)
		}
		']' => {
			gr_armor(&mut obj);
			Some(ARMORS - 1)
		}
		'/' => {
			gr_wand(&mut obj);
			Some(WANDS - 1)
		}
		'=' => {
			obj.what_is = Ring;
			Some(RINGS - 1)
		}
		_ => None
	};
	if let Some(max_kind) = max_kind {
		if let Some(kind) = get_kind(max_kind, game) {
			obj.which_kind = kind as u16;
			if obj.what_is == Ring {
				gr_ring(&mut obj, false);
			}
		} else {
			return;
		}
	}
	let obj_desc = get_obj_desc(&obj, game.player.settings.fruit.to_string(), &game.player);
	game.diary.add_entry(&obj_desc);
	game.player.combine_or_add_item_to_pack(obj);
}

fn get_kind(max_kind: usize, game: &mut GameState) -> Option<usize> {
	let good_kind = {
		let good_kind;
		loop {
			let line = get_input_line::<String>("which kind?", None, None, false, true, &mut game.diary);
			let trimmed_line = line.trim();
			if trimmed_line.is_empty() {
				good_kind = None;
				break;
			}
			match trimmed_line.parse::<isize>() {
				Err(_) => {
					sound_bell();
				}
				Ok(kind) => {
					if kind >= 0 && kind <= max_kind as isize {
						good_kind = Some(kind as usize);
						break;
					}
				}
			}
		}
		good_kind
	};
	good_kind
}

fn next_party(cur_level: isize) -> isize {
	const PARTY_TIME: isize = 10;   /* one party somewhere in each 10 level span */
	let mut n = cur_level;
	while (n % PARTY_TIME) > 0 {
		n += 1;
	}
	get_rand(n + 1, n + PARTY_TIME)
}