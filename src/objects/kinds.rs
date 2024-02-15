use crate::armors::ArmorKind;
use crate::objects::Object;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::ObjectWhat::{Armor, Wand, Weapon};
use crate::potions::kind::PotionKind;
use crate::ring::ring_kind::RingKind;
use crate::zap::wand_kind::WandKind;
use crate::weapons::kind::WeaponKind;

impl Object {
	pub fn weapon_kind(&self) -> Option<WeaponKind> {
		if self.what_is == Weapon {
			Some(WeaponKind::from(self.which_kind))
		} else {
			None
		}
	}
	pub fn armor_kind(&self) -> Option<ArmorKind> {
		if self.what_is == Armor {
			Some(ArmorKind::from_index(self.which_kind as usize))
		} else {
			None
		}
	}
	pub fn wand_kind(&self) -> Option<WandKind> {
		if self.what_is == Wand {
			Some(WandKind::from_index(self.which_kind as usize))
		} else {
			None
		}
	}
	pub fn potion_kind(&self) -> Option<PotionKind> {
		if self.what_is == ObjectWhat::Potion {
			Some(PotionKind::from_index(self.which_kind as usize))
		} else {
			None
		}
	}
	pub fn ring_kind(&self) -> Option<RingKind> {
		if self.what_is == ObjectWhat::Ring {
			Some(RingKind::from_index(self.which_kind as usize))
		} else {
			None
		}
	}
}

