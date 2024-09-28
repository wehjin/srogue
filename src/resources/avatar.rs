use crate::init::GameState;
use crate::objects::{Object, ObjectId, ObjectPack};
use crate::pack::mask_pack;
use crate::player::rings::HandUsage;
use crate::player::{Player, RogueHealth};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::PackFilter::Amulets;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::state::RunState;
use crate::resources::rogue::fighter::Fighter;
use crate::resources::rogue::spot::RogueSpot;
use crate::ring::effects::RingEffects;
use crate::ring::PlayerHand;
use crate::settings::Settings;

pub trait Avatar {
	fn as_settings(&self) -> &Settings;
	fn as_settings_mut(&mut self) -> &mut Settings;
	fn set_rogue_row_col(&mut self, row: i64, col: i64);
	fn as_ring_effects(&self) -> &RingEffects;

	fn as_health(&self) -> &RogueHealth;
	fn as_health_mut(&mut self) -> &mut RogueHealth;

	fn wizard(&self) -> bool;
	fn rogue_depth(&self) -> usize;
	fn max_depth(&self) -> usize;
	fn has_amulet(&self) -> bool;

	fn fight_monster(&self) -> Option<u64>;
	fn set_fight_monster(&mut self, value: Option<u64>);

	fn as_fighter(&self) -> &Fighter;
	fn as_fighter_mut(&mut self) -> &mut Fighter;

	fn exp(&self) -> isize {
		self.as_fighter().exp.level as isize
	}
	fn buffed_exp(&self) -> isize {
		self.as_ring_effects().apply_dexterity(self.exp())
	}
	fn debuf_exp(&self) -> isize {
		self.hand_usage().count_hands()
	}
	fn ring_id(&self, hand: PlayerHand) -> Option<ObjectId> {
		match hand {
			PlayerHand::Left => self.as_fighter().left_ring,
			PlayerHand::Right => self.as_fighter().right_ring,
		}
	}
	fn set_ring_id(&mut self, ring_id: ObjectId, hand: PlayerHand) {
		let fighter = self.as_fighter_mut();
		match hand {
			PlayerHand::Left => fighter.left_ring = Some(ring_id),
			PlayerHand::Right => fighter.right_ring = Some(ring_id),
		}
	}
	fn clear_ring_id(&mut self, hand: PlayerHand) {
		let fighter = self.as_fighter_mut();
		match hand {
			PlayerHand::Left => fighter.left_ring = None,
			PlayerHand::Right => fighter.right_ring = None,
		};
	}
	fn hand_usage(&self) -> HandUsage {
		let left = self.ring_id(PlayerHand::Left).is_some();
		let right = self.ring_id(PlayerHand::Right).is_some();
		match (left, right) {
			(true, true) => HandUsage::Both,
			(true, false) => HandUsage::Left,
			(false, true) => HandUsage::Right,
			(false, false) => HandUsage::None,
		}
	}
	fn as_rogue_pack(&self) -> &ObjectPack {
		&self.as_fighter().pack
	}

	fn armor(&self) -> Option<&Object> {
		match self.as_fighter().armor {
			Some(id) => self.as_rogue_pack().object_if_what(id, ObjectWhat::Armor),
			None => None,
		}
	}
}

impl Avatar for Player {
	fn as_settings(&self) -> &Settings { &self.settings }
	fn as_settings_mut(&mut self) -> &mut Settings { &mut self.settings }
	fn set_rogue_row_col(&mut self, row: i64, col: i64) {
		self.rogue.row = row;
		self.rogue.col = col;
	}
	fn as_ring_effects(&self) -> &RingEffects {
		&self.ring_effects
	}
	fn as_health(&self) -> &RogueHealth { &self.health }
	fn as_health_mut(&mut self) -> &mut RogueHealth { &mut self.health }

	fn wizard(&self) -> bool {
		self.wizard
	}

	fn rogue_depth(&self) -> usize {
		self.cur_depth
	}
	fn max_depth(&self) -> usize { self.max_depth }

	fn has_amulet(&self) -> bool {
		mask_pack(&self.rogue.pack, Amulets)
	}

	fn fight_monster(&self) -> Option<u64> {
		self.fight_monster
	}

	fn set_fight_monster(&mut self, value: Option<u64>) {
		self.fight_monster = value;
	}

	fn as_fighter(&self) -> &Fighter {
		&self.rogue
	}
	fn as_fighter_mut(&mut self) -> &mut Fighter {
		&mut self.rogue
	}
}

impl Avatar for GameState {
	fn as_settings(&self) -> &Settings { self.player.as_settings() }
	fn as_settings_mut(&mut self) -> &mut Settings { self.player.as_settings_mut() }

	fn set_rogue_row_col(&mut self, row: i64, col: i64) { self.player.set_rogue_row_col(row, col) }
	fn as_ring_effects(&self) -> &RingEffects {
		self.player.as_ring_effects()
	}
	fn as_health(&self) -> &RogueHealth { self.player.as_health() }

	fn as_health_mut(&mut self) -> &mut RogueHealth { self.player.as_health_mut() }

	fn wizard(&self) -> bool {
		self.player.wizard
	}

	fn rogue_depth(&self) -> usize {
		self.player.rogue_depth()
	}

	fn max_depth(&self) -> usize {
		self.player.max_depth
	}

	fn has_amulet(&self) -> bool {
		self.player.has_amulet()
	}

	fn fight_monster(&self) -> Option<u64> {
		self.player.fight_monster
	}

	fn set_fight_monster(&mut self, value: Option<u64>) {
		self.player.set_fight_monster(value)
	}

	fn as_fighter(&self) -> &Fighter {
		self.player.as_fighter()
	}

	fn as_fighter_mut(&mut self) -> &mut Fighter {
		self.player.as_fighter_mut()
	}
}

impl Avatar for RunState {
	fn as_settings(&self) -> &Settings { &self.settings }
	fn as_settings_mut(&mut self) -> &mut Settings { &mut self.settings }
	fn set_rogue_row_col(&mut self, row: i64, col: i64) {
		let spot = LevelSpot::from_i64(row, col);
		self.level.rogue.spot = RogueSpot::from_spot(spot, &self.level);
	}

	fn as_ring_effects(&self) -> &RingEffects {
		&self.level.rogue.ring_effects
	}

	fn as_health(&self) -> &RogueHealth {
		&self.level.rogue.health
	}

	fn as_health_mut(&mut self) -> &mut RogueHealth {
		&mut self.level.rogue.health
	}

	fn wizard(&self) -> bool {
		self.stats.wizard
	}

	fn rogue_depth(&self) -> usize {
		self.level.rogue.depth.usize()
	}

	fn max_depth(&self) -> usize {
		self.level.rogue.depth.max()
	}

	fn has_amulet(&self) -> bool {
		self.level.rogue.has_amulet
	}

	fn fight_monster(&self) -> Option<u64> {
		self.level.rogue.fight_monster
	}

	fn set_fight_monster(&mut self, value: Option<u64>) {
		self.level.rogue.fight_monster = value;
	}

	fn as_fighter(&self) -> &Fighter {
		&self.level.rogue.fighter
	}

	fn as_fighter_mut(&mut self) -> &mut Fighter {
		&mut self.level.rogue.fighter
	}
}