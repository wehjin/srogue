use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct ObjectId(u64);

impl ObjectId {
	pub fn new(id: u64) -> Self {
		Self(id)
	}
}

