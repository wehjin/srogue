#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

extern "C" {
	pub type ldat;
	fn waddch(_: *mut WINDOW, _: chtype) -> libc::c_int;
	fn winch(_: *mut WINDOW) -> chtype;
	fn wmove(_: *mut WINDOW, _: libc::c_int, _: libc::c_int) -> libc::c_int;
	static mut stdscr: *mut WINDOW;
	fn sprintf(_: *mut libc::c_char, _: *const libc::c_char, _: ...) -> libc::c_int;
	static mut rogue: fighter;
	static mut rooms: [room; 0];
	static mut dungeon: [[libc::c_ushort; 80]; 24];
	static mut level_objects: object;
	static mut id_scrolls: [id; 0];
	static mut id_potions: [id; 0];
	static mut level_monsters: object;
	fn name_of() -> *mut libc::c_char;
	fn reg_move() -> libc::c_char;
	fn get_letter_object() -> *mut object;
	fn object_at() -> *mut object;
	fn get_id_table() -> *mut id;
	static mut bear_trap: libc::c_short;
	static mut hunger_str: [libc::c_char; 0];
	static mut cur_room: libc::c_short;
	static mut level_points: [libc::c_long; 0];
	static mut being_held: libc::c_char;
	static mut fruit: *mut libc::c_char;
	static mut you_can_move_again: *mut libc::c_char;
	static mut sustain_strength: libc::c_char;
}

use crate::prelude::*;

pub type chtype = libc::c_uint;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _win_st {
	pub _cury: libc::c_short,
	pub _curx: libc::c_short,
	pub _maxy: libc::c_short,
	pub _maxx: libc::c_short,
	pub _begy: libc::c_short,
	pub _begx: libc::c_short,
	pub _flags: libc::c_short,
	pub _attrs: attr_t,
	pub _bkgd: chtype,
	pub _notimeout: libc::c_int,
	pub _clear: libc::c_int,
	pub _leaveok: libc::c_int,
	pub _scroll: libc::c_int,
	pub _idlok: libc::c_int,
	pub _idcok: libc::c_int,
	pub _immed: libc::c_int,
	pub _sync: libc::c_int,
	pub _use_keypad: libc::c_int,
	pub _delay: libc::c_int,
	pub _line: *mut ldat,
	pub _regtop: libc::c_short,
	pub _regbottom: libc::c_short,
	pub _parx: libc::c_int,
	pub _pary: libc::c_int,
	pub _parent: *mut WINDOW,
	pub _pad: pdat,
	pub _yoffset: libc::c_short,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pdat {
	pub _pad_y: libc::c_short,
	pub _pad_x: libc::c_short,
	pub _pad_top: libc::c_short,
	pub _pad_left: libc::c_short,
	pub _pad_bottom: libc::c_short,
	pub _pad_right: libc::c_short,
}

pub type WINDOW = _win_st;
pub type attr_t = chtype;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct id {
	pub value: libc::c_short,
	pub title: [libc::c_char; 128],
	pub real: [libc::c_char; 128],
	pub id_status: libc::c_ushort,
}


#[derive(Copy, Clone)]
#[repr(C)]
pub struct fight {
	pub armor: *mut object,
	pub weapon: *mut object,
	pub left_ring: *mut object,
	pub right_ring: *mut object,
	pub hp_current: libc::c_short,
	pub hp_max: libc::c_short,
	pub str_current: libc::c_short,
	pub str_max: libc::c_short,
	pub pack: object,
	pub gold: libc::c_long,
	pub exp: libc::c_short,
	pub exp_points: libc::c_long,
	pub row: libc::c_short,
	pub col: libc::c_short,
	pub fchar: libc::c_short,
	pub moves_left: libc::c_short,
}

pub type fighter = fight;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct dr {
	pub oth_room: libc::c_short,
	pub oth_row: libc::c_short,
	pub oth_col: libc::c_short,
	pub door_row: libc::c_short,
	pub door_col: libc::c_short,
}

pub type door = dr;


#[no_mangle]
pub static mut halluc: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut blind: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut confused: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut levitate: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut haste_self: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut see_invisible: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut extra_hp: libc::c_short = 0 as libc::c_int as libc::c_short;
#[no_mangle]
pub static mut detect_monster: libc::c_char = 0 as libc::c_int as libc::c_char;
#[no_mangle]
pub static mut strange_feeling: *mut libc::c_char = b"you have a strange feeling for a moment, then it passes\0"
	as *const u8 as *const libc::c_char as *mut libc::c_char;

#[no_mangle]
pub unsafe extern "C" fn quaff() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut buf: [libc::c_char; 80] = [0; 80];
	let mut obj: *mut object = 0 as *mut object;
	ch = pack_letter(
		b"quaff what?\0" as *const u8 as *const libc::c_char,
		0o10 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).what_is as libc::c_int
		!= 0o10 as libc::c_int as libc::c_ushort as libc::c_int
	{
		message(
			b"you can't drink that\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	match (*obj).which_kind as libc::c_int {
		0 => {
			message(
				b"you feel stronger now, what bulging muscles!\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
			rogue.str_current += 1;
			rogue.str_current;
			if rogue.str_current as libc::c_int > rogue.str_max as libc::c_int {
				rogue.str_max = rogue.str_current;
			}
		}
		1 => {
			rogue.str_current = rogue.str_max;
			message(
				b"this tastes great, you feel warm all over\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
		}
		2 => {
			message(
				b"you begin to feel better\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			potion_heal(0 as libc::c_int);
		}
		3 => {
			message(
				b"you begin to feel much better\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			potion_heal(1 as libc::c_int);
		}
		4 => {
			if sustain_strength == 0 {
				rogue
					.str_current = (rogue.str_current as libc::c_int
					- get_rand(1 as libc::c_int, 3 as libc::c_int)) as libc::c_short;
				if (rogue.str_current as libc::c_int) < 1 as libc::c_int {
					rogue.str_current = 1 as libc::c_int as libc::c_short;
				}
			}
			message(
				b"you feel very sick now\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			if halluc != 0 {
				unhallucinate();
			}
		}
		5 => {
			rogue
				.exp_points = *level_points
				.as_mut_ptr()
				.offset((rogue.exp as libc::c_int - 1 as libc::c_int) as isize);
			add_exp(1 as libc::c_int, 1 as libc::c_int);
		}
		6 => {
			go_blind();
		}
		7 => {
			message(
				b"oh wow, everything seems so cosmic\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
			halluc = (halluc as libc::c_int
				+ get_rand(500 as libc::c_int, 800 as libc::c_int)) as libc::c_short;
		}
		8 => {
			show_monsters();
			if (level_monsters.next_object).is_null() {
				message(strange_feeling, 0 as libc::c_int);
			}
		}
		9 => {
			if !(level_objects.next_object).is_null() {
				if blind == 0 {
					show_objects();
				}
			} else {
				message(strange_feeling, 0 as libc::c_int);
			}
		}
		10 => {
			message(
				if halluc as libc::c_int != 0 {
					b"what a trippy feeling\0" as *const u8 as *const libc::c_char
				} else {
					b"you feel confused\0" as *const u8 as *const libc::c_char
				},
				0 as libc::c_int,
			);
			confuse();
		}
		11 => {
			message(
				b"you start to float in the air\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			levitate = (levitate as libc::c_int
				+ get_rand(15 as libc::c_int, 30 as libc::c_int)) as libc::c_short;
			bear_trap = 0 as libc::c_int as libc::c_short;
			being_held = bear_trap as libc::c_char;
		}
		12 => {
			message(
				b"you feel yourself moving much faster\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
			haste_self = (haste_self as libc::c_int
				+ get_rand(11 as libc::c_int, 21 as libc::c_int)) as libc::c_short;
			if haste_self as libc::c_int % 2 as libc::c_int == 0 {
				haste_self += 1;
				haste_self;
			}
		}
		13 => {
			sprintf(
				buf.as_mut_ptr(),
				b"hmm, this potion tastes like %sjuice\0" as *const u8
					as *const libc::c_char,
				fruit,
			);
			message(buf.as_mut_ptr(), 0 as libc::c_int);
			if blind != 0 {
				unblind();
			}
			see_invisible = 1 as libc::c_int as libc::c_char;
			relight();
		}
		_ => {}
	}
	print_stats(0o10 as libc::c_int | 0o4 as libc::c_int);
	if (*id_potions.as_mut_ptr().offset((*obj).which_kind as isize)).id_status
		as libc::c_int != 0o2 as libc::c_int as libc::c_ushort as libc::c_int
	{
		(*id_potions.as_mut_ptr().offset((*obj).which_kind as isize))
			.id_status = 0o1 as libc::c_int as libc::c_ushort;
	}
	vanish(obj, 1 as libc::c_int, &mut rogue.pack);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn read_scroll() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut msg: [libc::c_char; 80] = [0; 80];
	if blind != 0 {
		message(
			b"You can't see to read the scroll.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	ch = pack_letter(
		b"read what?\0" as *const u8 as *const libc::c_char,
		0o4 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).what_is as libc::c_int
		!= 0o4 as libc::c_int as libc::c_ushort as libc::c_int
	{
		message(
			b"you can't read that\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	match (*obj).which_kind as libc::c_int {
		7 => {
			message(
				b"you hear a maniacal laughter in the distance\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
		}
		1 => {
			hold_monster();
		}
		2 => {
			if !(rogue.weapon).is_null() {
				if (*rogue.weapon).what_is as libc::c_int
					== 0o2 as libc::c_int as libc::c_ushort as libc::c_int
				{
					sprintf(
						msg.as_mut_ptr(),
						b"your %sglow%s %sfor a moment\0" as *const u8
							as *const libc::c_char,
						name_of(rogue.weapon),
						if (*rogue.weapon).quantity as libc::c_int <= 1 as libc::c_int {
							b"s\0" as *const u8 as *const libc::c_char
						} else {
							b"\0" as *const u8 as *const libc::c_char
						},
						get_ench_color(),
					);
					message(msg.as_mut_ptr(), 0 as libc::c_int);
					if coin_toss() != 0 {
						(*rogue.weapon).hit_enchant += 1;
						(*rogue.weapon).hit_enchant;
					} else {
						(*rogue.weapon).d_enchant += 1;
						(*rogue.weapon).d_enchant;
					}
				}
				(*rogue.weapon).is_cursed = 0 as libc::c_int as libc::c_short;
			} else {
				message(
					b"your hands tingle\0" as *const u8 as *const libc::c_char,
					0 as libc::c_int,
				);
			}
		}
		3 => {
			if !(rogue.armor).is_null() {
				sprintf(
					msg.as_mut_ptr(),
					b"your armor glows %sfor a moment\0" as *const u8
						as *const libc::c_char,
					get_ench_color(),
				);
				message(msg.as_mut_ptr(), 0 as libc::c_int);
				(*rogue.armor).d_enchant += 1;
				(*rogue.armor).d_enchant;
				(*rogue.armor).is_cursed = 0 as libc::c_int as libc::c_short;
				print_stats(0o20 as libc::c_int);
			} else {
				message(
					b"your skin crawls\0" as *const u8 as *const libc::c_char,
					0 as libc::c_int,
				);
			}
		}
		4 => {
			message(
				b"this is a scroll of identify\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			(*obj).identified = 1 as libc::c_int as libc::c_short;
			(*id_scrolls.as_mut_ptr().offset((*obj).which_kind as isize))
				.id_status = 0o1 as libc::c_int as libc::c_ushort;
			idntfy();
		}
		5 => {
			tele();
		}
		6 => {
			message(
				b"you fall asleep\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
			take_a_nap();
		}
		0 => {
			if !(rogue.armor).is_null() {
				message(
					b"your armor is covered by a shimmering gold shield\0" as *const u8
						as *const libc::c_char,
					0 as libc::c_int,
				);
				(*rogue.armor).is_protected = 1 as libc::c_int as libc::c_short;
				(*rogue.armor).is_cursed = 0 as libc::c_int as libc::c_short;
			} else {
				message(
					b"your acne seems to have disappeared\0" as *const u8
						as *const libc::c_char,
					0 as libc::c_int,
				);
			}
		}
		8 => {
			message(
				if halluc == 0 {
					b"you feel as though someone is watching over you\0" as *const u8
						as *const libc::c_char
				} else {
					b"you feel in touch with the universal oneness\0" as *const u8
						as *const libc::c_char
				},
				0 as libc::c_int,
			);
			uncurse_all();
		}
		9 => {
			create_monster();
		}
		10 => {
			aggravate();
		}
		11 => {
			message(
				b"this scroll seems to have a map on it\0" as *const u8
					as *const libc::c_char,
				0 as libc::c_int,
			);
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
	vanish(
		obj,
		((*obj).which_kind as libc::c_int != 6 as libc::c_int) as libc::c_int,
		&mut rogue.pack,
	);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn eat() -> libc::c_int {
	let mut ch: libc::c_short = 0;
	let mut moves: libc::c_short = 0;
	let mut obj: *mut object = 0 as *mut object;
	let mut buf: [libc::c_char; 70] = [0; 70];
	ch = pack_letter(
		b"eat what?\0" as *const u8 as *const libc::c_char,
		0o40 as libc::c_int as libc::c_ushort as libc::c_int,
	) as libc::c_short;
	if ch as libc::c_int == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as libc::c_int);
	if obj.is_null() {
		message(
			b"no such item.\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).what_is as libc::c_int
		!= 0o40 as libc::c_int as libc::c_ushort as libc::c_int
	{
		message(
			b"you can't eat that\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		return;
	}
	if (*obj).which_kind as libc::c_int == 1 as libc::c_int
		|| rand_percent(60 as libc::c_int) != 0
	{
		moves = get_rand(900 as libc::c_int, 1100 as libc::c_int) as libc::c_short;
		if (*obj).which_kind as libc::c_int == 0 as libc::c_int {
			message(
				b"yum, that tasted good\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
		} else {
			sprintf(
				buf.as_mut_ptr(),
				b"my, that was a yummy %s\0" as *const u8 as *const libc::c_char,
				fruit,
			);
			message(buf.as_mut_ptr(), 0 as libc::c_int);
		}
	} else {
		moves = get_rand(700 as libc::c_int, 900 as libc::c_int) as libc::c_short;
		message(
			b"yuk, that food tasted awful\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
		add_exp(2 as libc::c_int, 1 as libc::c_int);
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
	vanish(obj, 1 as libc::c_int, &mut rogue.pack);
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn hallucinate() -> libc::c_int {
	let mut obj: *mut object = 0 as *mut object;
	let mut monster: *mut object = 0 as *mut object;
	let mut ch: libc::c_short = 0;
	if blind != 0 {
		return;
	}
	obj = level_objects.next_object;
	while !obj.is_null() {
		ch = (if wmove(stdscr, (*obj).row as libc::c_int, (*obj).col as libc::c_int)
			== -(1 as libc::c_int)
		{
			-(1 as libc::c_int) as chtype
		} else {
			winch(stdscr)
		}) as libc::c_short;
		if ((ch as libc::c_int) < 'A' as i32 || ch as libc::c_int > 'Z' as i32)
			&& ((*obj).row as libc::c_int != rogue.row as libc::c_int
			|| (*obj).col as libc::c_int != rogue.col as libc::c_int)
		{
			if ch as libc::c_int != ' ' as i32 && ch as libc::c_int != '.' as i32
				&& ch as libc::c_int != '#' as i32 && ch as libc::c_int != '+' as i32
			{
				waddch(stdscr, gr_obj_char() as chtype);
			}
		}
		obj = (*obj).next_object;
	}
	monster = level_monsters.next_object;
	while !monster.is_null() {
		ch = (if wmove(
			stdscr,
			(*monster).row as libc::c_int,
			(*monster).col as libc::c_int,
		) == -(1 as libc::c_int)
		{
			-(1 as libc::c_int) as chtype
		} else {
			winch(stdscr)
		}) as libc::c_short;
		if ch as libc::c_int >= 'A' as i32 && ch as libc::c_int <= 'Z' as i32 {
			waddch(stdscr, get_rand('A' as i32, 'Z' as i32) as chtype);
		}
		monster = (*monster).next_object;
	}
	panic!("Reached end of non-void function without returning");
}

#[no_mangle]
pub unsafe extern "C" fn get_ench_color() -> *mut libc::c_char {
	if halluc != 0 {
		return ((*id_potions
			.as_mut_ptr()
			.offset(
				get_rand(0 as libc::c_int, 14 as libc::c_int - 1 as libc::c_int) as isize,
			))
			.title)
			.as_mut_ptr();
	}
	return b"blue \0" as *const u8 as *const libc::c_char as *mut libc::c_char;
}
