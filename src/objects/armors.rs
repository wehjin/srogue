use crate::armors::ArmorKind;
use crate::objects::obj;
use crate::prelude::object_what::ObjectWhat::{Armor};

impl obj {
	pub fn armor_kind(&self) -> Option<ArmorKind> {
		if self.what_is == Armor {
			Some(ArmorKind::from_index(self.which_kind as usize))
		} else {
			None
		}
	}
}
