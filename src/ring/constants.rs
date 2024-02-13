use crate::ring::ring_gem::RingGem;

pub const RINGS: usize = 11;

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
pub const MAX_GEM: usize = 14;

pub(crate) const GEMS: [&'static str; MAX_GEM] = [
	"diamond ",
	"stibotantalite ",
	"lapi-lazuli ",
	"ruby ",
	"emerald ",
	"sapphire ",
	"amethyst ",
	"quartz ",
	"tiger-eye ",
	"opal ",
	"agate ",
	"turquoise ",
	"pearl ",
	"garnet ",
];
pub(crate) const ALL_RING_GEMS: [RingGem; MAX_GEM] = [
	RingGem::DIAMOND,
	RingGem::STIBOTANTALITE,
	RingGem::LAPI_LAZULI,
	RingGem::RUBY,
	RingGem::EMERALD,
	RingGem::SAPPHIRE,
	RingGem::AMETHYST,
	RingGem::QUARTZ,
	RingGem::TIGER_EYE,
	RingGem::OPAL,
	RingGem::AGATE,
	RingGem::TURQUOISE,
	RingGem::PEARL,
	RingGem::GARNET,
];

