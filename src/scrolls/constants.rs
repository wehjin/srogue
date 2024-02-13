use crate::objects::{Note, NoteStatus, Title};
use crate::scrolls::ScrollKind;
use crate::scrolls::ScrollKind::{AggravateMonster, CreateMonster, EnchArmor, EnchWeapon, HoldMonster, Identify, MagicMapping, ProtectArmor, RemoveCurse, ScareMonster, Sleep, Teleport};

pub const SCROLLS: usize = 12;

impl ScrollKind {
	pub const ALL_SCROLLS: [ScrollKind; SCROLLS] = [
		ProtectArmor, HoldMonster, EnchWeapon, EnchArmor, Identify, Teleport,
		Sleep, ScareMonster, RemoveCurse, CreateMonster, AggravateMonster, MagicMapping
	];
	pub const REAL_NAME: [&'static str; SCROLLS] = [
		"of protect armor ", "of hold monster ", "of enchant weapon ", "of enchant armor ", "of identify ", "of teleportation ",
		"of sleep ", "of scare monster ", "of remove curse ", "of create monster ", "of aggravate monster ", "of magic mapping "
	];
}

pub const MAX_SYLLABLE: usize = 40;

impl ScrollKind {
	pub const fn to_id(self) -> Note {
		Note {
			title: Title::None,
			status: NoteStatus::Unidentified,
		}
	}
}

pub const SYLLABLES: [&'static str; MAX_SYLLABLE] = [
	"blech ",
	"foo ",
	"barf ",
	"rech ",
	"bar ",
	"blech ",
	"quo ",
	"bloto ",
	"woh ",
	"caca ",
	"blorp ",
	"erp ",
	"festr ",
	"rot ",
	"slie ",
	"snorf ",
	"iky ",
	"yuky ",
	"ooze ",
	"ah ",
	"bahl ",
	"zep ",
	"druhl ",
	"flem ",
	"behil ",
	"arek ",
	"mep ",
	"zihr ",
	"grit ",
	"kona ",
	"kini ",
	"ichi ",
	"niah ",
	"ogr ",
	"ooh ",
	"ighr ",
	"coph ",
	"swerr ",
	"mihln ",
	"poxi ",
];

