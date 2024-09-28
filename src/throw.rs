use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};

use crate::hit::{get_hit_chance, get_weapon_damage, mon_damage};
use crate::init::{Dungeon, GameState};
use crate::level::DungeonCell;
use crate::monster::{mv_aquatars, MonsterIndex};
use crate::motion::{get_dir_or_cancel, is_passable};
use crate::objects::{place_at, Object, ObjectId};
use crate::pack::{pack_letter, unwear, unwield, CURSE_MESSAGE};
use crate::player::Player;
use crate::prelude::item_usage::NOT_USED;
use crate::prelude::object_what::ObjectWhat::Wand;
use crate::prelude::object_what::PackFilter::Weapons;
use crate::prelude::*;
use crate::r#use::vanish;
use crate::random::rand_percent;
use crate::render_system::animation;
use crate::resources::avatar::Avatar;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::ring::un_put_hand;
use crate::throw::Motion::{Down, DownLeft, DownRight, Left, Right, Same, Up, UpLeft, UpRight};
use crate::weapons::constants::ARROW;
use crate::weapons::kind::WeaponKind;
use crate::zap::zap_monster;

impl GameState {
	pub fn has_non_imitating_monster_at(&self, spot: DungeonSpot) -> bool {
		let mon_id = self.mash.monster_id_at_spot(spot.row, spot.col);
		match mon_id {
			None => false,
			Some(id) if self.mash.monster_flags(id).imitates => false,
			Some(_) => true
		}
	}
	pub fn is_impassable_at(&self, spot: DungeonSpot) -> bool {
		!is_passable(spot.row, spot.col, &self.level)
	}
	pub fn mon_id_at(&self, spot: DungeonSpot) -> MonsterIndex {
		self.mash.monster_id_at_spot(spot.row, spot.col).expect("monster at spot")
	}
	pub fn cell_at(&self, spot: DungeonSpot) -> &DungeonCell {
		self.level.cell(spot)
	}
	pub fn cell_at_mut(&mut self, spot: DungeonSpot) -> &mut DungeonCell {
		self.level.cell_mut(spot)
	}
	pub fn player_can_see(&self, spot: DungeonSpot) -> bool {
		self.player.can_see_spot(&spot, &self.level)
	}
	pub fn player_is_at(&self, spot: DungeonSpot) -> bool {
		self.player.is_at_spot(spot)
	}
}

pub fn throw(game: &mut GameState) {
	let dir = get_dir_or_cancel(game);
	if dir == CANCEL_CHAR {
		return;
	}
	let wch = pack_letter("throw what?", Weapons, game);
	if wch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(wch) {
		None => {
			game.diary.add_entry("no such item.");
			return;
		}
		Some(obj_id) => {
			if game.player.check_object(obj_id, |it| it.is_being_used() && it.is_cursed()) {
				game.diary.add_entry(CURSE_MESSAGE);
				return;
			}
			if game.player.check_object(obj_id, |it| it.is_being_wielded() && it.quantity <= 1) {
				unwield(&mut game.player);
			} else if game.player.check_object(obj_id, |it| it.is_being_worn()) {
				mv_aquatars(game);
				unwear(&mut game.player);
				game.as_diary_mut().set_stats_changed(true);
			} else if let Some(hand) = game.player.ring_hand(obj_id) {
				un_put_hand(hand, game);
			}
			let throw_ending = simulate_throw(Motion::from(dir), game);
			animation::animate_throw(obj_id, &throw_ending, game);
			match throw_ending {
				ThrowEnding::StrikesWall { bounce_spot, .. } => {
					flop_object_from_spot(obj_id, bounce_spot, game);
				}
				ThrowEnding::ReachesMonster { mon_id, mon_spot, .. } => {
					{
						let monster = game.mash.monster_mut(mon_id);
						monster.wake_up();
						monster.m_flags.seeks_gold = false;
					}
					if !throw_at_monster(mon_id, obj_id, game) {
						flop_object_from_spot(obj_id, mon_spot, game);
					}
				}
				ThrowEnding::LandsOnGround { landing_spot, .. } => {
					flop_object_from_spot(obj_id, landing_spot, game);
				}
			}
			vanish(obj_id, true, game);
		}
	}
}

fn throw_at_monster(mon_id: u64, obj_id: ObjectId, game: &mut GameState) -> bool {
	let hit_chance = {
		let player_exp = game.player.buffed_exp();
		let player_debuf = game.player.debuf_exp();
		let player_weapon_is_bow = rogue_weapon_is_bow(&game.player);

		let obj = game.player.object(obj_id).expect("obj in pack");
		let mut hit_chance = get_hit_chance(Some(obj), player_exp, player_debuf);
		if obj.which_kind == ARROW && player_weapon_is_bow {
			hit_chance += hit_chance / 3;
		} else if obj.is_wielded_throwing_weapon() {
			hit_chance += hit_chance / 3;
		}
		hit_chance
	};
	{
		let name = game.player.to_object_name_with_quantity(obj_id, 1).trim().to_string();
		let diary = game.as_diary_mut();
		diary.hit_message = format!("the {}", name);
		if !rand_percent(hit_chance) {
			diary.hit_message += " misses  ";
			return false;
		}
		diary.hit_message += " hit  ";
	}
	if game.player.object_what(obj_id) == Wand && rand_percent(75) {
		zap_monster(mon_id, game.player.object_kind(obj_id), game);
	} else {
		let player_str = game.player.buffed_strength();
		let player_exp = game.buffed_exp();
		let player_debuf = game.debuf_exp();
		let damage = {
			let mut damage = get_weapon_damage(game.player.object(obj_id), player_str, player_exp, player_debuf);
			if game.player.object_kind(obj_id) == ARROW && rogue_weapon_is_bow(&game.player) {
				damage += get_weapon_damage(game.player.weapon(), player_str, player_exp, player_debuf);
				damage = (damage * 2) / 3;
			} else if game.player.check_object(obj_id, Object::is_wielded_throwing_weapon) {
				damage = (damage * 3) / 2;
			}
			damage
		};
		mon_damage(mon_id, damage, game);
	}
	true
}

fn rogue_weapon_is_bow(player: &Player) -> bool {
	player.weapon_kind() == Some(WeaponKind::Bow)
}

fn simulate_throw(motion: Motion, game: &GameState) -> ThrowEnding {
	let rogue_spot = game.player.to_spot();
	let mut flight_path = Vec::new();
	let mut timeout = 0;
	loop {
		if timeout >= 24 {
			break;
		}
		let cur_spot = match flight_path.last() {
			None => &rogue_spot,
			Some(last_spot) => last_spot,
		};
		match cur_spot.after_motion(motion) {
			None => {
				return ThrowEnding::StrikesWall { bounce_spot: *cur_spot, flight_path, rogue_spot };
			}
			Some(next_spot) if next_spot == *cur_spot => {
				return ThrowEnding::LandsOnGround { landing_spot: rogue_spot, flight_path, rogue_spot };
			}
			Some(next_spot) if game.is_impassable_at(next_spot) => {
				return ThrowEnding::StrikesWall { bounce_spot: *cur_spot, flight_path, rogue_spot };
			}
			Some(next_spot) if game.has_non_imitating_monster_at(next_spot) => {
				let mon_id: MonsterIndex = game.mon_id_at(next_spot);
				return ThrowEnding::ReachesMonster { mon_spot: next_spot, mon_id, flight_path, rogue_spot };
			}
			Some(next_spot) => {
				if game.cell_at(next_spot).is_any_tunnel() {
					// This causes the thrown object to land closer to the rogue when flying through a tunnel.
					timeout += 2;
				}
				flight_path.push(next_spot);
			}
		}
	}
	let landing_spot = *flight_path.last().expect("spot in flight path");
	ThrowEnding::LandsOnGround { landing_spot, flight_path, rogue_spot }
}

pub enum ThrowEnding {
	StrikesWall { bounce_spot: DungeonSpot, flight_path: Vec<DungeonSpot>, rogue_spot: DungeonSpot },
	ReachesMonster { mon_spot: DungeonSpot, mon_id: u64, flight_path: Vec<DungeonSpot>, rogue_spot: DungeonSpot },
	LandsOnGround { landing_spot: DungeonSpot, flight_path: Vec<DungeonSpot>, rogue_spot: DungeonSpot },
}

impl ThrowEnding {
	pub fn flight_path(&self) -> &Vec<DungeonSpot> {
		match self {
			ThrowEnding::StrikesWall { flight_path, .. } => flight_path,
			ThrowEnding::ReachesMonster { flight_path, .. } => flight_path,
			ThrowEnding::LandsOnGround { flight_path, .. } => flight_path,
		}
	}
}

fn flop_object_from_spot(obj_id: ObjectId, spot: DungeonSpot, game: &mut GameState) {
	let row = spot.row;
	let col = spot.col;
	let mut found_good_spot = false;
	let mut walk = RandomWalk::new(row, col);
	fn good_cell(cell: DungeonCell) -> bool {
		!(cell.has_object() || cell.is_any_trap() || cell.is_stairs() || cell.is_any_hidden())
			&& (cell.is_any_floor() || cell.is_any_tunnel() || cell.is_any_door() || cell.has_monster())
	}
	for _ in 0..9 {
		let cell = game.level.dungeon[walk.spot().row as usize][walk.spot().col as usize];
		if good_cell(cell) {
			break;
		}
		walk.step();
		let spot = walk.spot();
		let spot_cell = game.level.dungeon[spot.row as usize][spot.col as usize];
		if spot.is_out_of_bounds() || spot_cell.is_nothing() || !good_cell(spot_cell) {
			continue;
		}
		found_good_spot = true;
		break;
	}
	let spot = walk.spot().clone();
	if found_good_spot || walk.steps_taken == 0 {
		let mut new_obj = {
			let obj = game.player.object(obj_id).expect("obj in pack");
			obj.clone_with_new_id(&mut thread_rng())
		};
		{
			new_obj.in_use_flags = NOT_USED;
			new_obj.quantity = 1;
			new_obj.ichar = 'L';
		}
		place_at(new_obj, spot.row, spot.col, &mut game.level, &mut game.ground);
		game.render_spot(spot);
	} else {
		let obj_name = game.player.to_object_name_with_quantity(obj_id, 1);
		let msg = format!("the {}vanishes as it hits the ground", obj_name);
		game.diary.add_entry(&msg);
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Motion {
	DownRight,
	DownLeft,
	UpRight,
	UpLeft,
	Right,
	Down,
	Same,
	Up,
	Left,
}

impl From<char> for Motion {
	fn from(value: char) -> Self {
		match value {
			'n' => DownRight,
			'b' => DownLeft,
			'u' => UpRight,
			'y' => UpLeft,
			'l' => Right,
			'k' => Down,
			' ' => Same,
			'j' => Up,
			'h' => Left,
			_ => panic!("Invalid char '{value}' for Motion")
		}
	}
}

impl Motion {
	pub fn delta(&self) -> (isize, isize) {
		match self {
			DownRight => (1, 1),
			DownLeft => (1, -1),
			UpRight => (-1, 1),
			UpLeft => (-1, -1),
			Right => (0, 1),
			Down => (1, 0),
			Same => (0, 0),
			Up => (-1, 0),
			Left => (0, -1),
		}
	}
	pub fn random8(rng: &mut impl Rng) -> Self {
		match rng.gen_range(1..=8) {
			1 => Up,
			2 => Down,
			3 => Right,
			4 => Left,
			5 => UpLeft,
			6 => UpRight,
			7 => DownLeft,
			8 => DownRight,
			_ => unreachable!("out of bounds")
		}
	}

	pub fn to_char(&self) -> char {
		match self {
			DownRight => 'n',
			DownLeft => 'b',
			UpRight => 'u',
			UpLeft => 'y',
			Right => 'l',
			Down => 'k',
			Same => ' ',
			Up => 'j',
			Left => 'h',
		}
	}
	pub fn apply(&self, row: i64, col: i64) -> (i64, i64) {
		let (r_delta, c_delta) = self.delta();
		(row + r_delta as i64, col + c_delta as i64)
	}
}

pub struct RandomWalk {
	moves: [Motion; 9],
	spot: DungeonSpot,
	pub steps_taken: usize,
}

impl RandomWalk {
	pub fn new(row: i64, col: i64) -> Self {
		let mut moves: [Motion; 9] = [Left, Up, DownLeft, UpLeft, Right, Down, UpRight, Same, DownRight];
		moves.shuffle(&mut thread_rng());
		RandomWalk { spot: DungeonSpot { row, col }, moves, steps_taken: 0 }
	}
	pub fn step(&mut self) {
		if self.steps_taken < self.moves.len() {
			let (row, col) = self.moves[self.steps_taken].apply(self.spot.row, self.spot.col);
			self.spot.row = row;
			self.spot.col = col;
			self.steps_taken += 1;
		}
	}
	pub fn spot(&self) -> &DungeonSpot { &self.spot }
}
