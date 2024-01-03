#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]


use ncurses::{addch, chtype, mvaddch};
use crate::objects::IdStatus::{Called, Identified};
use crate::prelude::*;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_EITHER_HAND};
use crate::prelude::object_what::ObjectWhat::Potion;
use crate::prelude::object_what::PackFilter::{Foods, Potions, Scrolls};
use crate::prelude::potion_kind::PotionKind;
use crate::prelude::scroll_kind::SLEEP;
use crate::prelude::stat_const::{STAT_HP, STAT_STRENGTH};
use crate::settings::fruit;

pub static mut halluc: usize = 0;
pub static mut blind: usize = 0;
pub static mut confused: usize = 0;
pub static mut levitate: usize = 0;
pub static mut haste_self: usize = 0;
pub static mut see_invisible: bool = false;
pub static mut extra_hp: isize = 0;
pub static strange_feeling: &'static str = "you have a strange feeling for a moment, then it passes";

pub unsafe fn quaff() {
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
			potion_heal(false);
		}
		PotionKind::ExtraHealing => {
			message("you begin to feel much better", 0);
			potion_heal(true);
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
				unhallucinate();
			}
		}
		PotionKind::RaiseLevel => {
			rogue.exp_points = level_points[(rogue.exp - 1) as usize];
			add_exp(1, true);
		}
		PotionKind::Blindness => {
			go_blind();
		}
		PotionKind::Hallucination => {
			message("oh wow, everything seems so cosmic", 0);
			halluc += get_rand(500, 800);
		}
		PotionKind::DetectMonster => {
			show_monsters();
			if level_monsters.next_object.is_null() {
				message(strange_feeling, 0);
			}
		}
		PotionKind::DetectObjects => {
			if !level_objects.next_object.is_null() {
				if blind == 0 {
					show_objects();
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
			bear_trap = 0;
			being_held = false;
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
				unblind();
			}
			see_invisible = true;
			relight();
		}
	}
	print_stats(STAT_STRENGTH | STAT_HP);
	if id_potions[(*obj).which_kind as usize].id_status != Called {
		id_potions[(*obj).which_kind as usize].id_status = Identified;
	}
	vanish(&mut *obj, true, &mut rogue.pack);
}

#[no_mangle]
pub unsafe extern "C" fn read_scroll() -> i64 {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut msg: [libc::c_char; 80] = [0; 80];
	if blind != 0 {
		message(
			b"You can't see to read the scroll.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	ch = pack_letter("read what?", Scrolls) as libc::c_short;
	if ch as i64 == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as i64);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	if (*obj).what_is as i64
		!= 0o4 as i64 as libc::c_ushort as i64
	{
		message(
			b"you can't read that\0" as *const u8 as *const libc::c_char,
			0 as i64,
		);
		return;
	}
	match (*obj).which_kind as i64 {
		7 => {
			message(
				b"you hear a maniacal laughter in the distance\0" as *const u8
					as *const libc::c_char,
				0 as i64,
			);
		}
		1 => {
			hold_monster();
		}
		2 => {
			if !(rogue.weapon).is_null() {
				if (*rogue.weapon).what_is as i64
					== 0o2 as i64 as libc::c_ushort as i64
				{
					let msg = format!(
						"your {}glow{} {}for a moment",
						name_of(&rogue.weapon),
						if (*rogue.weapon).quantity as i64 <= 1 { "s" } else { "" },
						get_ench_color(),
					);
					message(&msg, 0 as i64);
					if coin_toss() {
						(*rogue.weapon).hit_enchant += 1;
						(*rogue.weapon).hit_enchant;
					} else {
						(*rogue.weapon).d_enchant += 1;
						(*rogue.weapon).d_enchant;
					}
				}
				(*rogue.weapon).is_cursed = 0;
			} else {
				message("your hands tingle", 0 as i64);
			}
		}
		3 => {
			if !(rogue.armor).is_null() {
				let msg = format!("your armor glows {}for a moment", get_ench_color(), );
				message(&msg, 0 as i64);
				(*rogue.armor).d_enchant += 1;
				(*rogue.armor).d_enchant;
				(*rogue.armor).is_cursed = 0;
				print_stats(0o20 as i64);
			} else {
				message("your skin crawls", 0 as i64);
			}
		}
		4 => {
			message("this is a scroll of identify", 0 as i64);
			(*obj).identified = 1 as libc::c_short;
			(*id_scrolls.as_mut_ptr().offset((*obj).which_kind as isize))
				.id_status = 0o1 as libc::c_ushort;
			idntfy();
		}
		5 => {
			tele();
		}
		6 => {
			message("you fall asleep", 0 as i64);
			take_a_nap();
		}
		0 => {
			if !(rogue.armor).is_null() {
				message("your armor is covered by a shimmering gold shield", 0 as i64);
				(*rogue.armor).is_protected = 1 as libc::c_short;
				(*rogue.armor).is_cursed = 0;
			} else {
				message("your acne seems to have disappeared", 0 as libc::c_int);
			}
		}
		8 => {
			let msg = if !player_hallucinating() {
				"you feel as though someone is watching over you"
			} else {
				"you feel in touch with the universal oneness"
			};
			message(msg, 0 as libc::c_int);
			uncurse_all();
		}
		9 => {
			create_monster();
		}
		10 => {
			aggravate();
		}
		11 => {
			message("this scroll seems to have a map on it", 0 as libc::c_int);
			draw_magic_map();
		}
		_ => {}
	}
	if (*id_scrolls.as_mut_ptr().offset((*obj).which_kind as isize)).id_status
		as libc::c_int != 0o2 as libc::c_int as libc::c_ushort as libc::c_int
	{
		(*id_scrolls.as_mut_ptr().offset((*obj).which_kind as isize))
			.id_status = 0o1 as libc::c_int as libc::c_ushort;
	}
	vanish(&mut *obj, (*obj).which_kind != SLEEP, &mut rogue.pack);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn vanish(obj: &mut obj, do_regular_move: bool, pack: &mut obj) {
	/* vanish() does NOT handle a quiver of weapons with more than one
	   arrow (or whatever) in the quiver.  It will only decrement the count.
	*/
	if (*obj).quantity > 1 {
		(*obj).quantity -= 1;
	} else {
		if (*obj).in_use_flags & BEING_WIELDED {
			unwield(obj);
		} else if (*obj).in_use_flags & BEING_WORN {
			unwear(obj);
		} else if (*obj).in_use_flags & ON_EITHER_HAND {
			un_put_on(obj);
		}
		take_from_pack(obj, pack);
		free_object(obj);
	}
	if do_regular_move {
		reg_move();
	}
}

unsafe fn potion_heal(extra: bool) {
	rogue.hp_current += rogue.exp;

	let ratio = rogue.hp_current as f32 / rogue.hp_max;
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
		let add = ratio * (rogue.hp_max - rogue.hp_current);
		rogue.hp_current += add;
		if rogue.hp_current > rogue.hp_max {
			rogue.hp_current = rogue.hp_max;
		}
	}
	if blind != 0 {
		unblind();
	}
	if confused != 0 && extra {
		unconfuse();
	} else if confused != 0 {
		confused = (confused / 2) + 1;
	}
	if halluc != 0 && extra {
		unhallucinate();
	} else if halluc {
		halluc = (halluc / 2) + 1;
	}
}

#[no_mangle]
pub unsafe extern "C" fn eat() {
	let mut ch: libc::c_short = 0;
	let mut moves: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut buf: [libc::c_char; 70] = [0; 70];
	ch = pack_letter("eat what?", Foods) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message("no such item.", 0 as libc::c_int);
		return;
	}
	if (*obj).what_is as libc::c_int
		!= 0o40 as libc::c_int as libc::c_ushort as libc::c_int
	{
		message("you can't eat that", 0 as libc::c_int);
		return;
	}
	if (*obj).which_kind as libc::c_int == 1 as libc::c_int
		|| rand_percent(60) != 0
	{
		moves = get_rand(900 as libc::c_int, 1100 as libc::c_int) as libc::c_short;
		if (*obj).which_kind as libc::c_int == 0 as libc::c_int {
			message("yum, that tasted good", 0 as libc::c_int);
		} else {
			let buf = format!("my, that was a yummy {}", &fruit());
			message(&buf, 0 as libc::c_int);
		}
	} else {
		moves = get_rand(700 as libc::c_int, 900 as libc::c_int) as libc::c_short;
		message("yuk, that food tasted awful", 0 as libc::c_int);
		add_exp(2 as libc::c_int, true);
	}
	rogue
		.moves_left = (rogue.moves_left as libc::c_int / 3 as libc::c_int)
		as libc::c_short;
	rogue
		.moves_left = (rogue.moves_left as libc::c_int + moves as libc::c_int)
		as libc::c_short;
	*hunger_str
		.as_mut_ptr()
		.offset(0 as libc::c_int as isize) = 0 as libc::c_int as libc::c_char;
	print_stats(0o100 as libc::c_int);
	vanish(&mut *obj, true, &mut rogue.pack);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn tele() {
	mvaddch(rogue.row as i32, rogue.col as i32, get_dungeon_char(rogue.row, rogue.col));

	if cur_room >= 0 {
		darken_room(cur_room);
	}
	put_player(get_room_number(rogue.row, rogue.col));
	being_held = false;
	bear_trap = 0;
}

pub unsafe fn hallucinate() {
	let mut obj: *mut object = 0 as *mut object;
	let mut monster: *mut object = 0 as *mut object;
	let mut ch: libc::c_short = 0;
	if blind != 0 {
		return;
	}
	obj = level_objects.next_object;
	while !obj.is_null() {
		ch = (if ncurses::wmove(ncurses::stdscr(), (*obj).row as libc::c_int, (*obj).col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int) as ncurses::chtype
		} else {
			ncurses::winch(ncurses::stdscr())
		}) as libc::c_short;
		if ((ch as libc::c_int) < 'A' as i32 || ch as libc::c_int > 'Z' as i32)
			&& ((*obj).row as libc::c_int != rogue.row as libc::c_int
			|| (*obj).col as libc::c_int != rogue.col as libc::c_int)
		{
			if ch as libc::c_int != ' ' as i32 && ch as libc::c_int != '.' as i32
				&& ch as libc::c_int != '#' as i32 && ch as libc::c_int != '+' as i32
			{
				addch(gr_obj_char() as ncurses::chtype);
			}
		}
		obj = (*obj).next_object;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		ch = (if ncurses::wmove(
			ncurses::stdscr(),
			(*monster).row as libc::c_int,
			(*monster).col as libc::c_int,
		) == -(1 as libc::c_int)
		{
			-(1 as libc::c_int) as ncurses::chtype
		} else {
			ncurses::winch(ncurses::stdscr())
		}) as libc::c_short;
		if ch as libc::c_int >= 'A' as i32 && ch as libc::c_int <= 'Z' as i32 {
			addch(get_rand('A' as i32, 'Z' as i32) as ncurses::chtype);
		}
		monster = (*monster).next_object;
	}
}

pub unsafe fn unhallucinate() {
	halluc = 0;
	relight();
	message("everything looks SO boring now", 1);
}

pub unsafe fn unblind()
{
	blind = 0;
	message("the veil of darkness lifts", 1);
	relight();
	if halluc {
		hallucinate();
	}
	if detect_monster {
		show_monsters();
	}
}

pub unsafe fn relight() {
	if cur_room == PASSAGE {
		light_passage(rogue.row, rogue.col);
	} else {
		light_up_room(cur_room);
	}
	mvaddch(rogue.row as i32, rogue.col as i32, chtype::from(rogue.fchar));
}

pub unsafe fn take_a_nap() {
	let mut i = get_rand(2, 5);
	md_sleep(1);
	while i > 0 {
		i -= 1;
		mv_mons();
	}
	md_sleep(1);
	message(YOU_CAN_MOVE_AGAIN, 0);
}

unsafe fn go_blind() {
	if blind == 0 {
		message("a cloak of darkness falls around you", 0);
	}
	blind += get_rand(500, 800);

	if detect_monster {
		let mut monster = level_monsters.next_monster();
		while !monster.is_null() {
			mvaddch((*monster).row as i32, (*monster).col as i32, (*monster).trail_char());
			monster = (*monster).next_monster();
		}
	}
	if cur_room >= 0 {
		for i in (rooms[cur_room].top_row as usize + 1)..rooms[cur_room].bottom_row as usize {
			for j in (rooms[cur_room].left_col as usize + 1)..rooms[cur_room].right_col as usize {
				mvaddch(i as i32, j as i32, chtype::from(' '));
			}
		}
	}
	mvaddch(rogue.row as i32, rogue.col as i32, chtype::from(rogue.fchar));
}

#[no_mangle]
pub unsafe extern "C" fn get_ench_color() -> *mut libc::c_char {
	if halluc != 0 {
		return ((*id_potions
			.as_mut_ptr()
			.offset(
				get_rand(0, 14 - 1) as isize,
			))
			.title)
			.as_mut_ptr();
	}
	return b"blue \0" as *const u8 as *const libc::c_char as *mut libc::c_char;
}

pub unsafe fn confuse() {
	confused += get_rand(12, 22);
}

pub unsafe fn unconfuse() {
	confused = 0;
	let msg = format!("you feel less {} now", if halluc > 0 { "trippy" } else { "confused" });
	message(&msg, 1);
}
