use crate::init::{Dungeon, GameState};
use crate::level::put_player_legacy;
use crate::monster::{mv_mons, show_monsters};
use crate::motion::{reg_move, YOU_CAN_MOVE_AGAIN};
use crate::objects::ObjectId;
use crate::pack::{take_from_pack, unwear, unwield};
use crate::player::{Player, RoomMark};
use crate::potions::colors::ALL_POTION_COLORS;
use crate::potions::kind::POTIONS;
use crate::random::get_rand;
use crate::render_system;
use crate::render_system::backend;
use crate::render_system::hallucinate::show_hallucination;
use crate::resources::avatar::Avatar;
use crate::ring::un_put_hand;
use crate::room::{visit_room, visit_spot_area};

pub fn vanish(obj_id: ObjectId, do_regular_move: bool, game: &mut GameState) {
	/* vanish() does NOT handle a quiver of weapons with more than one
	   arrow (or whatever) in the quiver.  It will only decrement the count.
	*/
	let obj = game.player.object_mut(obj_id).expect("obj in player");
	if obj.quantity > 1 {
		obj.quantity -= 1;
	} else {
		if obj.is_being_wielded() {
			unwield(&mut game.player);
		} else if obj.is_being_worn() {
			unwear(&mut game.player);
		} else if let Some(hand) = game.player.ring_hand(obj_id) {
			un_put_hand(hand, game);
		}
		take_from_pack(obj_id, &mut game.player.rogue.pack);
	}
	if do_regular_move {
		reg_move(game);
	}
}

pub fn tele(game: &mut GameState) {
	let exit_spot = game.player.to_spot();
	render_system::show_darkened_room_after_player_exit(exit_spot, game);
	let avoid_room = game.player.cur_room;
	put_player_legacy(avoid_room, game);
	let health = game.as_health_mut();
	health.being_held = false;
	health.bear_trap = 0;
}


pub fn unhallucinate(game: &mut GameState) {
	game.player.health.halluc.clear();
	relight(game);
	game.player.interrupt_and_slurp();
	game.diary.add_entry("everything looks SO boring now");
}

pub fn unblind(game: &mut GameState) {
	game.player.health.blind.clear();
	game.player.interrupt_and_slurp();
	game.diary.add_entry("the veil of darkness lifts");
	relight(game);
	if game.player.health.halluc.is_active() {
		show_hallucination(game);
	}
	if game.level.detect_monster {
		show_monsters(game);
	}
}

pub fn relight(game: &mut GameState) {
	match game.player.cur_room {
		RoomMark::None => {}
		RoomMark::Passage => {
			visit_spot_area(game.player.rogue.row, game.player.rogue.col, game);
		}
		RoomMark::Cavern(rn) => {
			visit_room(rn, game);
		}
	}
	game.render_spot(game.player.to_spot());
}

pub fn take_a_nap(game: &mut GameState) {
	let mut i = get_rand(2, 5);
	backend::await_frame();
	while i > 0 {
		i -= 1;
		mv_mons(game);
		backend::await_frame()
	}
	game.diary.add_entry(YOU_CAN_MOVE_AGAIN);
}

pub fn get_ench_color(player: &Player) -> &'static str {
	if player.health.halluc.is_active() {
		ALL_POTION_COLORS[get_rand(0, POTIONS - 1)].name()
	} else {
		"blue "
	}
}

pub fn confuse(game: &mut impl Dungeon) {
	let amount = get_rand(12, 22);
	game.as_health_mut().confused.extend(amount);
}

pub fn unconfuse(game: &mut GameState) {
	game.player.health.confused.clear();
	let feeling = if game.player.health.halluc.is_active() { "trippy" } else { "confused" };
	let msg = format!("you feel less {} now", feeling);
	game.player.interrupt_and_slurp();
	game.diary.add_entry(&msg);
}
