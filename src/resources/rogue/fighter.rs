use crate::armors::constants::RINGMAIL;
use crate::level::RogueExp;
use crate::objects::{alloc_object, Object, ObjectId, ObjectPack};
use crate::player::constants::INIT_HP;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN};
use crate::prelude::object_what::ObjectWhat::{Armor, Weapon};
use crate::random::get_rand;
use crate::weapons::constants::{ARROW, BOW, MACE};
use libc::c_short;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct Fighter {
	pub armor: Option<ObjectId>,
	pub weapon: Option<ObjectId>,
	pub left_ring: Option<ObjectId>,
	pub right_ring: Option<ObjectId>,
	pub hp_current: isize,
	pub hp_max: isize,
	pub str_current: isize,
	pub str_max: isize,
	pub pack: ObjectPack,
	pub gold: usize,
	pub exp: RogueExp,
	pub row: i64,
	pub col: i64,
	pub moves_left: isize,
}

impl Fighter {
	pub fn do_wield(&mut self, obj_id: ObjectId) {
		self.weapon = Some(obj_id);
		let obj = self.pack.object_mut(obj_id).expect("wield obj not in pack");
		obj.in_use_flags |= BEING_WIELDED;
	}
	pub fn do_wear(&mut self, obj_id: ObjectId) {
		self.armor = Some(obj_id);
		let obj = self.pack.object_mut(obj_id).expect("wear obj not in pack");
		obj.in_use_flags |= BEING_WORN;
		obj.identified = true;
	}
	pub fn provision(&mut self, rng: &mut impl Rng) {
		self.pack.clear();
		// Food
		{
			self.pack.combine_or_add_item(Object::roll_food(true, rng));
		}
		// Armor
		{
			let mut obj = alloc_object(rng);
			obj.what_is = Armor;
			obj.which_kind = RINGMAIL;
			obj.class = RINGMAIL as isize + 2;
			obj.is_protected = 0;
			obj.d_enchant = 1;
			let added = self.pack.combine_or_add_item(obj);
			self.do_wear(added);
		}
		// Mace
		{
			let mut obj = alloc_object(rng);
			obj.what_is = Weapon;
			obj.which_kind = MACE;
			obj.hit_enchant = 1;
			obj.d_enchant = 1;
			obj.identified = true;
			let added = self.pack.combine_or_add_item(obj);
			self.do_wield(added);
		}
		// Bow
		{
			let mut obj = alloc_object(rng);
			obj.what_is = Weapon;
			obj.which_kind = BOW;
			obj.hit_enchant = 1;
			obj.d_enchant = 0;
			obj.identified = true;
			self.pack.combine_or_add_item(obj);
		}
		// Arrows
		{
			let mut obj = alloc_object(rng);
			obj.what_is = Weapon;
			obj.which_kind = ARROW;
			obj.quantity = get_rand(25, 35) as c_short;
			obj.hit_enchant = 0;
			obj.d_enchant = 0;
			obj.identified = true;
			self.pack.combine_or_add_item(obj);
		}
	}
}

impl Default for Fighter {
	fn default() -> Self {
		Self {
			armor: None,
			weapon: None,
			left_ring: None,
			right_ring: None,
			hp_current: INIT_HP,
			hp_max: INIT_HP,
			str_current: 16,
			str_max: 16,
			pack: ObjectPack::new(),
			gold: 0,
			exp: RogueExp::new(),
			row: 0,
			col: 0,
			moves_left: 1250,
		}
	}
}
