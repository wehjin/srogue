use crate::armors::constants::{ARMORS, PLATE, SPLINT};
use crate::objects::Object;
use crate::potions::kind::PotionKind;
use crate::potions::kind::PotionKind::{Blindness, Confusion, DetectMonster, DetectObjects, ExtraHealing, Hallucination, Healing, IncreaseStrength, Levitation, Poison, RaiseLevel, RestoreStrength, SeeInvisible};
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::ring::gr_ring;
use crate::scrolls::ScrollKind;
use crate::weapons::constants::{ARROW, DAGGER, DART, SHURIKEN, WEAPONS};
use crate::zap::constants::WANDS;
use crate::zap::wand_kind::WandKind;

impl Object {
	pub fn roll_scroll() -> Object {
		let mut object = Object::new(Scroll);
		gr_scroll(&mut object);
		object
	}
	pub fn roll_potion() -> Object {
		let mut object = Object::new(Potion);
		gr_potion(&mut object);
		object
	}
	pub fn roll_weapon(assign_kind: bool) -> Object {
		let mut object = Object::new(Weapon);
		gr_weapon(&mut object, assign_kind);
		object
	}
	pub fn roll_armor() -> Object {
		let mut object = Object::new(Armor);
		gr_armor(&mut object);
		object
	}
	pub fn roll_wand() -> Object {
		let mut object = Object::new(Wand);
		gr_wand(&mut object);
		object
	}
	pub fn roll_food(force_ration: bool) -> Object {
		let mut object = Object::new(Food);
		get_food(&mut object, force_ration);
		object
	}
	pub fn roll_ring(assign_kind: bool) -> Object {
		let mut object = Object::new(Ring);
		gr_ring(&mut object, assign_kind);
		object
	}
	pub fn roll_gold(depth: usize, boosted: bool) -> Object {
		let mut object = Object::new(Gold);
		let boost = if boosted { 1.5 } else { 1.0 };
		let quantity = get_rand(2 * depth, 16 * depth) as f64 * boost;
		object.quantity = quantity as i16;
		object
	}
}

fn gr_scroll(obj: &mut Object) {
	let percent = get_rand(0, 85);
	obj.what_is = Scroll;

	let kind = if percent <= 5 {
		ScrollKind::ProtectArmor
	} else if percent <= 11 {
		ScrollKind::HoldMonster
	} else if percent <= 20 {
		ScrollKind::CreateMonster
	} else if percent <= 35 {
		ScrollKind::Identify
	} else if percent <= 43 {
		ScrollKind::Teleport
	} else if percent <= 50 {
		ScrollKind::Sleep
	} else if percent <= 55 {
		ScrollKind::ScareMonster
	} else if percent <= 64 {
		ScrollKind::RemoveCurse
	} else if percent <= 69 {
		ScrollKind::EnchArmor
	} else if percent <= 74 {
		ScrollKind::EnchWeapon
	} else if percent <= 80 {
		ScrollKind::AggravateMonster
	} else {
		ScrollKind::MagicMapping
	};
	obj.which_kind = kind.to_index() as u16;
}

fn gr_potion(obj: &mut Object) {
	obj.what_is = Potion;
	obj.which_kind = gr_potion_kind().to_index() as u16;
}

fn gr_potion_kind() -> PotionKind {
	let percent = get_rand(1, 118);
	let kind = if percent <= 5 {
		RaiseLevel
	} else if percent <= 15 {
		DetectObjects
	} else if percent <= 25 {
		DetectMonster
	} else if percent <= 35 {
		IncreaseStrength
	} else if percent <= 45 {
		RestoreStrength
	} else if percent <= 55 {
		Healing
	} else if percent <= 65 {
		ExtraHealing
	} else if percent <= 75 {
		Blindness
	} else if percent <= 85 {
		Hallucination
	} else if percent <= 95 {
		Confusion
	} else if percent <= 105 {
		Poison
	} else if percent <= 110 {
		Levitation
	} else if percent <= 114 {
		Hallucination
	} else {
		SeeInvisible
	};
	kind
}

pub fn gr_weapon(obj: &mut Object, assign_wk: bool) {
	obj.what_is = Weapon;
	if assign_wk {
		obj.which_kind = get_rand(0, (WEAPONS - 1) as u16);
	}
	if (obj.which_kind == ARROW) || (obj.which_kind == DAGGER) || (obj.which_kind == SHURIKEN) | (obj.which_kind == DART) {
		obj.quantity = get_rand(3, 15);
		obj.quiver = get_rand(0, 126);
	} else {
		obj.quantity = 1;
	}
	obj.hit_enchant = 0;
	obj.d_enchant = 0;

	let percent = get_rand(1, 96);
	let blessing = get_rand(1, 3);

	let mut increment = 0;
	if percent <= 16 {
		increment = 1;
	} else if percent <= 32 {
		increment = -1;
		obj.is_cursed = 1;
	}
	if percent <= 32 {
		for _ in 0..blessing {
			if coin_toss() {
				obj.hit_enchant += increment;
			} else {
				obj.d_enchant += increment as isize;
			}
		}
	}
}

pub fn gr_armor(obj: &mut Object) {
	obj.what_is = Armor;
	obj.which_kind = get_rand(0, (ARMORS - 1) as u16);
	obj.class = (obj.which_kind + 2) as isize;
	if obj.which_kind == PLATE || obj.which_kind == SPLINT {
		obj.class -= 1;
	}
	obj.is_protected = 0;
	obj.d_enchant = 0;

	let percent = get_rand(1, 100);
	let blessing = get_rand(1, 3);

	if percent <= 16 {
		obj.is_cursed = 1;
		obj.d_enchant -= blessing;
	} else if percent <= 33 {
		obj.raise_armor_enchant(blessing);
	}
}

pub fn gr_wand(obj: &mut Object) {
	obj.what_is = Wand;
	obj.which_kind = get_rand(0, (WANDS - 1) as u16);
	if obj.which_kind == WandKind::MagicMissile.to_index() as u16 {
		obj.class = get_rand(6, 12);
	} else if obj.which_kind == WandKind::Cancellation.to_index() as u16 {
		obj.class = get_rand(5, 9);
	} else {
		obj.class = get_rand(3, 6);
	}
}

pub fn get_food(obj: &mut Object, force_ration: bool) {
	obj.what_is = Food;
	if force_ration || rand_percent(80) {
		obj.which_kind = RATION;
	} else {
		obj.which_kind = FRUIT;
	}
}
