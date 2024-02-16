#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{addch, chtype, mvaddch, mvinch};

use crate::level::{add_exp, Level, put_player};
use crate::machdep::md_sleep;
use crate::message::{CANCEL, check_message, hunger_str, message, print_stats};
use crate::monster::{aggravate, create_monster, gr_obj_char, MonsterMash, mv_mons, show_monsters};
use crate::objects::{ObjectId, ObjectPack};
use crate::objects::NoteStatus::Identified;
use crate::pack::{pack_letter, take_from_pack, unwear, unwield};
use crate::player::{Player, RoomMark};
use crate::potions::colors::ALL_POTION_COLORS;
use crate::potions::kind::POTIONS;
use crate::potions::quaff::quaff_potion;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Foods, Potions, Scrolls};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_HP, STAT_HUNGER, STAT_STRENGTH};
use crate::r#move::{reg_move, YOU_CAN_MOVE_AGAIN};
use crate::random::{coin_toss, get_rand, rand_percent};
use crate::ring::un_put_hand;
use crate::room::{darken_room, draw_magic_map, get_dungeon_char, light_passage, light_up_room};
use crate::scrolls::ScrollKind;
use crate::trap::is_off_screen;

pub const STRANGE_FEELING: &'static str = "you have a strange feeling for a moment, then it passes";

pub unsafe fn quaff(mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	let ch = pack_letter("quaff what?", Potions, player);
	if ch == CANCEL {
		return;
	}
	match player.object_id_with_letter(ch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			match player.expect_object(obj_id).potion_kind() {
				None => {
					message("you can't drink that", 0);
					return;
				}
				Some(potion_kind) => {
					quaff_potion(potion_kind, mash, player, level, ground);
					print_stats(STAT_STRENGTH | STAT_HP, player);
					player.notes.identify_if_un_called(Potion, potion_kind.to_index());
					vanish(obj_id, true, mash, player, level, ground);
				}
			}
		}
	}
}

pub unsafe fn read_scroll(mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	if player.blind.is_active() {
		message("You can't see to read the scroll.", 0);
		return;
	}

	let ch = pack_letter("read what?", Scrolls, player);
	if ch == CANCEL {
		return;
	}
	match player.object_id_with_letter(ch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			match player.expect_object(obj_id).scroll_kind() {
				None => {
					message("you can't read that", 0);
					return;
				}
				Some(scroll_kind) => {
					match scroll_kind {
						ScrollKind::ScareMonster => {
							message("you hear a maniacal laughter in the distance", 0);
						}
						ScrollKind::HoldMonster => {
							hold_monster(mash, player, level);
						}
						ScrollKind::EnchWeapon => {
							let glow_color = get_ench_color(player);
							if let Some(weapon_id) = player.weapon_id() {
								let weapon_name = player.name_of(weapon_id);
								let weapon = player.expect_object_mut(weapon_id);
								let plural_char = if weapon.quantity <= 1 { "s" } else { "" };
								let msg = format!("your {}glow{} {}for a moment", weapon_name, plural_char, glow_color);
								message(&msg, 0);
								if coin_toss() {
									weapon.hit_enchant += 1;
								} else {
									weapon.d_enchant += 1;
								}
								weapon.is_cursed = 0;
							} else {
								message("your hands tingle", 0);
							}
						}
						ScrollKind::EnchArmor => {
							let glow_color = get_ench_color(player);
							if let Some(armor) = player.armor_mut() {
								let msg = format!("your armor glows {} for a moment", glow_color.trim());
								message(&msg, 0);
								armor.d_enchant += 1;
								armor.is_cursed = 0;
								print_stats(STAT_ARMOR, player);
							} else {
								message("your skin crawls", 0);
							}
						}
						ScrollKind::Identify => {
							message("this is a scroll of identify", 0);
							let obj = player.expect_object_mut(obj_id);
							obj.identified = true;
							player.notes.note_mut(Scroll, scroll_kind.to_index()).status = Identified;
							idntfy(player);
						}
						ScrollKind::Teleport => {
							tele(mash, player, level);
						}
						ScrollKind::Sleep => {
							message("you fall asleep", 0);
							take_a_nap(mash, player, level, ground);
						}
						ScrollKind::ProtectArmor => {
							if let Some(armor) = player.armor_mut() {
								message("your armor is covered by a shimmering gold shield", 0);
								armor.is_protected = 1;
								armor.is_cursed = 0;
							} else {
								message("your acne seems to have disappeared", 0);
							}
						}
						ScrollKind::RemoveCurse => {
							let msg = if player.halluc.is_active() {
								"you feel in touch with the universal oneness"
							} else {
								"you feel as though someone is watching over you"
							};
							message(msg, 0);
							uncurse_all(player);
						}
						ScrollKind::CreateMonster => {
							create_monster(mash, player, level);
						}
						ScrollKind::AggravateMonster => {
							aggravate(mash, player, level);
						}
						ScrollKind::MagicMapping => {
							message("this scroll seems to have a map on it", 0);
							draw_magic_map(mash, level);
						}
					}
					player.notes.identify_if_un_called(Scroll, scroll_kind.to_index());
					vanish(obj_id, scroll_kind != ScrollKind::Sleep, mash, player, level, ground);
				}
			}
		}
	}
}

pub unsafe fn vanish(obj_id: ObjectId, do_regular_move: bool, mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	/* vanish() does NOT handle a quiver of weapons with more than one
	   arrow (or whatever) in the quiver.  It will only decrement the count.
	*/
	let obj = player.object_mut(obj_id).expect("obj in player");
	if obj.quantity > 1 {
		obj.quantity -= 1;
	} else {
		if obj.is_being_wielded() {
			unwield(player);
		} else if obj.is_being_worn() {
			unwear(player);
		} else if let Some(hand) = player.ring_hand(obj_id) {
			un_put_hand(hand, mash, player, level);
		}
		take_from_pack(obj_id, &mut player.rogue.pack);
	}
	if do_regular_move {
		reg_move(mash, player, level, ground);
	}
}

unsafe fn idntfy(player: &mut Player) {
	loop {
		let ch = pack_letter("what would you like to identify?", AllObjects, player);
		if ch == CANCEL {
			return;
		}
		match player.object_id_with_letter(ch) {
			None => {
				message("no such item, try again", 0);
				message("", 0);
				check_message();
				continue;
			}
			Some(obj_id) => {
				let obj = player.expect_object_mut(obj_id);
				obj.identified = true;
				let what = obj.what_is;
				match what {
					Scroll | Potion | Weapon | Armor | Wand | Ring => {
						let kind = obj.which_kind as usize;
						player.notes.identify(what, kind);
					}
					_ => {}
				}
				let msg = player.get_obj_desc(obj_id);
				message(&msg, 0);
			}
		}
	}
}


pub unsafe fn eat(mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	let ch = pack_letter("eat what?", Foods, player);
	if ch == CANCEL {
		return;
	}
	match player.object_id_with_letter(ch) {
		None => {
			message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			if player.object_what(obj_id) != Food {
				message("you can't eat that", 0);
				return;
			}
			let kind = player.object_kind(obj_id);
			let moves = if kind == FRUIT || rand_percent(60) {
				let msg = if kind == RATION {
					"yum, that tasted good".to_string()
				} else {
					format!("my, that was a yummy {}", &player.settings.fruit)
				};
				message(&msg, 0);
				get_rand(900, 1100)
			} else {
				message("yuk, that food tasted awful", 0);
				add_exp(2, true, player);
				get_rand(700, 900)
			};
			player.rogue.moves_left /= 3;
			player.rogue.moves_left += moves;
			hunger_str.clear();
			print_stats(STAT_HUNGER, player);
			vanish(obj_id, true, mash, player, level, ground);
		}
	}
}

unsafe fn hold_monster(mash: &mut MonsterMash, player: &Player, level: &Level) {
	let mut mcount = 0;
	for i in -2..=2 {
		for j in -2..=2 {
			let row = player.rogue.row + i;
			let col = player.rogue.col + j;
			if is_off_screen(row, col) {
				continue;
			}
			if level.dungeon[row as usize][col as usize].has_monster() {
				let monster = mash.monster_at_spot_mut(row, col).expect("monster at spot");
				monster.m_flags.asleep = true;
				monster.m_flags.wakens = false;
				mcount += 1;
			}
		}
	}
	if mcount == 0 {
		message("you feel a strange sense of loss", 0);
	} else if mcount == 1 {
		message("the monster freezes", 0);
	} else {
		message("the monsters around you freeze", 0);
	}
}

pub unsafe fn tele(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, get_dungeon_char(player.rogue.row, player.rogue.col, mash, player, level));
	if let RoomMark::Area(cur_room) = player.cur_room {
		darken_room(cur_room, mash, player, level);
	}
	let avoid_room = player.cur_room;
	put_player(avoid_room, mash, player, level);
	level.being_held = false;
	level.bear_trap = 0;
}

pub unsafe fn hallucinate_on_screen(mash: &mut MonsterMash, player: &Player, ground: &ObjectPack) {
	if player.blind.is_active() {
		return;
	}

	for obj in ground.objects() {
		let ch = mvinch(obj.row as i32, obj.col as i32);
		let row = obj.row;
		let col = obj.col;
		if !is_monster_char(ch) && !player.is_at(row, col) {
			let should_overdraw = match ch as u8 as char {
				' ' | '.' | '#' | '+' => false,
				_ => true
			};
			if should_overdraw {
				addch(gr_obj_char() as chtype);
			}
		}
	}
	for monster in &mash.monsters {
		let ch = mvinch(monster.spot.row as i32, monster.spot.col as i32);
		if is_monster_char(ch) {
			addch(get_rand(chtype::from('A'), chtype::from('Z')));
		}
	}
}

pub fn is_monster_char(ch: chtype) -> bool {
	match ch as u8 as char {
		'A'..='Z' => true,
		_ => false,
	}
}

pub unsafe fn unhallucinate(mash: &mut MonsterMash, player: &mut Player, level: &mut Level) {
	player.halluc.clear();
	relight(mash, player, level);
	message("everything looks SO boring now", 1);
}

pub unsafe fn unblind(mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	player.blind.clear();
	message("the veil of darkness lifts", 1);
	relight(mash, player, level);
	if player.halluc.is_active() {
		hallucinate_on_screen(mash, player, ground);
	}
	if level.detect_monster {
		show_monsters(mash, player, level);
	}
}

pub unsafe fn relight(mash: &mut MonsterMash, player: &Player, level: &mut Level) {
	match player.cur_room {
		RoomMark::None => {}
		RoomMark::Passage => {
			light_passage(player.rogue.row, player.rogue.col, mash, player, level);
		}
		RoomMark::Area(cur_room) => {
			light_up_room(cur_room, mash, player, level);
		}
	}
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, chtype::from(player.rogue.fchar));
}

pub unsafe fn take_a_nap(mash: &mut MonsterMash, player: &mut Player, level: &mut Level, ground: &ObjectPack) {
	let mut i = get_rand(2, 5);
	md_sleep(1);
	while i > 0 {
		i -= 1;
		mv_mons(mash, player, level, ground);
	}
	md_sleep(1);
	message(YOU_CAN_MOVE_AGAIN, 0);
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

pub unsafe fn unconfuse(player: &mut Player) {
	player.confused.clear();
	let feeling = if player.halluc.is_active() { "trippy" } else { "confused" };
	let msg = format!("you feel less {} now", feeling);
	message(&msg, 1);
}

fn uncurse_all(player: &mut Player) {
	for obj_id in player.object_ids() {
		let obj = player.expect_object_mut(obj_id);
		obj.is_cursed = 0;
	}
}
