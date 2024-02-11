use rand::{RngCore, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObjectId(u64);

impl ObjectId {
	pub fn random() -> Self {
		ObjectId(thread_rng().next_u64())
	}
}
