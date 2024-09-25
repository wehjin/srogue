use crate::init::GameState;
use crate::odds::{PARTY_WAKE_PERCENT, WAKE_PERCENT};
use crate::random::rand_percent;
use crate::resources::dice;
use crate::resources::level::room_id::RoomId;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::DungeonLevel;
use rand::Rng;

pub fn wake_room_legacy(rn: usize, entering: bool, row: i64, col: i64, game: &mut GameState) {
	let normal_chance = game.level.room_wake_percent(rn);
	let buffed_chance = game.player.ring_effects.apply_stealthy(normal_chance);
	for mon_id in game.mash.monster_ids() {
		let monster = game.mash.monster_mut(mon_id);
		if monster.in_room(rn, &game.level) {
			if entering {
				monster.clear_target();
			} else {
				monster.set_target_spot(row, col);
			}
			if monster.m_flags.wakens {
				if rand_percent(buffed_chance) {
					monster.wake_up();
				}
			}
		}
	}
}

pub enum WakeType {
	DropIn(Option<RoomId>),
	EnterVault(RoomId),
	ExitVault(RoomId, LevelSpot),
}
pub fn wake_room(wake_type: WakeType, level: &mut DungeonLevel, rng: &mut impl Rng) {
	let (wake_room, target_spot) = match wake_type {
		WakeType::DropIn(room) => (room, None),
		WakeType::EnterVault(room) => (Some(room), None),
		WakeType::ExitVault(room, spot) => (Some(room), Some(spot)),
	};
	if let Some(room) = wake_room {
		let chance = wake_chance(room, level);
		let buffed_chance = level.rogue.ring_effects.apply_stealthy(chance);
		for monster_spot in level.monster_spots_in(room) {
			let monster = level.as_monster_mut(monster_spot);
			if let Some(target) = target_spot {
				monster.set_target(target)
			} else {
				monster.clear_target()
			}
			if monster.wakens() {
				if dice::roll_chance(buffed_chance, rng) {
					monster.wake_up();
				}
			}
		}
	}
}

fn wake_chance(room: RoomId, level: &mut DungeonLevel) -> usize {
	if let Some(party_room) = level.party_room {
		if party_room == room {
			return PARTY_WAKE_PERCENT;
		}
	}
	WAKE_PERCENT
}
