use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RingGem {
	DIAMOND,
	STIBOTANTALITE,
	LAPI_LAZULI,
	RUBY,
	EMERALD,
	SAPPHIRE,
	AMETHYST,
	QUARTZ,
	TIGER_EYE,
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
			RingGem::LAPI_LAZULI => "lapi-lazuli ",
			RingGem::RUBY => "ruby ",
			RingGem::EMERALD => "emerald ",
			RingGem::SAPPHIRE => "sapphire ",
			RingGem::AMETHYST => "amethyst ",
			RingGem::QUARTZ => "quartz ",
			RingGem::TIGER_EYE => "tiger-eye ",
			RingGem::OPAL => "opal ",
			RingGem::AGATE => "agate ",
			RingGem::TURQUOISE => "turquoise ",
			RingGem::PEARL => "pearl ",
			RingGem::GARNET => "garnet ",
		}
	}
}
