use crate::objects::{obj, ObjectId};
use crate::player::Player;
use crate::prelude::PlayerHand;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum HandUsage {
	None,
	Left,
	Right,
	Both,
}

impl Player {
	pub fn hand_usage(&self) -> HandUsage {
		let left = self.ring_id(PlayerHand::Left).is_some();
		let right = self.ring_id(PlayerHand::Right).is_some();
		match (left, right) {
			(true, true) => HandUsage::Both,
			(true, false) => HandUsage::Left,
			(false, true) => HandUsage::Right,
			(false, false) => HandUsage::None,
		}
	}
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
	pub fn check_ring(&self, hand: PlayerHand, f: impl Fn(&obj) -> bool) -> bool {
		if let Some(ring) = self.ring(hand) {
			f(ring)
		} else {
			false
		}
	}
	pub fn ring(&self, hand: PlayerHand) -> Option<&obj> {
		if let Some(id) = self.ring_id(hand) {
			self.object(id)
		} else {
			None
		}
	}
	pub fn ring_mut(&mut self, hand: PlayerHand) -> Option<&mut obj> {
		if let Some(id) = self.ring_id(hand) {
			self.object_mut(id)
		} else {
			None
		}
	}
	pub fn ring_id(&self, hand: PlayerHand) -> Option<ObjectId> {
		let ring_id = match hand {
			PlayerHand::Left => self.rogue.left_ring,
			PlayerHand::Right => self.rogue.right_ring,
		};
		ring_id
	}
	fn set_ring_id(&mut self, ring_id: ObjectId, hand: PlayerHand) {
		match hand {
			PlayerHand::Left => {
				self.rogue.left_ring = Some(ring_id);
			}
			PlayerHand::Right => {
				self.rogue.right_ring = Some(ring_id);
			}
		}
	}
	fn clear_ring_id(&mut self, hand: PlayerHand) {
		match hand {
			PlayerHand::Left => {
				self.rogue.left_ring = None;
			}
			PlayerHand::Right => {
				self.rogue.right_ring = None;
			}
		};
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