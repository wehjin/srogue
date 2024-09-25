use crate::init::GameState;
use crate::random::rand_percent;
use crate::resources::game::RogueSpot;
use crate::resources::level::room_id::RoomId;

pub fn wake_room(rn: usize, entering: bool, row: i64, col: i64, game: &mut GameState) {
	let normal_chance = game.level.room_wake_percent(rn);
	let buffed_chance = game.player.ring_effects.apply_stealthy(normal_chance);
	for mon_id in game.mash.monster_ids() {
		let monster = game.mash.monster_mut(mon_id);
		if monster.in_room(rn, &game.level) {
			if entering {
				monster.clear_target_spot();
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

pub fn wake_room2(rogue_spot: RogueSpot, party_room: Option<RoomId>) {


}

