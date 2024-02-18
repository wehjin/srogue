use crate::ring::ring_gem::RingGem;

pub const RINGS: usize = 11;

pub const MAX_GEM: usize = 14;

pub(crate) const ALL_RING_GEMS: [RingGem; MAX_GEM] = [
	RingGem::DIAMOND,
	RingGem::STIBOTANTALITE,
	RingGem::LapiLazuli,
	RingGem::RUBY,
	RingGem::EMERALD,
	RingGem::SAPPHIRE,
	RingGem::AMETHYST,
	RingGem::QUARTZ,
	RingGem::TigerEye,
	RingGem::OPAL,
	RingGem::AGATE,
	RingGem::TURQUOISE,
	RingGem::PEARL,
	RingGem::GARNET,
];

