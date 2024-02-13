use crate::armors::ArmorKind;
use crate::armors::ArmorKind::{Banded, Chain, Leather, Plate, Ringmail, Scale, Splint};
use crate::objects::{Note, NoteStatus, Title};

pub const LEATHER: u16 = 0;
pub const RINGMAIL: u16 = 1;
pub const SCALE: u16 = 2;
pub const CHAIN: u16 = 3;
pub const BANDED: u16 = 4;
pub const SPLINT: u16 = 5;
pub const PLATE: u16 = 6;
pub const ARMORS: usize = 7;
pub const ARMOR_NAMES: [&'static str; ARMORS] = [
	"leather armor ", "ring mail ", "scale mail ", "chain mail ", "banded mail ", "splint mail ", "plate mail ",
];

impl ArmorKind {
	pub const ALL_KINDS: [ArmorKind; ARMORS] = [
		Leather, Ringmail, Scale, Chain, Banded, Splint, Plate,
	];
	pub const fn to_id(self) -> Note {
		Note {
			title: Title::ArmorName(self),
			status: NoteStatus::Unidentified,
		}
	}
}
