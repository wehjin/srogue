use crate::components::hunger::HungerLevel;
use crate::init::GameState;
use crate::level::{add_exp, put_player};
use crate::message::{CANCEL, print_stats};
use crate::monster::{aggravate, create_monster, mv_mons, show_monsters};
use crate::motion::{reg_move, YOU_CAN_MOVE_AGAIN};
use crate::objects::NoteStatus::Identified;
use crate::objects::ObjectId;
use crate::pack::{pack_letter, take_from_pack, unwear, unwield};
use crate::player::{Player, RoomMark};
use crate::potions::colors::ALL_POTION_COLORS;
use crate::potions::kind::POTIONS;
use crate::potions::quaff::quaff_potion;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Foods, Potions, Scrolls};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_HP, STAT_HUNGER, STAT_STRENGTH};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::render_system;
use crate::ring::un_put_hand;
use crate::room::{draw_magic_map, light_up_room};
use crate::scrolls::ScrollKind;
use crate::trap::is_off_screen;

pub const STRANGE_FEELING: &'static str = "you have a strange feeling for a moment, then it passes";

pub fn quaff(game: &mut GameState) {
	let ch = pack_letter("quaff what?", Potions, game);
	if ch == CANCEL {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			match game.player.expect_object(obj_id).potion_kind() {
				None => {
					game.dialog.message("you can't drink that", 0);
					return;
				}
				Some(potion_kind) => {
					quaff_potion(potion_kind, game);
					print_stats(STAT_STRENGTH | STAT_HP, &mut game.player);
					game.player.notes.identify_if_un_called(Potion, potion_kind.to_index());
					vanish(obj_id, true, game);
				}
			}
		}
	}
}

pub fn read_scroll(game: &mut GameState) {
	if game.player.blind.is_active() {
		game.dialog.message("You can't see to read the scroll.", 0);
		return;
	}

	let ch = pack_letter("read what?", Scrolls, game);
	if ch == CANCEL {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			match game.player.expect_object(obj_id).scroll_kind() {
				None => {
					game.dialog.message("you can't read that", 0);
					return;
				}
				Some(scroll_kind) => {
					match scroll_kind {
						ScrollKind::ScareMonster => {
							game.dialog.message("you hear a maniacal laughter in the distance", 0);
						}
						ScrollKind::HoldMonster => {
							hold_monster(game);
						}
						ScrollKind::EnchWeapon => {
							let glow_color = get_ench_color(&game.player);
							if let Some(weapon_id) = game.player.weapon_id() {
								let weapon_name = game.player.name_of(weapon_id);
								let weapon = game.player.expect_object_mut(weapon_id);
								let plural_char = if weapon.quantity <= 1 { "s" } else { "" };
								let msg = format!("your {}glow{} {}for a moment", weapon_name, plural_char, glow_color);
								game.dialog.message(&msg, 0);
								if coin_toss() {
									weapon.hit_enchant += 1;
								} else {
									weapon.d_enchant += 1;
								}
								weapon.is_cursed = 0;
							} else {
								game.dialog.message("your hands tingle", 0);
							}
						}
						ScrollKind::EnchArmor => {
							let glow_color = get_ench_color(&game.player);
							if let Some(armor) = game.player.armor_mut() {
								let msg = format!("your armor glows {} for a moment", glow_color.trim());
								game.dialog.message(&msg, 0);
								armor.d_enchant += 1;
								armor.is_cursed = 0;
								print_stats(STAT_ARMOR, &mut game.player);
							} else {
								game.dialog.message("your skin crawls", 0);
							}
						}
						ScrollKind::Identify => {
							game.dialog.message("this is a scroll of identify", 0);
							let obj = game.player.expect_object_mut(obj_id);
							obj.identified = true;
							game.player.notes.note_mut(Scroll, scroll_kind.to_index()).status = Identified;
							idntfy(game);
						}
						ScrollKind::Teleport => {
							tele(game);
						}
						ScrollKind::Sleep => {
							game.dialog.message("you fall asleep", 0);
							take_a_nap(game);
						}
						ScrollKind::ProtectArmor => {
							if let Some(armor) = game.player.armor_mut() {
								game.dialog.message("your armor is covered by a shimmering gold shield", 0);
								armor.is_protected = 1;
								armor.is_cursed = 0;
							} else {
								game.dialog.message("your acne seems to have disappeared", 0);
							}
						}
						ScrollKind::RemoveCurse => {
							let msg = if game.player.halluc.is_active() {
								"you feel in touch with the universal oneness"
							} else {
								"you feel as though someone is watching over you"
							};
							game.dialog.message(msg, 0);
							uncurse_all(&mut game.player);
						}
						ScrollKind::CreateMonster => {
							create_monster(game);
						}
						ScrollKind::AggravateMonster => {
							aggravate(game);
						}
						ScrollKind::MagicMapping => {
							game.dialog.message("this scroll seems to have a map on it", 0);
							draw_magic_map(&mut game.mash, &mut game.level);
						}
					}
					game.player.notes.identify_if_un_called(Scroll, scroll_kind.to_index());
					vanish(obj_id, scroll_kind != ScrollKind::Sleep, game);
				}
			}
		}
	}
}

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

fn idntfy(game: &mut GameState) {
	loop {
		let ch = pack_letter("what would you like to identify?", AllObjects, game);
		if ch == CANCEL {
			return;
		}
		match game.player.object_id_with_letter(ch) {
			None => {
				game.dialog.message("no such item, try again", 0);
				game.dialog.message("", 0);
				game.dialog.clear_message();
				continue;
			}
			Some(obj_id) => {
				let obj = game.player.expect_object_mut(obj_id);
				obj.identified = true;
				let what = obj.what_is;
				match what {
					Scroll | Potion | Weapon | Armor | Wand | Ring => {
						let kind = obj.which_kind as usize;
						game.player.notes.identify(what, kind);
					}
					_ => {}
				}
				let msg = game.player.get_obj_desc(obj_id);
				game.dialog.message(&msg, 0);
			}
		}
	}
}


pub fn eat(game: &mut GameState) {
	let ch = pack_letter("eat what?", Foods, game);
	if ch == CANCEL {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			if game.player.object_what(obj_id) != Food {
				game.dialog.message("you can't eat that", 0);
				return;
			}
			let kind = game.player.object_kind(obj_id);
			let moves = if kind == FRUIT || rand_percent(60) {
				let msg = if kind == RATION {
					"yum, that tasted good".to_string()
				} else {
					format!("my, that was a yummy {}", &game.player.settings.fruit)
				};
				game.dialog.message(&msg, 0);
				get_rand(900, 1100)
			} else {
				game.dialog.message("yuk, that food tasted awful", 0);
				add_exp(2, true, game);
				get_rand(700, 900)
			};
			game.player.rogue.moves_left /= 3;
			game.player.rogue.moves_left += moves;
			game.player.hunger = HungerLevel::Normal;
			print_stats(STAT_HUNGER, &mut game.player);
			vanish(obj_id, true, game);
		}
	}
}

fn hold_monster(game: &mut GameState) {
	let mut mcount = 0;
	for i in -2..=2 {
		for j in -2..=2 {
			let row = game.player.rogue.row + i;
			let col = game.player.rogue.col + j;
			if is_off_screen(row, col) {
				continue;
			}
			if game.level.dungeon[row as usize][col as usize].has_monster() {
				let monster = game.mash.monster_at_spot_mut(row, col).expect("monster at spot");
				monster.m_flags.asleep = true;
				monster.m_flags.wakens = false;
				mcount += 1;
			}
		}
	}
	if mcount == 0 {
		game.dialog.message("you feel a strange sense of loss", 0);
	} else if mcount == 1 {
		game.dialog.message("the monster freezes", 0);
	} else {
		game.dialog.message("the monsters around you freeze", 0);
	}
}

pub fn tele(game: &mut GameState) {
	let exit_spot = game.player.to_spot();
	render_system::show_darkened_room_after_player_exit(exit_spot, game);
	let avoid_room = game.player.cur_room;
	put_player(avoid_room, game);
	game.level.being_held = false;
	game.level.bear_trap = 0;
}


pub fn unhallucinate(game: &mut GameState) {
	game.player.halluc.clear();
	relight(game);
	game.player.interrupt_and_slurp();
	game.dialog.message("everything looks SO boring now", 1);
}

pub fn unblind(game: &mut GameState) {
	game.player.blind.clear();
	game.player.interrupt_and_slurp();
	game.dialog.message("the veil of darkness lifts", 1);
	relight(game);
	if game.player.halluc.is_active() {
		render_system::show_hallucination(game);
	}
	if game.level.detect_monster {
		show_monsters(game);
	}
}

pub fn relight(game: &mut GameState) {
	match game.player.cur_room {
		RoomMark::None => {}
		RoomMark::Passage => {
			render_system::show_spot_surroundings(game.player.rogue.row, game.player.rogue.col, game);
		}
		RoomMark::Cavern(cur_room) => {
			light_up_room(cur_room, game);
		}
	}
	render_system::show_player(game);
}

pub fn take_a_nap(game: &mut GameState) {
	let mut i = get_rand(2, 5);
	render_system::await_frame();
	while i > 0 {
		i -= 1;
		mv_mons(game);
		render_system::await_frame()
	}
	game.dialog.message(YOU_CAN_MOVE_AGAIN, 0);
}

pub fn get_ench_color(player: &Player) -> &'static str {
	if player.halluc.is_active() {
		ALL_POTION_COLORS[get_rand(0, POTIONS - 1)].name()
	} else {
		"blue "
	}
}

pub fn confuse(player: &mut Player) {
	let amount = get_rand(12, 22);
	player.confused.extend(amount);
}

pub fn unconfuse(game: &mut GameState) {
	game.player.confused.clear();
	let feeling = if game.player.halluc.is_active() { "trippy" } else { "confused" };
	let msg = format!("you feel less {} now", feeling);
	game.player.interrupt_and_slurp();
	game.dialog.message(&msg, 1);
}

fn uncurse_all(player: &mut Player) {
	for obj_id in player.object_ids() {
		let obj = player.expect_object_mut(obj_id);
		obj.is_cursed = 0;
	}
}
