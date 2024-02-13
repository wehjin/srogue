use crate::armors::ArmorKind;
use crate::objects::obj;
use crate::prelude::object_what::ObjectWhat;
use crate::potions::kind::PotionKind;
use crate::ring::ring_kind::RingKind;
use crate::scrolls::ScrollKind;
use crate::weapons::kind::WeaponKind;
use crate::zap::wand_kind::WandKind;

impl obj {
	/// From score.c
	pub fn sale_value(&self) -> usize {
		let mut value = match self.what_is {
			ObjectWhat::Weapon => self.weapon_value(),
			ObjectWhat::Armor => self.armor_value(),
			ObjectWhat::Wand => self.wand_value(),
			ObjectWhat::Scroll => self.scroll_value(),
			ObjectWhat::Potion => self.potion_value(),
			ObjectWhat::Amulet => 5000,
			ObjectWhat::Ring => self.ring_value(),
			ObjectWhat::Gold => 0,
			ObjectWhat::Food => 0,
			ObjectWhat::None => 0,
		};
		if value <= 0 {
			value = 10;
		}
		value as usize
	}
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
	pub fn scroll_value(&self) -> i16 {
		let mut value = self.scroll_kind().expect("scroll kind").sale_value();
		value *= self.quantity;
		value
	}
	pub fn potion_value(&self) -> i16 {
		let mut value = self.potion_kind().expect("potion kind").sale_value();
		value *= self.quantity;
		value
	}
	pub fn ring_value(&self) -> i16 {
		let mut value = self.ring_kind().expect("ring kind").sale_value();
		value *= (self.class as i16) + 1;
		value
	}
}

impl Sellable for RingKind {
	fn sale_value(&self) -> i16 {
		match self {
			RingKind::Stealth => 250,
			RingKind::RTeleport => 100,
			RingKind::Regeneration => 255,
			RingKind::SlowDigest => 295,
			RingKind::AddStrength => 200,
			RingKind::SustainStrength => 250,
			RingKind::Dexterity => 250,
			RingKind::Adornment => 25,
			RingKind::RSeeInvisible => 300,
			RingKind::MaintainArmor => 290,
			RingKind::Searching => 270,
		}
	}
}

pub trait Sellable {
	fn sale_value(&self) -> i16;
}

impl Sellable for PotionKind {
	fn sale_value(&self) -> i16 {
		match self {
			PotionKind::IncreaseStrength => 100,
			PotionKind::RestoreStrength => 250,
			PotionKind::Healing => 100,
			PotionKind::ExtraHealing => 200,
			PotionKind::Poison => 10,
			PotionKind::RaiseLevel => 300,
			PotionKind::Blindness => 10,
			PotionKind::Hallucination => 25,
			PotionKind::DetectMonster => 100,
			PotionKind::DetectObjects => 100,
			PotionKind::Confusion => 10,
			PotionKind::Levitation => 80,
			PotionKind::HasteSelf => 150,
			PotionKind::SeeInvisible => 145,
		}
	}
}

impl Sellable for ScrollKind {
	fn sale_value(&self) -> i16 {
		match self {
			ScrollKind::ProtectArmor => 505,
			ScrollKind::HoldMonster => 200,
			ScrollKind::EnchWeapon => 235,
			ScrollKind::EnchArmor => 235,
			ScrollKind::Identify => 175,
			ScrollKind::Teleport => 190,
			ScrollKind::Sleep => 25,
			ScrollKind::ScareMonster => 610,
			ScrollKind::RemoveCurse => 210,
			ScrollKind::CreateMonster => 100,
			ScrollKind::AggravateMonster => 25,
			ScrollKind::MagicMapping => 180,
		}
	}
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
