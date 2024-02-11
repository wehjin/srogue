use crate::objects::obj;
use crate::prelude::item_usage::BEING_WIELDED;

impl obj {
	pub fn is_wielded_throwing_weapon(&self) -> bool {
		self.is_wielded() && self.is_throwing_weapon()
	}
	pub fn is_throwing_weapon(&self) -> bool {
		if let Some(kind) = self.weapon_kind() {
			kind.is_throwing_weapon()
		} else {
			false
		}
	}
	pub fn is_arrow_or_throwing_weapon(&self) -> bool {
		if let Some(kind) = self.weapon_kind() {
			kind.is_arrow_or_throwing_weapon()
		} else {
			false
		}
	}
	pub fn is_weapon(&self) -> bool { self.weapon_kind().is_some() }
	pub fn is_wielded(&self) -> bool {
		(self.in_use_flags & BEING_WIELDED) != 0
	}
}
