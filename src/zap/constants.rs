use crate::zap::wand_materials::WandMaterial;

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
pub const MAX_WAND_MATERIAL: usize = 30;
pub const MAX_METAL: usize = 14;
pub const ALL_WAND_MATERIALS: [WandMaterial; MAX_WAND_MATERIAL] = [
	WandMaterial::STEEL,
	WandMaterial::BRONZE,
	WandMaterial::GOLD,
	WandMaterial::SILVER,
	WandMaterial::COPPER,
	WandMaterial::NICKEL,
	WandMaterial::COBALT,
	WandMaterial::TIN,
	WandMaterial::IRON,
	WandMaterial::MAGNESIUM,
	WandMaterial::CHROME,
	WandMaterial::CARBON,
	WandMaterial::PLATINUM,
	WandMaterial::SILICON,
	WandMaterial::TITANIUM,
	WandMaterial::TEAK,
	WandMaterial::OAK,
	WandMaterial::CHERRY,
	WandMaterial::BIRCH,
	WandMaterial::PINE,
	WandMaterial::CEDAR,
	WandMaterial::REDWOOD,
	WandMaterial::BALSA,
	WandMaterial::IVORY,
	WandMaterial::WALNUT,
	WandMaterial::MAPLE,
	WandMaterial::MAHOGANY,
	WandMaterial::ELM,
	WandMaterial::PALM,
	WandMaterial::WOODEN,
];
