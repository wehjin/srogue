use crate::armors::ArmorKind;
use crate::objects::obj;
use crate::prelude::object_what::ObjectWhat::{Armor, Wand, Weapon};
use crate::zap::wand_kind::WandKind;
use crate::weapons::WeaponKind;

impl obj {
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
}

