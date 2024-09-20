pub struct DungeonSettings {
	pub party_depth: usize,
}


pub fn roll_settings() -> DungeonSettings {
	let party_depth = party::roll_depth(0);
	DungeonSettings { party_depth }
}

pub mod party {
	use crate::random::get_rand;

	const PARTY_INTERVAL: usize = 10;
	pub fn roll_depth(rogue_depth: usize) -> usize {
		let base_level = (rogue_depth as f32 / PARTY_INTERVAL as f32).ceil() as usize * PARTY_INTERVAL;
		base_level + get_rand(1, PARTY_INTERVAL)
	}
}
