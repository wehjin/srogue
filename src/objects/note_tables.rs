use rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::armors::constants::ARMORS;
use crate::objects::NoteStatus::{Called, Identified};
use crate::objects::{Note, NoteStatus, Title};
use crate::potions::kind::POTIONS;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::object_what::ObjectWhat::{Armor, Potion, Ring, Scroll, Wand, Weapon};
use crate::random::get_rand;
use crate::ring::constants::{ALL_RING_GEMS, MAX_GEM, RINGS};
use crate::scrolls::constants::{MAX_SYLLABLE, SCROLLS, SYLLABLES};
use crate::weapons::constants::WEAPONS;
use crate::zap::constants::{ALL_WAND_MATERIALS, MAX_WAND_MATERIAL, WANDS};

impl NoteTables {
	pub fn assign_dynamic_titles(&mut self) {
		self.shuffle_potions_colors();
		self.make_scroll_titles();
		self.assign_wand_materials();
		self.assign_ring_gems()
	}
	fn shuffle_potions_colors(&mut self) {
		for _ in 0..=32 {
			let j = get_rand(0, POTIONS - 1);
			let k = get_rand(0, POTIONS - 1);
			self.potions.swap(j, k);
		}
	}
	fn make_scroll_titles(&mut self) {
		for i in 0..SCROLLS {
			let syllables = Self::random_scroll_syllables()
				.into_iter()
				.map(|it| it.trim().to_string())
				.collect::<Vec<_>>()
				;
			let joined = format!("'{}' ", syllables.join(" "));
			self.scrolls[i].title = Title::SyllableString(joined);
		}
	}
	fn random_scroll_syllables() -> Vec<String> {
		(0..get_rand(2, 5))
			.map(|_| random_syllable().to_string())
			.collect::<Vec<_>>()
	}
	pub fn assign_ring_gems(&mut self) {
		let mut unused_gem = (0..MAX_GEM).collect::<Vec<_>>();
		unused_gem.shuffle(&mut thread_rng());
		for i in 0..RINGS {
			if let Some(j) = unused_gem.pop() {
				self.rings[i].title = Title::RingGem(ALL_RING_GEMS[j]);
			}
		}
	}

	pub fn assign_wand_materials(&mut self) {
		let mut unused_material = (0..MAX_WAND_MATERIAL).collect::<Vec<_>>();
		unused_material.shuffle(&mut thread_rng());
		for i in 0..WANDS {
			if let Some(j) = unused_material.pop() {
				let wand_material = ALL_WAND_MATERIALS[j];
				let note = &mut self.wands[i];
				note.is_wood = wand_material.is_wood();
				note.title = Title::WandMaterial(wand_material);
			}
		}
	}
	pub fn identify_all(&mut self) {
		for i in 0..POTIONS {
			self.potions[i].status = Identified;
		}
		for i in 0..SCROLLS {
			self.scrolls[i].status = Identified;
		}
		for i in 0..WEAPONS {
			self.weapons[i].status = Identified;
		}
		for i in 0..ARMORS {
			self.armors[i].status = Identified;
		}
		for i in 0..WANDS {
			self.wands[i].status = Identified;
		}
		for i in 0..RINGS {
			self.rings[i].status = Identified;
		}
	}

	pub fn identify(&mut self, what: ObjectWhat, kind: usize) {
		let id = self.note_mut(what, kind);
		id.status = Identified;
	}
	pub fn identify_if_un_called(&mut self, what: ObjectWhat, kind: usize) {
		let id = self.note_mut(what, kind);
		if id.status != Called {
			id.status = Identified;
		}
	}
	pub fn title(&self, what: ObjectWhat, kind: usize) -> Title {
		self.note(what, kind).title.clone()
	}
	pub fn status(&self, what: ObjectWhat, kind: usize) -> NoteStatus {
		self.note(what, kind).status
	}

	pub fn note(&self, what: ObjectWhat, kind: usize) -> &Note {
		let table = self.table(what);
		let id = &table[kind];
		id
	}
	pub fn note_mut(&mut self, what: ObjectWhat, kind: usize) -> &mut Note {
		let table = self.table_mut(what);
		let id = &mut table[kind];
		id
	}
	fn table(&self, what: ObjectWhat) -> &[Note] {
		match what {
			Scroll => &self.scrolls[..],
			Potion => &self.potions[..],
			Wand => &self.wands[..],
			Ring => &self.rings[..],
			Weapon => &self.weapons[..],
			Armor => &self.armors[..],
			_ => panic!("no id table"),
		}
	}
	pub(crate) fn table_mut(&mut self, what: ObjectWhat) -> &mut [Note] {
		match what {
			Scroll => &mut self.scrolls[..],
			Potion => &mut self.potions[..],
			Wand => &mut self.wands[..],
			Ring => &mut self.rings[..],
			Weapon => &mut self.weapons[..],
			Armor => &mut self.armors[..],
			_ => panic!("no id table"),
		}
	}
}

#[derive(Clone, Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct NoteTables {
	pub potions: [Note; POTIONS],
	pub scrolls: [Note; SCROLLS],
	pub weapons: [Note; WEAPONS],
	pub armors: [Note; ARMORS],
	pub wands: [Note; WANDS],
	pub rings: [Note; RINGS],
}

impl Default for NoteTables {
	fn default() -> Self {
		Self::new()
	}
}

mod constants {
	use crate::armors::constants::ARMORS;
	use crate::armors::ArmorKind;
	use crate::objects::note_tables::NoteTables;
	use crate::objects::Note;
	use crate::potions::colors::PotionColor;
	use crate::potions::kind::POTIONS;
	use crate::ring::constants::RINGS;
	use crate::ring::ring_kind::RingKind;
	use crate::scrolls::constants::SCROLLS;
	use crate::scrolls::ScrollKind;
	use crate::weapons::constants::WEAPONS;
	use crate::weapons::kind::WeaponKind;
	use crate::zap::constants::WANDS;
	use crate::zap::wand_kind::WandKind;

	impl NoteTables {
		pub const fn new() -> Self {
			NoteTables {
				potions: ID_POTIONS,
				scrolls: ID_SCROLLS,
				weapons: ID_WEAPONS,
				armors: ID_ARMORS,
				wands: ID_WANDS,
				rings: ID_RINGS,
			}
		}
	}

	pub const ID_POTIONS: [Note; POTIONS] = [
		PotionColor::Blue.to_id(),
		PotionColor::Red.to_id(),
		PotionColor::Green.to_id(),
		PotionColor::Grey.to_id(),
		PotionColor::Brown.to_id(),
		PotionColor::Clear.to_id(),
		PotionColor::Pink.to_id(),
		PotionColor::White.to_id(),
		PotionColor::Purple.to_id(),
		PotionColor::Black.to_id(),
		PotionColor::Yellow.to_id(),
		PotionColor::Plaid.to_id(),
		PotionColor::Burgundy.to_id(),
		PotionColor::Beige.to_id(),
	];

	pub const ID_SCROLLS: [Note; SCROLLS] = [
		ScrollKind::ProtectArmor.to_id(),
		ScrollKind::HoldMonster.to_id(),
		ScrollKind::EnchWeapon.to_id(),
		ScrollKind::EnchArmor.to_id(),
		ScrollKind::Identify.to_id(),
		ScrollKind::Teleport.to_id(),
		ScrollKind::Sleep.to_id(),
		ScrollKind::ScareMonster.to_id(),
		ScrollKind::RemoveCurse.to_id(),
		ScrollKind::CreateMonster.to_id(),
		ScrollKind::AggravateMonster.to_id(),
		ScrollKind::MagicMapping.to_id(),
	];
	pub const ID_WEAPONS: [Note; WEAPONS] = [
		WeaponKind::Bow.to_id(),
		WeaponKind::Dart.to_id(),
		WeaponKind::Arrow.to_id(),
		WeaponKind::Dagger.to_id(),
		WeaponKind::Shuriken.to_id(),
		WeaponKind::Mace.to_id(),
		WeaponKind::LongSword.to_id(),
		WeaponKind::TwoHandedSword.to_id(),
	];
	pub const ID_ARMORS: [Note; ARMORS] = [
		ArmorKind::Leather.to_id(),
		ArmorKind::Ringmail.to_id(),
		ArmorKind::Scale.to_id(),
		ArmorKind::Chain.to_id(),
		ArmorKind::Banded.to_id(),
		ArmorKind::Splint.to_id(),
		ArmorKind::Plate.to_id(),
	];
	pub const ID_WANDS: [Note; WANDS] = [
		WandKind::TeleAway.to_id(),
		WandKind::SlowMonster.to_id(),
		WandKind::ConfuseMonster.to_id(),
		WandKind::Invisibility.to_id(),
		WandKind::Polymorph.to_id(),
		WandKind::HasteMonster.to_id(),
		WandKind::PutToSleep.to_id(),
		WandKind::MagicMissile.to_id(),
		WandKind::Cancellation.to_id(),
		WandKind::DoNothing.to_id(),
	];
	pub const ID_RINGS: [Note; RINGS] = [
		RingKind::Stealth.to_id(),
		RingKind::RTeleport.to_id(),
		RingKind::Regeneration.to_id(),
		RingKind::SlowDigest.to_id(),
		RingKind::AddStrength.to_id(),
		RingKind::SustainStrength.to_id(),
		RingKind::Dexterity.to_id(),
		RingKind::Adornment.to_id(),
		RingKind::RSeeInvisible.to_id(),
		RingKind::MaintainArmor.to_id(),
		RingKind::Searching.to_id(),
	];
}

fn random_syllable() -> &'static str {
	SYLLABLES[get_rand(1, MAX_SYLLABLE - 1)]
}

