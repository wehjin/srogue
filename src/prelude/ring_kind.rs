pub enum RingKind {
	Stealth,
	RTeleport,
	Regeneration,
	SlowDigest,
	AddStrength,
	SustainStrength,
	Dexterity,
	Adornment,
	RSeeInvisible,
	MaintainArmor,
	Searching,
}

impl RingKind {
	pub fn from_code(code: u16) -> Self {
		match code {
			STEALTH => Self::Stealth,
			R_TELEPORT => Self::RTeleport,
			REGENERATION => Self::Regeneration,
			SLOW_DIGEST => Self::SlowDigest,
			ADD_STRENGTH => Self::AddStrength,
			SUSTAIN_STRENGTH => Self::SustainStrength,
			DEXTERITY => Self::Dexterity,
			ADORNMENT => Self::Adornment,
			R_SEE_INVISIBLE => Self::RSeeInvisible,
			MAINTAIN_ARMOR => Self::MaintainArmor,
			SEARCHING => Self::Searching,
			_ => unreachable!("invalid ring code"),
		}
	}
}

pub const STEALTH: u16 = 0;
pub const R_TELEPORT: u16 = 1;
pub const REGENERATION: u16 = 2;
pub const SLOW_DIGEST: u16 = 3;
pub const ADD_STRENGTH: u16 = 4;
pub const SUSTAIN_STRENGTH: u16 = 5;
pub const DEXTERITY: u16 = 6;
pub const ADORNMENT: u16 = 7;
pub const R_SEE_INVISIBLE: u16 = 8;
pub const MAINTAIN_ARMOR: u16 = 9;
pub const SEARCHING: u16 = 10;
pub const RINGS: usize = 11;
