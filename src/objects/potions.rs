use crate::objects::obj;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::potion_kind::PotionKind;

impl obj {
	pub fn potion_kind(&self) -> Option<PotionKind> {
		if self.what_is == ObjectWhat::Potion {
			let kind = PotionKind::from_index(self.which_kind as usize);
			Some(kind)
		} else {
			None
		}
	}
}