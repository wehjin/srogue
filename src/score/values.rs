use crate::objects::obj;

impl obj {
	pub fn weapon_value(&self) -> i16 {
		let mut val = self.weapon_kind().expect("weapon kind").value();
		if self.is_arrow_or_throwing_weapon() {
			val *= self.quantity;
		}
		val += self.d_enchant as i16 * 85;
		val += self.hit_enchant * 85;
		val
	}
}
