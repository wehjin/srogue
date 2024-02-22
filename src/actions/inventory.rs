use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::inventory;
use crate::inventory::inventory;
use crate::player::rings::HandUsage;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::ring::PlayerHand;
use crate::systems::play_level::PlayResult;

pub struct Inventory;

impl PlayerAction for Inventory {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		inventory(AllObjects, game);
		None
	}
}

pub struct InventoryArmor;

impl PlayerAction for InventoryArmor {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		inv_armor_weapon(false, game);
		None
	}
}

pub struct InventoryWeapons;

impl PlayerAction for InventoryWeapons {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		inv_armor_weapon(true, game);
		None
	}
}

pub struct InventoryRings;

impl PlayerAction for InventoryRings {
	fn update(_input_key: char, game: &mut GameState) -> Option<PlayResult> {
		inv_rings(game);
		None
	}
}

 fn inv_armor_weapon(is_weapon: bool, game: &mut GameState) {
	if is_weapon {
		if let Some(weapon) = game.player.weapon() {
			inventory::single_inv(Some(weapon.ichar), game);
		} else {
			game.dialog.message("not wielding anything", 0);
		}
	} else {
		if let Some(armor) = game.player.armor() {
			inventory::single_inv(Some(armor.ichar), game);
		} else {
			game.dialog.message("not wearing anything", 0);
		}
	}
}

pub fn inv_rings(game: &mut GameState) {
	let hand_usage = game.player.hand_usage();
	if hand_usage == HandUsage::None {
		game.dialog.message("not wearing any rings", 0);
	} else {
		for ring_hand in PlayerHand::ALL_HANDS {
			if let Some(ring_id) = game.player.ring_id(ring_hand) {
				let msg = game.player.get_obj_desc(ring_id);
				game.dialog.message(&msg, 0);
			}
		}
	}
	if game.player.wizard {
		game.dialog.message(
			&format!("ste {}, r_r {}, e_r {}, r_t {}, s_s {}, a_s {}, reg {}, r_e {}, s_i {}, m_a {}, aus {}",
			         game.player.ring_effects.stealthy(), hand_usage.count_hands(),
			         game.player.ring_effects.calorie_burn(), game.player.ring_effects.has_teleport(),
			         game.player.ring_effects.has_sustain_strength(), game.player.ring_effects.add_strength(),
			         game.player.ring_effects.regeneration(), game.player.ring_effects.dexterity(),
			         game.player.ring_effects.has_see_invisible(), game.player.ring_effects.has_maintain_armor(),
			         game.player.ring_effects.auto_search()),
			0,
		);
	}
}
