use crate::init::GameState;
use crate::objects::note_tables::NoteTables;
use crate::objects::{Object, ObjectId, ObjectPack};
use crate::pack::mask_pack;
use crate::player::rings::HandUsage;
use crate::player::{Player, RogueHealth};
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::PackFilter::Amulets;
use crate::resources::level::size::LevelSpot;
use crate::resources::play::state::RunState;
use crate::resources::rogue::energy::RogueEnergy;
use crate::resources::rogue::fighter::Fighter;
use crate::resources::rogue::spot::RogueSpot;
use crate::ring::effects::RingEffects;
use crate::ring::PlayerHand;
use crate::settings::Settings;
use crate::weapons::kind::WeaponKind;

pub trait Avatar {
	fn pack_object(&self, obj_id: ObjectId) -> &Object {
		self.as_rogue_pack().object(obj_id).unwrap()
	}
	fn pack_object_mut(&mut self, obj_id: ObjectId) -> &mut Object {
		self.as_rogue_pack_mut().object_mut(obj_id).unwrap()
	}

	fn as_settings(&self) -> &Settings;
	fn as_settings_mut(&mut self) -> &mut Settings;
	fn rogue_energy(&self) -> RogueEnergy {
		RogueEnergy::from_moves(self.as_fighter().moves_left)
	}
	fn as_notes(&self) -> &NoteTables;
	fn as_notes_mut(&mut self) -> &mut NoteTables;

	fn set_rogue_row_col(&mut self, row: i64, col: i64);
	fn as_ring_effects(&self) -> &RingEffects;

	fn as_health(&self) -> &RogueHealth;
	fn as_health_mut(&mut self) -> &mut RogueHealth;

	fn wizard(&self) -> bool;
	fn rogue_depth(&self) -> usize;
	fn max_depth(&self) -> usize;
	fn has_amulet(&self) -> bool;

	fn fight_to_death(&self) -> Option<u64>;
	fn set_fight_to_death(&mut self, value: Option<u64>);

	fn as_fighter(&self) -> &Fighter;
	fn as_fighter_mut(&mut self) -> &mut Fighter;

	fn upgrade_hp(&mut self, hp: isize) {
		let fighter = self.as_fighter_mut();
		fighter.hp_max = (fighter.hp_max + hp).max(1);
		fighter.hp_current = (fighter.hp_current + hp).max(1);
	}

	fn exp(&self) -> isize {
		self.as_fighter().exp.level as isize
	}
	fn buffed_exp(&self) -> isize {
		self.as_ring_effects().apply_dexterity(self.exp())
	}
	fn debuf_exp(&self) -> isize {
		self.hand_usage().count_hands()
	}
	fn buffed_strength(&self) -> isize {
		self.as_ring_effects().apply_add_strength(self.cur_strength())
	}
	fn cur_strength(&self) -> isize { self.as_fighter().str_current }
	fn max_strength(&self) -> isize { self.as_fighter().str_max }

	fn weapon_id(&self) -> Option<ObjectId> { self.as_fighter().weapon }
	fn weapon_kind(&self) -> Option<WeaponKind> {
		self.weapon().map(|it| it.weapon_kind()).flatten()
	}
	fn weapon(&self) -> Option<&Object> {
		if let Some(id) = self.weapon_id() {
			self.as_rogue_pack().object_if_what(id, ObjectWhat::Weapon)
		} else {
			None
		}
	}
	fn weapon_mut(&mut self) -> Option<&mut Object> {
		if let Some(id) = self.weapon_id() {
			self.as_rogue_pack_mut().object_if_what_mut(id, ObjectWhat::Weapon)
		} else {
			None
		}
	}
	fn unwield_weapon(&mut self) {
		if let Some(obj) = self.weapon_mut() {
			obj.in_use_flags &= !BEING_WIELDED;
		}
		let fighter = self.as_fighter_mut();
		fighter.weapon = None;
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
	fn ring_hand(&self, ring_id: ObjectId) -> Option<PlayerHand> {
		let target = Some(ring_id);
		for ring_hand in PlayerHand::ALL_HANDS {
			if self.ring_id(ring_hand) == target {
				return Some(ring_hand);
			}
		}
		None
	}
	fn ring(&self, hand: PlayerHand) -> Option<&Object> {
		if let Some(id) = self.ring_id(hand) {
			self.as_rogue_pack().object(id)
		} else {
			None
		}
	}
	fn ring_mut(&mut self, hand: PlayerHand) -> Option<&mut Object> {
		if let Some(id) = self.ring_id(hand) {
			self.as_rogue_pack_mut().object_mut(id)
		} else {
			None
		}
	}
	fn check_ring(&self, hand: PlayerHand, f: impl Fn(&Object) -> bool) -> bool {
		if let Some(ring) = self.ring(hand) {
			f(ring)
		} else {
			false
		}
	}
	fn un_put_ring(&mut self, hand: PlayerHand) {
		if let Some(ring) = self.ring_mut(hand) {
			ring.in_use_flags &= !hand.use_flag();
		};
		self.clear_ring_id(hand);
		// TODO ring_stats(true, game);
	}
	fn as_rogue_pack(&self) -> &ObjectPack {
		&self.as_fighter().pack
	}
	fn as_rogue_pack_mut(&mut self) -> &mut ObjectPack {
		&mut self.as_fighter_mut().pack
	}

	fn armor(&self) -> Option<&Object> {
		match self.as_fighter().armor {
			Some(id) => self.as_rogue_pack().object_if_what(id, ObjectWhat::Armor),
			None => None,
		}
	}
	fn armor_mut(&mut self) -> Option<&mut Object> {
		match self.as_fighter().armor {
			Some(id) => self.as_rogue_pack_mut().object_if_what_mut(id, ObjectWhat::Armor),
			None => None,
		}
	}
	fn unwear_armor(&mut self) {
		if let Some(armor) = self.armor_mut() {
			armor.in_use_flags &= !BEING_WORN;
		}
		let fighter = self.as_fighter_mut();
		fighter.armor = None;
	}

	fn pack_weight_with_new_object(&self, new_obj: Option<&Object>) -> usize {
		// TODO Revisit. This is kind of screwed up.
		let mut weight = 0;
		for obj in self.as_fighter().pack.objects() {
			weight += obj.pack_weight_with_new_obj(new_obj);
		}
		// Note: the original C code forgets to include weight of the new object.
		if let Some(new_obj) = new_obj {
			weight += new_obj.pack_weight_with_new_obj(None);
		}
		weight
	}
	fn combine_or_add_item_to_pack(&mut self, obj: Object) -> ObjectId {
		let fighter = self.as_fighter_mut();
		fighter.pack.combine_or_add_item(obj)
	}
}

impl Avatar for Player {
	fn as_settings(&self) -> &Settings { &self.settings }
	fn as_settings_mut(&mut self) -> &mut Settings { &mut self.settings }

	fn as_notes(&self) -> &NoteTables {
		&self.notes
	}

	fn as_notes_mut(&mut self) -> &mut NoteTables {
		&mut self.notes
	}

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

	fn fight_to_death(&self) -> Option<u64> {
		self.fight_monster
	}

	fn set_fight_to_death(&mut self, value: Option<u64>) {
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

	fn as_notes(&self) -> &NoteTables {
		self.player.as_notes()
	}

	fn as_notes_mut(&mut self) -> &mut NoteTables {
		self.player.as_notes_mut()
	}

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

	fn fight_to_death(&self) -> Option<u64> {
		self.player.fight_monster
	}

	fn set_fight_to_death(&mut self, value: Option<u64>) {
		self.player.set_fight_to_death(value)
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

	fn as_notes(&self) -> &NoteTables {
		&self.level.rogue.notes
	}

	fn as_notes_mut(&mut self) -> &mut NoteTables {
		&mut self.level.rogue.notes
	}

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
		self.level.rogue.wizard
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

	fn fight_to_death(&self) -> Option<u64> {
		self.level.rogue.fight_to_death
	}

	fn set_fight_to_death(&mut self, value: Option<u64>) {
		self.level.rogue.fight_to_death = value;
	}

	fn as_fighter(&self) -> &Fighter {
		&self.level.rogue.fighter
	}

	fn as_fighter_mut(&mut self) -> &mut Fighter {
		&mut self.level.rogue.fighter
	}
}