use crate::objects::{Object, ObjectId};
use crate::prelude::food_kind::FRUIT;
use crate::prelude::object_what::ObjectWhat;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct ObjectPack(Vec<Object>);

impl ObjectPack {
	pub fn combine_or_add_item(&mut self, mut obj: Object) -> ObjectId {
		if let Some(id) = self.try_combine(&obj) {
			return id;
		}
		let obj_id = obj.id();
		obj.ichar = self.next_pack_ichar();
		self.add(obj);
		obj_id
	}
	fn next_pack_ichar(&self) -> char {
		let mut used = [false; 26];
		for obj in self.objects() {
			let letter_index = (obj.ichar as u8 - 'a' as u8) as usize;
			used[letter_index] = true;
		}
		if let Some(unused) = used.into_iter().position(|used| used == false) {
			(unused as u8 + 'a' as u8) as char
		} else { '?' }
	}
	fn try_combine(&mut self, obj: &Object) -> Option<ObjectId> {
		let combinable = match obj.what_is {
			ObjectWhat::Weapon | ObjectWhat::Food | ObjectWhat::Scroll | ObjectWhat::Potion => true,
			_ => false,
		};
		if !combinable {
			return None;
		}
		if obj.what_is == ObjectWhat::Food && obj.which_kind == FRUIT {
			return None;
		}
		if let Some(found) = self.find_object_mut(|pack_obj| obj.can_join_existing_pack_object(pack_obj)) {
			found.quantity += obj.quantity;
			Some(found.id())
		} else {
			None
		}
	}
}

impl ObjectPack {
	pub const fn new() -> Self {
		ObjectPack(Vec::new())
	}
	pub fn add(&mut self, obj: Object) {
		self.0.push(obj);
	}
	pub fn remove(&mut self, obj_id: ObjectId) -> Option<Object> {
		let index = self.0.iter().position(|it| it.id() == obj_id);
		if let Some(index) = index {
			Some(self.0.remove(index))
		} else {
			None
		}
	}
	pub fn clear(&mut self) { self.0.clear(); }
	pub fn is_empty(&self) -> bool { self.0.is_empty() }
	pub fn find_id_at(&self, row: i64, col: i64) -> Option<ObjectId> {
		self.find_id(|obj| obj.is_at(row, col))
	}
	pub fn find_object_at(&self, row: i64, col: i64) -> Option<&Object> {
		self.find_id_at(row, col).map(|id| {
			self.object(id).expect("object in object_at")
		})
	}
	pub fn check_object(&self, obj_id: ObjectId, f: impl Fn(&Object) -> bool) -> bool {
		self.object(obj_id).map(f).unwrap_or(false)
	}
	pub fn try_map_object<T>(&self, obj_id: ObjectId, f: impl Fn(&Object) -> Option<T>) -> Option<T> {
		if let Some(obj) = self.object(obj_id) {
			f(obj)
		} else { None }
	}
	pub fn find_object(&self, f: impl Fn(&Object) -> bool) -> Option<&Object> {
		self.find_id(f).map(|id| self.object(id)).flatten()
	}
	pub fn find_object_mut(&mut self, f: impl Fn(&Object) -> bool) -> Option<&mut Object> {
		self.find_id(f).map(|id| self.object_mut(id)).flatten()
	}
	pub fn find_id(&self, f: impl Fn(&Object) -> bool) -> Option<ObjectId> {
		for obj in &self.0 {
			if f(obj) {
				return Some(obj.id());
			}
		}
		return None;
	}
	pub fn object_if_what(&self, id: ObjectId, what: ObjectWhat) -> Option<&Object> {
		self.object_if(id, |obj| obj.what_is == what)
	}
	pub fn object_if_what_mut(&mut self, id: ObjectId, what: ObjectWhat) -> Option<&mut Object> {
		self.object_if_mut(id, |obj| obj.what_is == what)
	}
	pub fn object_if(&self, obj_id: ObjectId, f: impl Fn(&Object) -> bool) -> Option<&Object> {
		if self.check_object(obj_id, f) {
			self.object(obj_id)
		} else {
			None
		}
	}
	pub fn object_if_mut(&mut self, obj_id: ObjectId, f: impl Fn(&Object) -> bool) -> Option<&mut Object> {
		if self.check_object(obj_id, f) {
			self.object_mut(obj_id)
		} else {
			None
		}
	}

	pub fn as_object(&self, obj_id: ObjectId) -> &Object {
		self.object(obj_id).unwrap()
	}
	pub fn object(&self, obj_id: ObjectId) -> Option<&Object> {
		if let Some(position) = self.obj_position(obj_id) {
			Some(&self.0[position])
		} else { None }
	}
	pub fn object_mut(&mut self, obj_id: ObjectId) -> Option<&mut Object> {
		if let Some(position) = self.obj_position(obj_id) {
			Some(&mut self.0[position])
		} else { None }
	}
	fn obj_position(&self, obj_id: ObjectId) -> Option<usize> {
		self.0.iter().position(|obj| obj.id() == obj_id)
	}
	pub fn object_ids(&self) -> Vec<ObjectId> {
		self.0.iter().map(Object::id).collect()
	}
	pub fn object_ids_when(&self, f: impl Fn(&Object) -> bool) -> Vec<ObjectId> {
		let mut result = Vec::new();
		for obj in self.objects() {
			if f(obj) {
				result.push(obj.id());
			}
		}
		result
	}
	pub fn objects(&self) -> &Vec<Object> { &self.0 }
	pub fn object_at_index_mut(&mut self, index: usize) -> &mut Object {
		&mut self.0[index]
	}
	pub fn len(&self) -> usize {
		self.0.len()
	}
}
