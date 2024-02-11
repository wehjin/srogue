use crate::armors::ArmorKind;
use crate::objects::{obj};
use crate::weapons::WeaponKind;

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
	pub fn armor_value(&self) -> i16 {
		let mut val = self.armor_kind().expect("armor kind").value();
		val += self.d_enchant as i16 * 75;
		if self.is_protected != 0 {
			val += 200;
		}
		val
	}
}

impl WeaponKind {
	pub const fn value(&self) -> i16 {
		match self {
			WeaponKind::Bow => 150,
			WeaponKind::Dart => 8,
			WeaponKind::Arrow => 15,
			WeaponKind::Dagger => 27,
			WeaponKind::Shuriken => 35,
			WeaponKind::Mace => 360,
			WeaponKind::LongSword => 470,
			WeaponKind::TwoHandedSword => 580,
		}
	}
}

impl ArmorKind {
	pub const fn value(&self) -> i16 {
		match self {
			ArmorKind::Leather => 300,
			ArmorKind::Ringmail => 300,
			ArmorKind::Scale => 400,
			ArmorKind::Chain => 500,
			ArmorKind::Banded => 600,
			ArmorKind::Splint => 600,
			ArmorKind::Plate => 700,
		}
	}
}
