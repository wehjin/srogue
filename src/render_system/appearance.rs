use crate::init::GameState;
use crate::level::materials::{CellMaterial, FloorFixture, TunnelFixture, Visibility};
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::DungeonSpot;
use crate::render_system::appearance::player::player_sees_invisible;
use crate::render_system::appearance::spot::{monster_at_spot, object_at_spot, spot_is_line_of_sight};
use crate::render_system::{EMPTY_CHAR, STAIRS_CHAR, TRAP_CHAR, TUNNEL_CHAR};

pub(crate) enum SpotAppearance {
	None,
	FogBlack,
	Disguise(char),
	Monster(char),
	Object(ObjectWhat),
	Material(char),
}

impl SpotAppearance {
	pub fn to_char(&self) -> char {
		let char = match self {
			SpotAppearance::None
			| SpotAppearance::FogBlack => EMPTY_CHAR,
			SpotAppearance::Disguise(char) => *char,
			SpotAppearance::Monster(char) => *char,
			SpotAppearance::Object(what) => what.to_char(),
			SpotAppearance::Material(char) => *char,
		};
		char
	}
}

pub(crate) fn appearance_for_spot(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	if spot.is_out_of_bounds() {
		SpotAppearance::None
	} else if game.player.is_blind() {
		SpotAppearance::FogBlack
	} else if spot_is_line_of_sight(spot, game) {
		when_line_of_sight(spot, game)
	} else {
		when_out_of_sight(spot, game)
	}
}

fn when_line_of_sight(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	match monster_at_spot(spot, game) {
		Some(monster) if game.level.detect_monster => SpotAppearance::Monster(monster.as_char()),
		Some(monster) if !monster.is_invisible() || player_sees_invisible(game) => {
			if monster.imitates() {
				SpotAppearance::Disguise(monster.disguise.char())
			} else {
				SpotAppearance::Monster(monster.as_char())
			}
		}
		Some(_) | None => when_line_of_sight_no_mon(spot, game),
	}
}

fn when_line_of_sight_no_mon(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	match object_at_spot(spot, game) {
		Some(object) => SpotAppearance::Object(object.what_is),
		None => when_line_of_sight_no_obj_mon(spot, game),
	}
}

fn when_line_of_sight_no_obj_mon(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	let material = game.cell_at(spot).material();
	match material {
		CellMaterial::None => SpotAppearance::FogBlack,
		_ => SpotAppearance::Material(material.to_char()),
	}
}

fn when_out_of_sight(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	let cell = game.cell_at(spot);
	if cell.is_visited() {
		// Out of sight, visited
		match monster_at_spot(spot, game) {
			Some(monster) if game.level.detect_monster => SpotAppearance::Monster(monster.as_char()),
			Some(monster) if monster.imitates() => SpotAppearance::Disguise(monster.disguise.char()),
			Some(_) | None => when_out_of_sight_visited_no_mon(spot, game),
		}
	} else {
		// Out of sight, unvisited
		match monster_at_spot(spot, game) {
			Some(monster) if game.level.detect_monster => SpotAppearance::Monster(monster.as_char()),
			Some(_) | None => SpotAppearance::FogBlack,
		}
	}
}

fn when_out_of_sight_visited_no_mon(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	match object_at_spot(spot, game) {
		Some(object) => SpotAppearance::Object(object.what_is),
		None => when_out_of_sight_visited_no_obj_mon(spot, game)
	}
}

fn when_out_of_sight_visited_no_obj_mon(spot: DungeonSpot, game: &GameState) -> SpotAppearance {
	let material = game.cell_at(spot).material();
	match material {
		CellMaterial::HorizontalWall
		| CellMaterial::VerticalWall
		| CellMaterial::Door(_, _) => SpotAppearance::Material(material.to_char()),
		CellMaterial::None => SpotAppearance::None,
		CellMaterial::Floor(FloorFixture::Stairs) => SpotAppearance::Material(STAIRS_CHAR),
		CellMaterial::Floor(FloorFixture::Trap(Visibility::Visible)) => SpotAppearance::Material(TRAP_CHAR),
		CellMaterial::Floor(_) => SpotAppearance::FogBlack,
		CellMaterial::Tunnel(Visibility::Hidden, _) => SpotAppearance::FogBlack,
		CellMaterial::Tunnel(Visibility::Visible, TunnelFixture::Stairs) => SpotAppearance::Material(STAIRS_CHAR),
		CellMaterial::Tunnel(_, _) => SpotAppearance::Material(TUNNEL_CHAR),
	}
}

mod player {
	use crate::init::GameState;

	pub fn player_sees_invisible(game: &GameState) -> bool {
		game.level.see_invisible || game.player.ring_effects.has_see_invisible()
	}
}

mod spot {
	use crate::init::GameState;
	use crate::monster::Monster;
	use crate::objects::Object;
	use crate::prelude::DungeonSpot;

	pub(super) fn object_at_spot(spot: DungeonSpot, game: &GameState) -> Option<&Object> {
		game.ground.find_object_at(spot.row, spot.col)
	}

	pub(super) fn monster_at_spot(spot: DungeonSpot, game: &GameState) -> Option<&Monster> {
		game.mash.monster_at_spot(spot.row, spot.col)
	}

	pub(super) fn spot_is_line_of_sight(spot: DungeonSpot, game: &GameState) -> bool {
		player_in_same_cavern_as_spot(spot, game)
			|| (game.player.is_near_spot(spot) && !game.cell_at(spot).is_any_wall())
	}

	pub(super) fn player_in_same_cavern_as_spot(spot: DungeonSpot, game: &GameState) -> bool {
		let player_room = game.player.cur_room;
		player_room.is_cavern() && game.level.room_at_spot(spot) == player_room
	}
}