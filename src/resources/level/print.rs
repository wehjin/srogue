use crate::level::materials::Visibility;
use crate::monster::MonsterKind;
use crate::prelude::object_what::ObjectWhat;
use crate::render_system::{DOOR_CHAR, EMPTY_CHAR, PLAYER_CHAR, STAIRS_CHAR, TRAP_CHAR, TUNNEL_CHAR};
use crate::resources::level::map::feature::Feature;
use crate::resources::level::size::LevelSpot;
use crate::resources::level::DungeonLevel;

impl DungeonLevel {
	pub fn print(&self, reveal_hidden: bool) {
		let bounds = self.map.bounds();
		for row in bounds.rows() {
			let mut line = String::new();
			for col in bounds.cols() {
				let spot = LevelSpot::from_i64(row, col);
				let spot_view = SpotView::new(spot, &self);
				line.push(spot_view.to_char(reveal_hidden));
			}
			println!("{}", line);
		}
	}
}

pub enum SpotView {
	Unlit,
	Rogue,
	Monster(MonsterKind),
	Object(ObjectWhat),
	Feature(Feature),
}

impl SpotView {
	pub fn new(spot: LevelSpot, level: &DungeonLevel) -> Self {
		match level.lighting_enabled {
			true => Self::with_lighting(spot, level),
			false => Self::lit(spot, level),
		}
	}

	fn with_lighting(spot: LevelSpot, level: &DungeonLevel) -> SpotView {
		if level.rogue_at_spot(spot) {
			SpotView::Rogue
		} else {
			if let Some(room) = level.room_at_spot(spot) {
				if level.rogue_spot.is_in_room(room) {
					Self::lit(spot, level)
				} else {
					SpotView::Unlit
				}
			} else {
				SpotView::Unlit
			}
		}
	}

	fn lit(spot: LevelSpot, level: &DungeonLevel) -> SpotView {
		if level.rogue_at_spot(spot) {
			Self::Rogue
		} else if let Some(monster) = level.try_monster(spot) {
			Self::Monster(monster.kind)
		} else if let Some(object) = level.try_object(spot) {
			Self::Object(object.what_is)
		} else {
			Self::Feature(level.map.feature_at_spot(spot))
		}
	}
	fn to_char(&self, reveal_hidden: bool) -> char {
		match self {
			SpotView::Unlit => EMPTY_CHAR,
			SpotView::Rogue => PLAYER_CHAR,
			SpotView::Monster(kind) => kind.screen_char(),
			SpotView::Object(what) => what.to_char(),
			SpotView::Feature(feature) => match feature {
				Feature::None => ' ',
				Feature::HorizWall => '-',
				Feature::VertWall => '|',
				Feature::Floor => '.',
				Feature::Tunnel => TUNNEL_CHAR,
				Feature::ConcealedTunnel => if reveal_hidden { '_' } else { TUNNEL_CHAR },
				Feature::Door => DOOR_CHAR,
				Feature::ConcealedDoor => if reveal_hidden { '_' } else { DOOR_CHAR },
				Feature::Stairs => STAIRS_CHAR,
				Feature::Trap(_, visibility) => {
					match visibility {
						Visibility::Visible => TRAP_CHAR,
						Visibility::Hidden => if reveal_hidden { 'v' } else { TRAP_CHAR },
					}
				}
			},
		}
	}
}