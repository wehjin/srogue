pub mod party {
	use crate::random::get_rand;

	const PARTY_INTERVAL: usize = 10;
	pub fn roll_next_depth(previous_depth: usize) -> usize {
		let base_level = (previous_depth as f32 / PARTY_INTERVAL as f32).ceil() as usize * PARTY_INTERVAL;
		base_level + get_rand(1, PARTY_INTERVAL)
	}
}
