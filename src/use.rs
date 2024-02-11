#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments)]

use ncurses::{addch, chtype, mvaddch, mvinch};
use crate::objects::IdStatus::{Called, Identified};
use crate::player::Player;
use crate::prelude::*;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Foods, Potions, Scrolls};
use crate::prelude::potion_kind::{PotionKind, POTIONS};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_HP, STAT_HUNGER, STAT_STRENGTH};
use crate::scrolls::ScrollKind;
use crate::settings::fruit;

pub static mut halluc: usize = 0;
pub static mut blind: usize = 0;
pub static mut confused: usize = 0;
pub static mut levitate: usize = 0;
pub static mut haste_self: usize = 0;
pub static mut extra_hp: isize = 0;
pub static strange_feeling: &'static str = "you have a strange feeling for a moment, then it passes";

pub unsafe fn quaff(player: &mut Player, level: &mut Level) {
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
					match potion_kind {
						PotionKind::IncreaseStrength => {
							message("you feel stronger now, what bulging muscles!", 0);
							player.rogue.str_current += 1;
							if player.rogue.str_current > player.rogue.str_max {
								player.rogue.str_max = player.rogue.str_current;
							}
						}
						PotionKind::RestoreStrength => {
							player.rogue.str_current = player.rogue.str_max;
							message("this tastes great, you feel warm all over", 0);
						}
						PotionKind::Healing => {
							message("you begin to feel better", 0);
							potion_heal(false, player, level);
						}
						PotionKind::ExtraHealing => {
							message("you begin to feel much better", 0);
							potion_heal(true, player, level);
						}
						PotionKind::Poison => {
							if !sustain_strength {
								player.rogue.str_current -= get_rand(1, 3);
								if player.rogue.str_current < 1 {
									player.rogue.str_current = 1;
								}
							}
							message("you feel very sick now", 0);
							if halluc != 0 {
								unhallucinate(player, level);
							}
						}
						PotionKind::RaiseLevel => {
							player.rogue.exp_points = LEVEL_POINTS[(player.rogue.exp - 1) as usize];
							add_exp(1, true, player);
						}
						PotionKind::Blindness => {
							go_blind(player, level);
						}
						PotionKind::Hallucination => {
							message("oh wow, everything seems so cosmic", 0);
							halluc += get_rand(500, 800);
						}
						PotionKind::DetectMonster => {
							show_monsters(level);
							if MASH.is_empty() {
								message(strange_feeling, 0);
							}
						}
						PotionKind::DetectObjects => {
							if level_objects.is_empty() {
								message(strange_feeling, 0);
							} else {
								if blind == 0 {
									show_objects(player, level);
								}
							}
						}
						PotionKind::Confusion => {
							message(if halluc != 0 { "what a trippy feeling" } else { "you feel confused" }, 0);
							confuse();
						}
						PotionKind::Levitation => {
							message("you start to float in the air", 0);
							levitate += get_rand(15, 30);
							level.bear_trap = 0;
							level.being_held = false;
						}
						PotionKind::HasteSelf => {
							message("you feel yourself moving much faster", 0);
							haste_self += get_rand(11, 21);
							if haste_self % 2 == 0 {
								haste_self += 1;
							}
						}
						PotionKind::SeeInvisible => {
							message(&format!("hmm, this potion tastes like {}juice", fruit()), 0);
							if blind != 0 {
								unblind(player, level);
							}
							level.see_invisible = true;
							relight(player, level);
						}
					}
					print_stats(STAT_STRENGTH | STAT_HP, player);
					if id_potions[potion_kind.to_index()].id_status != Called {
						id_potions[potion_kind.to_index()].id_status = Identified;
					}
					vanish(obj_id, true, player, level);
				}
			}
		}
	}
}

pub unsafe fn read_scroll(player: &mut Player, level: &mut Level) {
	if blind != 0 {
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
							hold_monster(player, level);
						}
						ScrollKind::EnchWeapon => {
							if let Some(weapon) = player.weapon_mut() {
								let weapon_name = name_of(weapon);
								let plural_char = if weapon.quantity <= 1 { "s" } else { "" };
								let glow_color = get_ench_color();
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
							if let Some(armor) = player.armor_mut() {
								let msg = format!("your armor glows {}for a moment", get_ench_color());
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
							id_scrolls[obj.which_kind as usize].id_status = Identified;
							idntfy(player);
						}
						ScrollKind::Teleport => {
							tele(player, level);
						}
						ScrollKind::Sleep => {
							message("you fall asleep", 0);
							take_a_nap(player, level);
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
							let msg = if !player_hallucinating() {
								"you feel as though someone is watching over you"
							} else {
								"you feel in touch with the universal oneness"
							};
							message(msg, 0);
							uncurse_all(player);
						}
						ScrollKind::CreateMonster => {
							create_monster(player, level);
						}
						ScrollKind::AggravateMonster => {
							aggravate(player, level);
						}
						ScrollKind::MagicMapping => {
							message("this scroll seems to have a map on it", 0);
							draw_magic_map(level);
						}
					}
					if id_scrolls[scroll_kind.to_index()].id_status != Called {
						id_scrolls[scroll_kind.to_index()].id_status = Identified;
					}
					vanish(obj_id, scroll_kind != ScrollKind::Sleep, player, level);
				}
			}
		}
	}
}

pub unsafe fn vanish(obj_id: ObjectId, do_regular_move: bool, player: &mut Player, level: &mut Level) {
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
			un_put_hand(hand, player, level);
		}
		take_from_pack(obj_id, &mut player.rogue.pack);
	}
	if do_regular_move {
		reg_move(player, level);
	}
}

unsafe fn potion_heal(extra: bool, player: &mut Player, level: &mut Level) {
	player.rogue.hp_current += player.rogue.exp;

	let mut ratio = player.rogue.hp_current as f32 / player.rogue.hp_max as f32;
	if ratio >= 1.00 {
		player.rogue.hp_max += if extra { 2 } else { 1 };
		extra_hp += if extra { 2 } else { 1 };
		player.rogue.hp_current = player.rogue.hp_max;
	} else if ratio >= 0.90 {
		player.rogue.hp_max += if extra { 1 } else { 0 };
		extra_hp += if extra { 1 } else { 0 };
		player.rogue.hp_current = player.rogue.hp_max;
	} else {
		if ratio < 0.33 {
			ratio = 0.33;
		}
		if extra {
			ratio += ratio;
		}
		let add = ratio * (player.rogue.hp_max - player.rogue.hp_current) as f32;
		player.rogue.hp_current += add as isize;
		if player.rogue.hp_current > player.rogue.hp_max {
			player.rogue.hp_current = player.rogue.hp_max;
		}
	}
	if blind != 0 {
		unblind(player, level);
	}
	if confused != 0 && extra {
		unconfuse();
	} else if confused != 0 {
		confused = (confused / 2) + 1;
	}
	if halluc != 0 && extra {
		unhallucinate(player, level);
	} else if halluc != 0 {
		halluc = (halluc / 2) + 1;
	}
}

unsafe fn idntfy(player: &mut Player) {
	loop {
		let ch = pack_letter("what would you like to identify?", AllObjects, player);
		if ch == CANCEL {
			return;
		}
		match player.object_with_letter_mut(ch) {
			None => {
				message("no such item, try again", 0);
				message("", 0);
				check_message();
				continue;
			}
			Some(obj) => {
				obj.identified = true;
				match obj.what_is {
					Scroll | Potion | Weapon | Armor | Wand | Ring => {
						let id_table = get_id_table(obj);
						id_table[obj.which_kind as usize].id_status = Identified;
					}
					_ => {}
				}
				let obj_desc = get_obj_desc(obj);
				message(&obj_desc, 0);
			}
		}
	}
}


pub unsafe fn eat(player: &mut Player, level: &mut Level) {
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
					format!("my, that was a yummy {}", &fruit())
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
			vanish(obj_id, true, player, level);
		}
	}
}

unsafe fn hold_monster(player: &Player, level: &Level) {
	let mut mcount = 0;
	for i in -2..=2 {
		for j in -2..=2 {
			let row = player.rogue.row + i;
			let col = player.rogue.col + j;
			if is_off_screen(row, col) {
				continue;
			}
			if level.dungeon[row as usize][col as usize].is_monster() {
				let monster = MASH.monster_at_spot_mut(row, col).expect("monster at spot");
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

pub unsafe fn tele(player: &mut Player, level: &mut Level) {
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, get_dungeon_char(player.rogue.row, player.rogue.col, level));

	if cur_room >= 0 {
		darken_room(cur_room, level);
	}
	put_player(get_opt_room_number(player.rogue.row, player.rogue.col, level), player, level);
	level.being_held = false;
	level.bear_trap = 0;
}

pub unsafe fn hallucinate(player: &Player) {
	if blind != 0 {
		return;
	}

	for obj in level_objects.objects() {
		let ch = mvinch(obj.row as i32, obj.col as i32);
		if !is_monster_char(ch) && no_rogue(obj.row, obj.col, player) {
			let should_overdraw = match ch as u8 as char {
				' ' | '.' | '#' | '+' => false,
				_ => true
			};
			if should_overdraw {
				addch(gr_obj_char() as chtype);
			}
		}
	}
	for monster in &MASH.monsters {
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

pub unsafe fn unhallucinate(player: &Player, level: &mut Level) {
	halluc = 0;
	relight(player, level);
	message("everything looks SO boring now", 1);
}

pub unsafe fn unblind(player: &Player, level: &mut Level)
{
	blind = 0;
	message("the veil of darkness lifts", 1);
	relight(player, level);
	if halluc != 0 {
		hallucinate(player);
	}
	if level.detect_monster {
		show_monsters(level);
	}
}

pub unsafe fn relight(player: &Player, level: &mut Level) {
	if cur_room == PASSAGE {
		light_passage(player.rogue.row, player.rogue.col, level);
	} else {
		light_up_room(cur_room, player, level);
	}
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, chtype::from(player.rogue.fchar));
}

pub unsafe fn take_a_nap(player: &mut Player, level: &mut Level) {
	let mut i = get_rand(2, 5);
	md_sleep(1);
	while i > 0 {
		i -= 1;
		mv_mons(player, level);
	}
	md_sleep(1);
	message(YOU_CAN_MOVE_AGAIN, 0);
}

unsafe fn go_blind(player: &Player, level: &Level) {
	if blind == 0 {
		message("a cloak of darkness falls around you", 0);
	}
	blind += get_rand(500, 800);

	if level.detect_monster {
		for monster in &MASH.monsters {
			mvaddch(monster.spot.row as i32, monster.spot.col as i32, monster.trail_char);
		}
	}
	if cur_room >= 0 {
		for i in (level.rooms[cur_room as usize].top_row as usize + 1)..level.rooms[cur_room as usize].bottom_row as usize {
			for j in (level.rooms[cur_room as usize].left_col as usize + 1)..level.rooms[cur_room as usize].right_col as usize {
				mvaddch(i as i32, j as i32, chtype::from(' '));
			}
		}
	}
	mvaddch(player.rogue.row as i32, player.rogue.col as i32, chtype::from(player.rogue.fchar));
}

pub unsafe fn get_ench_color() -> &'static str {
	if halluc != 0 {
		return PotionKind::from_index(get_rand(0, POTIONS - 1)).title();
	}
	return "blue ";
}

pub unsafe fn confuse() {
	confused += get_rand(12, 22);
}

pub unsafe fn unconfuse() {
	confused = 0;
	let msg = format!("you feel less {} now", if halluc > 0 { "trippy" } else { "confused" });
	message(&msg, 1);
}

fn uncurse_all(player: &mut Player) {
	for obj_id in player.object_ids() {
		let obj = player.expect_object_mut(obj_id);
		obj.is_cursed = 0;
	}
}
