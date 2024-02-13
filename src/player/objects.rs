use crate::objects::{obj, ObjectId};
use crate::player::Player;
use crate::random::get_rand;

impl Player {
	pub unsafe fn to_object_name_with_quantity(&self, obj_id: ObjectId, quantity: i16) -> String {
		let obj = self.object(obj_id).expect("obj in pack");
		obj.to_name_with_new_quantity(quantity, self.settings.fruit.to_string(), &self.notes)
	}
	pub fn expect_object(&self, obj_id: ObjectId) -> &obj {
		self.object(obj_id).expect("obj in pack")
	}
	pub fn expect_object_mut(&mut self, obj_id: ObjectId) -> &mut obj {
		self.object_mut(obj_id).expect("obj in pack")
	}
	pub fn object(&self, obj_id: ObjectId) -> Option<&obj> {
		self.rogue.pack.object(obj_id)
	}
	pub fn object_mut(&mut self, obj_id: ObjectId) -> Option<&mut obj> {
		self.rogue.pack.object_mut(obj_id)
	}
	pub fn object_ids(&self) -> Vec<ObjectId> {
		self.rogue.pack.object_ids()
	}
	pub fn object_ids_when(&self, f: impl Fn(&obj) -> bool) -> Vec<ObjectId> {
		self.rogue.pack.object_ids_when(f)
	}
	pub fn random_unused_object_id(&self) -> Option<ObjectId> {
		let unused = self.object_ids_when(|obj| !obj.is_being_used());
		match unused.len() {
			0 => None,
			1 => Some(unused[0]),
			_ => Some(unused[get_rand(0, unused.len() - 1)]),
		}
	}
}
