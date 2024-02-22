use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::random::rand_percent;
use crate::trap;

pub struct Search;

impl PlayerAction for Search {
	fn update(game: &mut GameState) {
		search(SearchKind::Manual, game);
	}
}

const SEARCH_OFFSETS: [(i64, i64); 8] = [
	(-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1)
];

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum SearchKind {
	Auto { n: usize },
	Manual,
}

pub(crate) fn search(kind: SearchKind, game: &mut GameState) {
	let mut found_spots = Vec::new();
	let center_spot = game.player.to_spot();
	for offset in SEARCH_OFFSETS {
		let search_spot = center_spot + offset;
		if search_spot.is_out_of_bounds() {
			continue;
		}
		if game.cell_at(search_spot).is_any_hidden() {
			found_spots.push(search_spot)
		}
	}
	let mut hidden_spots = found_spots.clone();
	let mut lucky_spots = 0usize;
	let n = if let SearchKind::Auto { n } = kind { n } else { 1 };
	for _ in 0..n {
		let mut unlucky_spots = Vec::new();
		for spot in hidden_spots {
			if rand_percent(17 + game.player.buffed_exp() as usize) {
				lucky_spots += 1;
				game.cell_at_mut(spot).set_visible();
				game.render_spot(spot);
				if game.cell_at(spot).is_any_trap() {
					let msg = trap::trap_at(spot.row as usize, spot.col as usize, &game.level).name();
					game.player.interrupt_and_slurp();
					game.dialog.message(msg, 1);
				}
			} else {
				unlucky_spots.push(spot);
			}
			if game.player.interrupted
				|| lucky_spots + unlucky_spots.len() == found_spots.len() {
				return;
			}
		}
		if let SearchKind::Manual = kind {
			game.player.reg_search_count += 1;
			if (game.player.reg_search_count & 1) == 0 {
				game.yield_turn_to_monsters();
			}
			return;
		}
		hidden_spots = unlucky_spots;
	}
}
