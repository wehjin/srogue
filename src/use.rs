#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{addch, chtype, mvaddch, mvinch};
use crate::objects::IdStatus::{Called, Identified};
use crate::prelude::*;
use crate::prelude::food_kind::{FRUIT, RATION};
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_EITHER_HAND};
use crate::prelude::object_what::ObjectWhat::{Armor, Food, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Foods, Potions, Scrolls};
use crate::prelude::potion_kind::{PotionKind, POTIONS};
use crate::prelude::scroll_kind::{ScrollKind};
use crate::prelude::stat_const::{STAT_ARMOR, STAT_HP, STAT_HUNGER, STAT_STRENGTH};
use crate::settings::fruit;

pub static mut halluc: usize = 0;
pub static mut blind: usize = 0;
pub static mut confused: usize = 0;
pub static mut levitate: usize = 0;
pub static mut haste_self: usize = 0;
pub static mut extra_hp: isize = 0;
pub static strange_feeling: &'static str = "you have a strange feeling for a moment, then it passes";

pub unsafe fn quaff(depth: &RogueDepth, level: &mut Level) {
	let ch = pack_letter("quaff what?", Potions);
	if ch == CANCEL {
		return;
	}

	let obj = get_letter_object(ch);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*obj).what_is != Potion {
		message("you can't drink that", 0);
		return;
	}

	let potion_kind = PotionKind::from_index((*obj).which_kind as usize);
	match potion_kind {
		PotionKind::IncreaseStrength => {
			message("you feel stronger now, what bulging muscles!", 0);
			rogue.str_current += 1;
			if rogue.str_current > rogue.str_max {
				rogue.str_max = rogue.str_current;
			}
		}
		PotionKind::RestoreStrength => {
			rogue.str_current = rogue.str_max;
			message("this tastes great, you feel warm all over", 0);
		}
		PotionKind::Healing => {
			message("you begin to feel better", 0);
			potion_heal(false, level);
		}
		PotionKind::ExtraHealing => {
			message("you begin to feel much better", 0);
			potion_heal(true, level);
		}
		PotionKind::Poison => {
			if !sustain_strength {
				rogue.str_current -= get_rand(1, 3);
				if rogue.str_current < 1 {
					rogue.str_current = 1;
				}
			}
			message("you feel very sick now", 0);
			if halluc != 0 {
				unhallucinate(level);
			}
		}
		PotionKind::RaiseLevel => {
			rogue.exp_points = LEVEL_POINTS[(rogue.exp - 1) as usize];
			add_exp(1, true, depth.cur);
		}
		PotionKind::Blindness => {
			go_blind(level);
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
			if !level_objects.next_object.is_null() {
				if blind == 0 {
					show_objects(level);
				}
			} else {
				message(strange_feeling, 0);
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
				unblind(level);
			}
			level.see_invisible = true;
			relight(level);
		}
	}
	print_stats(STAT_STRENGTH | STAT_HP, depth.cur);
	if id_potions[potion_kind.to_index()].id_status != Called {
		id_potions[potion_kind.to_index()].id_status = Identified;
	}
	vanish(&mut *obj, true, &mut rogue.pack, depth, level);
}

pub unsafe fn read_scroll(depth: &RogueDepth, level: &mut Level) {
	if blind != 0 {
		message("You can't see to read the scroll.", 0);
		return;
	}

	let ch = pack_letter("read what?", Scrolls);
	if ch == CANCEL {
		return;
	}

	let obj = get_letter_object(ch);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*obj).what_is != Scroll {
		message("you can't read that", 0);
		return;
	}

	let scroll_kind = ScrollKind::from_index((*obj).which_kind as usize);
	match scroll_kind {
		ScrollKind::ScareMonster => {
			message("you hear a maniacal laughter in the distance", 0);
		}
		ScrollKind::HoldMonster => {
			hold_monster(level);
		}
		ScrollKind::EnchWeapon => {
			if !rogue.weapon.is_null() {
				if (*rogue.weapon).what_is == Weapon {
					message(&format!(
						"your {}glow{} {}for a moment",
						name_of(&*rogue.weapon),
						if (*rogue.weapon).quantity <= 1 { "s" } else { "" },
						get_ench_color()
					), 0);
					if coin_toss() {
						(*rogue.weapon).hit_enchant += 1;
					} else {
						(*rogue.weapon).d_enchant += 1;
					}
				}
				(*rogue.weapon).is_cursed = 0;
			} else {
				message("your hands tingle", 0);
			}
		}
		ScrollKind::EnchArmor => {
			if !rogue.armor.is_null() {
				message(&format!("your armor glows {}for a moment", get_ench_color(), ), 0);
				(*rogue.armor).d_enchant += 1;
				(*rogue.armor).is_cursed = 0;
				print_stats(STAT_ARMOR, depth.cur);
			} else {
				message("your skin crawls", 0);
			}
		}
		ScrollKind::Identify => {
			message("this is a scroll of identify", 0);
			(*obj).identified = true;
			id_scrolls[(*obj).which_kind as usize].id_status = Identified;
			idntfy();
		}
		ScrollKind::Teleport => {
			tele(level);
		}
		ScrollKind::Sleep => {
			message("you fall asleep", 0);
			take_a_nap(depth, level);
		}
		ScrollKind::ProtectArmor => {
			if !rogue.armor.is_null() {
				message("your armor is covered by a shimmering gold shield", 0);
				(*rogue.armor).is_protected = 1;
				(*rogue.armor).is_cursed = 0;
			} else {
				message("your acne seems to have disappeared", 0);
			}
		}
		ScrollKind::RemoveCurse => {
			message(if !player_hallucinating() {
				"you feel as though someone is watching over you"
			} else {
				"you feel in touch with the universal oneness"
			}, 0);
			uncurse_all();
		}
		ScrollKind::CreateMonster => {
			create_monster(depth.cur, level);
		}
		ScrollKind::AggravateMonster => {
			aggravate(level);
		}
		ScrollKind::MagicMapping => {
			message("this scroll seems to have a map on it", 0);
			draw_magic_map(level);
		}
	}
	if id_scrolls[scroll_kind.to_index()].id_status != Called {
		id_scrolls[scroll_kind.to_index()].id_status = Identified;
	}
	vanish(&mut *obj, scroll_kind != ScrollKind::Sleep, &mut rogue.pack, depth, level);
}

pub unsafe fn vanish(obj: &mut obj, do_regular_move: bool, pack: &mut obj, depth: &RogueDepth, level: &mut Level) {
	/* vanish() does NOT handle a quiver of weapons with more than one
	   arrow (or whatever) in the quiver.  It will only decrement the count.
	*/
	if (*obj).quantity > 1 {
		(*obj).quantity -= 1;
	} else {
		if ((*obj).in_use_flags & BEING_WIELDED) != 0 {
			unwield(obj);
		} else if ((*obj).in_use_flags & BEING_WORN) != 0 {
			unwear(obj);
		} else if ((*obj).in_use_flags & ON_EITHER_HAND) != 0 {
			un_put_on(obj, depth.cur, level);
		}
		take_from_pack(obj, pack);
		free_object(obj);
	}
	if do_regular_move {
		reg_move(depth, level);
	}
}

unsafe fn potion_heal(extra: bool, level: &mut Level) {
	rogue.hp_current += rogue.exp;

	let mut ratio = rogue.hp_current as f32 / rogue.hp_max as f32;
	if ratio >= 1.00 {
		rogue.hp_max += if extra { 2 } else { 1 };
		extra_hp += if extra { 2 } else { 1 };
		rogue.hp_current = rogue.hp_max;
	} else if ratio >= 0.90 {
		rogue.hp_max += if extra { 1 } else { 0 };
		extra_hp += if extra { 1 } else { 0 };
		rogue.hp_current = rogue.hp_max;
	} else {
		if ratio < 0.33 {
			ratio = 0.33;
		}
		if extra {
			ratio += ratio;
		}
		let add = ratio * (rogue.hp_max - rogue.hp_current) as f32;
		rogue.hp_current += add as isize;
		if rogue.hp_current > rogue.hp_max {
			rogue.hp_current = rogue.hp_max;
		}
	}
	if blind != 0 {
		unblind(level);
	}
	if confused != 0 && extra {
		unconfuse();
	} else if confused != 0 {
		confused = (confused / 2) + 1;
	}
	if halluc != 0 && extra {
		unhallucinate(level);
	} else if halluc != 0 {
		halluc = (halluc / 2) + 1;
	}
}

unsafe fn idntfy() {
	loop {
		let ch = pack_letter("what would you like to identify?", AllObjects);
		if ch == CANCEL {
			return;
		}

		let obj = get_letter_object(ch);
		if obj.is_null() {
			message("no such item, try again", 0);
			message("", 0);
			check_message();
			continue;
		}

		(*obj).identified = true;
		match (*obj).what_is {
			Scroll | Potion | Weapon | Armor | Wand | Ring => {
				let id_table = get_id_table(&*obj);
				id_table[(*obj).which_kind as usize].id_status = Identified;
			}
			_ => {}
		}
		message(&get_desc(&*obj), 0);
	}
}


pub unsafe fn eat(depth: &RogueDepth, level: &mut Level) {
	let ch = pack_letter("eat what?", Foods);
	if ch == CANCEL {
		return;
	}

	let obj = get_letter_object(ch);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	if (*obj).what_is != Food {
		message("you can't eat that", 0);
		return;
	}

	let moves = if (*obj).which_kind == FRUIT || rand_percent(60) {
		if (*obj).which_kind == RATION {
			message("yum, that tasted good", 0);
		} else {
			message(&format!("my, that was a yummy {}", &fruit()), 0);
		}
		get_rand(900, 1100)
	} else {
		message("yuk, that food tasted awful", 0);
		add_exp(2, true, depth.cur);
		get_rand(700, 900)
	};
	rogue.moves_left /= 3;
	rogue.moves_left += moves;
	hunger_str.clear();
	print_stats(STAT_HUNGER, depth.cur);

	vanish(&mut *obj, true, &mut rogue.pack, depth, level);
}

unsafe fn hold_monster(level: &Level) {
	let mut mcount = 0;
	for i in -2..=2 {
		for j in -2..=2 {
			let row = rogue.row + i;
			let col = rogue.col + j;
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

pub unsafe fn tele(level: &mut Level) {
	mvaddch(rogue.row as i32, rogue.col as i32, get_dungeon_char(rogue.row, rogue.col, level));

	if cur_room >= 0 {
		darken_room(cur_room, level);
	}
	put_player(get_opt_room_number(rogue.row, rogue.col, level), level);
	level.being_held = false;
	level.bear_trap = 0;
}

pub unsafe fn hallucinate() {
	if blind != 0 {
		return;
	}
	let mut obj = level_objects.next_object;
	while !obj.is_null() {
		let ch = mvinch((*obj).row as i32, (*obj).col as i32);
		if !is_monster_char(ch) && no_rogue((*obj).row, (*obj).col) {
			let should_overdraw = match ch as u8 as char {
				' ' | '.' | '#' | '+' => false,
				_ => true
			};
			if should_overdraw {
				addch(gr_obj_char() as chtype);
			}
		}
		obj = (*obj).next_object;
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

pub unsafe fn unhallucinate(level: &mut Level) {
	halluc = 0;
	relight(level);
	message("everything looks SO boring now", 1);
}

pub unsafe fn unblind(level: &mut Level)
{
	blind = 0;
	message("the veil of darkness lifts", 1);
	relight(level);
	if halluc != 0 {
		hallucinate();
	}
	if level.detect_monster {
		show_monsters(level);
	}
}

pub unsafe fn relight(level: &mut Level) {
	if cur_room == PASSAGE {
		light_passage(rogue.row, rogue.col, level);
	} else {
		light_up_room(cur_room, level);
	}
	mvaddch(rogue.row as i32, rogue.col as i32, chtype::from(rogue.fchar));
}

pub unsafe fn take_a_nap(depth: &RogueDepth, level: &mut Level) {
	let mut i = get_rand(2, 5);
	md_sleep(1);
	while i > 0 {
		i -= 1;
		mv_mons(depth, level);
	}
	md_sleep(1);
	message(YOU_CAN_MOVE_AGAIN, 0);
}

unsafe fn go_blind(level: &Level) {
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
	mvaddch(rogue.row as i32, rogue.col as i32, chtype::from(rogue.fchar));
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

unsafe fn uncurse_all() {
	let mut obj = rogue.pack.next_object;
	while !obj.is_null() {
		(*obj).is_cursed = 0;
		obj = (*obj).next_object;
	}
}
