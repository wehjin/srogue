use crate::armors::ArmorKind;
use crate::objects::{obj};
use crate::weapons::WeaponKind;

use crate::zap::wand_kind::WandKind;

impl obj {
	pub fn weapon_value(&self) -> i16 {
		let mut value = self.weapon_kind().expect("weapon kind").sale_value();
		if self.is_arrow_or_throwing_weapon() {
			value *= self.quantity;
		}
		value += self.d_enchant as i16 * 85;
		value += self.hit_enchant * 85;
		value
	}
	pub fn armor_value(&self) -> i16 {
		let mut value = self.armor_kind().expect("armor kind").sale_value();
		value += self.d_enchant as i16 * 75;
		if self.is_protected != 0 {
			value += 200;
		}
		value
	}
	pub fn wand_value(&self) -> i16 {
		let mut value = self.wand_kind().expect("wand kind").sale_value();
		value *= (self.class as i16) + 1;
		value
	}
}

pub trait Sellable {
	fn sale_value(&self) -> i16;
}

impl Sellable for WandKind {
	fn sale_value(&self) -> i16 {
		match self {
			WandKind::TeleAway => 25,
			WandKind::SlowMonster => 50,
			WandKind::ConfuseMonster => 45,
			WandKind::Invisibility => 8,
			WandKind::Polymorph => 55,
			WandKind::HasteMonster => 2,
			WandKind::PutToSleep => 25,
			WandKind::MagicMissile => 20,
			WandKind::Cancellation => 20,
			WandKind::DoNothing => 0,
		}
	}
}

impl Sellable for ArmorKind {
	fn sale_value(&self) -> i16 {
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

impl Sellable for WeaponKind {
	fn sale_value(&self) -> i16 {
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
