pub const TELE_AWAY: u16 = 0;
pub const SLOW_MONSTER: u16 = 1;
pub const CONFUSE_MONSTER: u16 = 2;
pub const INVISIBILITY: u16 = 3;
pub const POLYMORPH: u16 = 4;
pub const HASTE_MONSTER: u16 = 5;
pub const PUT_TO_SLEEP: u16 = 6;
pub const MAGIC_MISSILE: u16 = 7;
pub const CANCELLATION: u16 = 8;
pub const DO_NOTHING: u16 = 9;
pub const WANDS: usize = 10;

pub enum WandKind {
	TeleAway,
	SlowMonster,
	ConfuseMonster,
	Invisibility,
	Polymorph,
	HasteMonster,
	PutToSleep,
	MagicMissile,
	Cancellation,
	DoNothing,
}

impl WandKind {
	pub fn from_code(code: u16) -> Self {
		match code {
			TELE_AWAY => WandKind::TeleAway,
			SLOW_MONSTER => WandKind::SlowMonster,
			CONFUSE_MONSTER => WandKind::ConfuseMonster,
			INVISIBILITY => WandKind::Invisibility,
			POLYMORPH => WandKind::Polymorph,
			HASTE_MONSTER => WandKind::HasteMonster,
			PUT_TO_SLEEP => WandKind::PutToSleep,
			MAGIC_MISSILE => WandKind::MagicMissile,
			CANCELLATION => WandKind::Cancellation,
			DO_NOTHING => WandKind::DoNothing,
			_ => panic!("invalid code")
		}
	}
}