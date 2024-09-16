use crate::actions::GameUpdater;
use crate::init::GameState;
use crate::inventory::{inventory, single_inv, ObjectSource};
use crate::player::rings::HandUsage;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::ring::PlayerHand;
use crate::systems::play_level::{LevelResult, UNKNOWN_COMMAND};

pub struct InventoryGround;

impl GameUpdater for InventoryGround {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		if game.player.wizard {
			inventory(AllObjects, ObjectSource::Ground, game);
		} else {
			game.diary.add_entry(UNKNOWN_COMMAND);
		}
		None
	}
}

pub struct InventoryOne;

impl GameUpdater for InventoryOne {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		single_inv(None, game);
		None
	}
}

pub struct Inventory;

impl GameUpdater for Inventory {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		inventory(AllObjects, ObjectSource::Player, game);
		None
	}
}

pub struct InventoryArmor;

impl GameUpdater for InventoryArmor {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		inv_armor_weapon(false, game);
		None
	}
}

pub struct InventoryWeapons;

impl GameUpdater for InventoryWeapons {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		inv_armor_weapon(true, game);
		None
	}
}

pub struct InventoryRings;

impl GameUpdater for InventoryRings {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		inv_rings(game);
		None
	}
}

fn inv_armor_weapon(is_weapon: bool, game: &mut GameState) {
	if is_weapon {
		if let Some(weapon) = game.player.weapon() {
			single_inv(Some(weapon.ichar), game);
		} else {
			game.diary.add_entry("not wielding anything");
		}
	} else {
		if let Some(armor) = game.player.armor() {
			single_inv(Some(armor.ichar), game);
		} else {
			game.diary.add_entry("not wearing anything");
		}
	}
}

pub fn inv_rings(game: &mut GameState) {
	let hand_usage = game.player.hand_usage();
	if hand_usage == HandUsage::None {
		game.diary.add_entry("not wearing any rings");
	} else {
		for ring_hand in PlayerHand::ALL_HANDS {
			if let Some(ring_id) = game.player.ring_id(ring_hand) {
				let msg = game.player.get_obj_desc(ring_id);
				game.diary.add_entry(&msg);
			}
		}
	}
	if game.player.wizard {
		game.diary.add_entry(
			&format!("ste {}, r_r {}, e_r {}, r_t {}, s_s {}, a_s {}, reg {}, r_e {}, s_i {}, m_a {}, aus {}",
			         game.player.ring_effects.stealthy(), hand_usage.count_hands(),
			         game.player.ring_effects.calorie_burn(), game.player.ring_effects.has_teleport(),
			         game.player.ring_effects.has_sustain_strength(), game.player.ring_effects.add_strength(),
			         game.player.ring_effects.regeneration(), game.player.ring_effects.dexterity(),
			         game.player.ring_effects.has_see_invisible(), game.player.ring_effects.has_maintain_armor(),
			         game.player.ring_effects.auto_search()),
		);
	}
}
