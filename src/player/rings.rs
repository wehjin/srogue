use crate::objects::{Object, ObjectId};
use crate::player::Player;
use crate::resources::avatar::Avatar;
use crate::ring::PlayerHand;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HandUsage {
	None,
	Left,
	Right,
	Both,
}

impl HandUsage {
	pub fn count_hands(&self) -> isize {
		match self {
			HandUsage::None => 0,
			HandUsage::Left => 1,
			HandUsage::Right => 1,
			HandUsage::Both => 2,
		}
	}
}

impl Player {
	pub fn hand_is_free(&self, hand: PlayerHand) -> bool { self.ring_id(hand).is_none() }
	pub fn ring_hand(&self, ring_id: ObjectId) -> Option<PlayerHand> {
		let target = Some(ring_id);
		for ring_hand in PlayerHand::ALL_HANDS {
			if self.ring_id(ring_hand) == target {
				return Some(ring_hand);
			}
		}
		None
	}
	pub fn check_ring(&self, hand: PlayerHand, f: impl Fn(&Object) -> bool) -> bool {
		if let Some(ring) = self.ring(hand) {
			f(ring)
		} else {
			false
		}
	}
	pub fn ring(&self, hand: PlayerHand) -> Option<&Object> {
		if let Some(id) = self.ring_id(hand) {
			self.object(id)
		} else {
			None
		}
	}
	pub fn ring_mut(&mut self, hand: PlayerHand) -> Option<&mut Object> {
		if let Some(id) = self.ring_id(hand) {
			self.object_mut(id)
		} else {
			None
		}
	}
	pub fn put_ring(&mut self, ring_id: ObjectId, hand: PlayerHand) {
		let ring = self.object_mut(ring_id).expect("ring in pack");
		ring.in_use_flags |= hand.use_flag();
		self.set_ring_id(ring_id, hand);
	}
	pub fn un_put_ring(&mut self, hand: PlayerHand) -> Option<ObjectId> {
		let un_put_id = if let Some(ring) = self.ring_mut(hand) {
			ring.in_use_flags &= !hand.use_flag();
			Some(ring.id())
		} else {
			None
		};
		self.clear_ring_id(hand);
		un_put_id
	}
}