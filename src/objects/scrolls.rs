use crate::objects::obj;
use crate::prelude::object_what::ObjectWhat;
use crate::scrolls::ScrollKind;

impl obj {
	pub fn scroll_kind(&self) -> Option<ScrollKind> {
		if self.what_is == ObjectWhat::Scroll {
			Some(ScrollKind::from_index(self.which_kind as usize))
		} else {
			None
		}
	}
}