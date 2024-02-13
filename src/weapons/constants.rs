use crate::objects::{Note, NoteStatus, Title};
use crate::weapons::kind::WeaponKind;
use crate::weapons::kind::WeaponKind::{Arrow, Bow, Dagger, Dart, LongSword, Mace, Shuriken, TwoHandedSword};

pub const BOW: u16 = 0;
pub const DART: u16 = 1;
pub const ARROW: u16 = 2;
pub const DAGGER: u16 = 3;
pub const SHURIKEN: u16 = 4;
pub const MACE: u16 = 5;
pub const LONG_SWORD: u16 = 6;
pub const TWO_HANDED_SWORD: u16 = 7;
pub const WEAPONS: usize = 8;
pub const WEAPON_NAMES: [&'static str; WEAPONS] = [
	"short bow ", "darts ", "arrows ", "daggers ", "shurikens ", "mace ", "long sword ", "two-handed sword "
];

impl WeaponKind {
	pub const ALL_KINDS: [WeaponKind; WEAPONS] = [
		Bow, Dart, Arrow, Dagger, Shuriken, Mace, LongSword, TwoHandedSword,
	];

	pub const fn to_id(self) -> Note {
		Note {
			title: Title::WeaponName(self),
			status: NoteStatus::Unidentified,
			is_wood: false,
		}
	}
}
