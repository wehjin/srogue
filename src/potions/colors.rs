use serde::{Deserialize, Serialize};
use crate::objects::{Note, NoteStatus, Title};
use crate::potions::kind::POTIONS;

#[derive(Copy, Clone, Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub enum PotionColor {
	Blue,
	Red,
	Green,
	Grey,
	Brown,
	Clear,
	Pink,
	White,
	Purple,
	Black,
	Yellow,
	Plaid,
	Burgundy,
	Beige,
}

impl PotionColor {
	pub const fn to_id(self) -> Note {
		Note {
			title: Title::PotionColor(self),
			status: NoteStatus::Unidentified,
			is_wood: false,
		}
	}
	pub const fn name(&self) -> &'static str {
		match self {
			PotionColor::Blue => "blue ",
			PotionColor::Red => "red ",
			PotionColor::Green => "green ",
			PotionColor::Grey => "grey ",
			PotionColor::Brown => "brown ",
			PotionColor::Clear => "clear ",
			PotionColor::Pink => "pink ",
			PotionColor::White => "white ",
			PotionColor::Purple => "purple ",
			PotionColor::Black => "black ",
			PotionColor::Yellow => "yellow ",
			PotionColor::Plaid => "plaid ",
			PotionColor::Burgundy => "burgundy ",
			PotionColor::Beige => "beige ",
		}
	}
}

pub const ALL_POTION_COLORS: [PotionColor; POTIONS] = [
	PotionColor::Blue,
	PotionColor::Red,
	PotionColor::Green,
	PotionColor::Grey,
	PotionColor::Brown,
	PotionColor::Clear,
	PotionColor::Pink,
	PotionColor::White,
	PotionColor::Purple,
	PotionColor::Black,
	PotionColor::Yellow,
	PotionColor::Plaid,
	PotionColor::Burgundy,
	PotionColor::Beige,
];