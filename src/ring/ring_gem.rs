use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum RingGem {
	DIAMOND,
	STIBOTANTALITE,
	LapiLazuli,
	RUBY,
	EMERALD,
	SAPPHIRE,
	AMETHYST,
	QUARTZ,
	TigerEye,
	OPAL,
	AGATE,
	TURQUOISE,
	PEARL,
	GARNET,
}

impl RingGem {
	pub fn name(&self) -> &'static str {
		match self {
			RingGem::DIAMOND => "diamond ",
			RingGem::STIBOTANTALITE => "stibotantalite ",
			RingGem::LapiLazuli => "lapi-lazuli ",
			RingGem::RUBY => "ruby ",
			RingGem::EMERALD => "emerald ",
			RingGem::SAPPHIRE => "sapphire ",
			RingGem::AMETHYST => "amethyst ",
			RingGem::QUARTZ => "quartz ",
			RingGem::TigerEye => "tiger-eye ",
			RingGem::OPAL => "opal ",
			RingGem::AGATE => "agate ",
			RingGem::TURQUOISE => "turquoise ",
			RingGem::PEARL => "pearl ",
			RingGem::GARNET => "garnet ",
		}
	}
}
