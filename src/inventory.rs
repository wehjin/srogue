#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{clrtoeol, mv, mvaddstr, mvinch, refresh};
use crate::message;
use crate::pack::wait_for_ack;
use crate::random::get_rand;

extern "C" {
	pub type ldat;

	fn strcat(_: *mut libc::c_char, _: *const libc::c_char) -> *mut libc::c_char;
	fn get_letter_object() -> *mut object;
	fn strncmp(
		_: *const libc::c_char,
		_: *const libc::c_char,
		_: libc::c_ulong,
	) -> i64;
}

use crate::prelude::*;
use crate::prelude::food_kind::RATION;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::{InventoryFilter};
use crate::prelude::object_what::ObjectWhat::{Amulet, Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::potion_kind::POTIONS;
use crate::prelude::ring_kind::{ADD_STRENGTH, DEXTERITY, RINGS};
use crate::prelude::scroll_kind::SCROLLS;
use crate::prelude::wand_kind::WANDS;


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
pub type attr_t = ncurses::chtype;


pub static mut is_wood: [bool; WANDS] = [false; WANDS];
pub static wand_materials: [&'static str; WAND_MATERIALS] = [
	"steel ",
	"bronze ",
	"gold ",
	"silver ",
	"copper ",
	"nickel ",
	"cobalt ",
	"tin ",
	"iron ",
	"magnesium ",
	"chrome ",
	"carbon ",
	"platinum ",
	"silicon ",
	"titanium ",
	"teak ",
	"oak ",
	"cherry ",
	"birch ",
	"pine ",
	"cedar ",
	"redwood ",
	"balsa ",
	"ivory ",
	"walnut ",
	"maple ",
	"mahogany ",
	"elm ",
	"palm ",
	"wooden ",
];

pub static gems: [&'static str; GEMS] = [
	"diamond ",
	"stibotantalite ",
	"lapi-lazuli ",
	"ruby ",
	"emerald ",
	"sapphire ",
	"amethyst ",
	"quartz ",
	"tiger-eye ",
	"opal ",
	"agate ",
	"turquoise ",
	"pearl ",
	"garnet ",
];
const SYLLABLES: [&'static str; MAXSYLLABLES] = [
	"blech ",
	"foo ",
	"barf ",
	"rech ",
	"bar ",
	"blech ",
	"quo ",
	"bloto ",
	"woh ",
	"caca ",
	"blorp ",
	"erp ",
	"festr ",
	"rot ",
	"slie ",
	"snorf ",
	"iky ",
	"yuky ",
	"ooze ",
	"ah ",
	"bahl ",
	"zep ",
	"druhl ",
	"flem ",
	"behil ",
	"arek ",
	"mep ",
	"zihr ",
	"grit ",
	"kona ",
	"kini ",
	"ichi ",
	"niah ",
	"ogr ",
	"ooh ",
	"ighr ",
	"coph ",
	"swerr ",
	"mihln ",
	"poxi ",
];

fn random_syllable() -> &'static str {
	SYLLABLES[get_rand(1, MAXSYLLABLES - 1)]
}

pub unsafe fn inventory(pack: &object, filter: InventoryFilter) {
	let mut obj = pack.next_object;
	if obj.is_null() {
		message("your pack is empty", 0);
		return;
	}
	let item_lines = {
		let mut item_lines = Vec::new();
		while !obj.is_null() {
			{
				let obj = &*obj;
				let what = obj.what_is.what_is();
				if filter.includes(what) {
					let close_char = if what == Armor && obj.is_protected != 0 { '}' } else { ')' };
					let line = format!(" {}{} {}", obj.m_char() as u8 as char, close_char, get_desc(obj));
					item_lines.push(line);
				}
			}
			obj = (*obj).next_object;
		}
		{
			let prompt_line = " --press space to continue--";
			item_lines.push(prompt_line.to_string());
		}
		item_lines
	};
	let item_lines_max_len = item_lines.iter().map(|it| it.chars().count()).max().expect("max length");

	let mut old_lines = Vec::new();
	let start_col = DCOLS - (item_lines_max_len + 2);
	let max_row = item_lines.len().min(DROWS);
	for row in 0..max_row {
		if row > 0 {
			let mut old_line: Vec<u8> = Vec::new();
			for col in start_col..DCOLS {
				let ch = mvinch(row as i32, col as i32);
				old_line.push(ch as u8)
			}
			old_lines.push(String::from_utf8(old_line).expect("utf8"));
		}
		mvaddstr(row as i32, start_col as i32, &item_lines[row]);
		clrtoeol();
	}
	refresh();
	wait_for_ack();

	mv(0, 0);
	clrtoeol();
	for row in 1..max_row {
		mvaddstr(row as i32, start_col as i32, &old_lines[row - 1]);
	}
}

pub unsafe fn mix_colors() {
	for _ in 0..=32 {
		let j = get_rand(0, POTIONS - 1);
		let k = get_rand(0, POTIONS - 1);
		let t = id_potions[j].title.to_string();
		id_potions[j].title = id_potions[k].title.to_string();
		id_potions[k].title = t;
	}
}

pub unsafe fn make_scroll_titles() {
	for i in 0..SCROLLS {
		let syllables = (0..get_rand(2, 5)).map(|_| random_syllable().to_string()).collect::<Vec<_>>();
		id_scrolls[i].title = format!("'{}' ", syllables.join(""));
	}
}

fn get_quantity(obj: &object) -> String {
	match &obj.what_is {
		WhatIsOrDisguise::WhatIs(what) => {
			match what {
				Armor => "".to_string(),
				_ => {
					if obj.quantity == 1 {
						"a ".to_string()
					} else {
						format!("{} ", obj.quantity)
					}
				}
			}
		}
		WhatIsOrDisguise::Disguise(_) => "".to_string()
	}
}

unsafe fn get_title(obj: &object) -> &String {
	let id_table = get_id_table(obj);
	&id_table[obj.which_kind as usize].title
}

unsafe fn get_id_status(obj: &object) -> IdStatus {
	get_id_table(obj)[obj.which_kind as usize].id_status
}

unsafe fn get_id_real(obj: &object) -> &String {
	&get_id_table(obj)[obj.which_kind as usize].real
}

unsafe fn get_identified(obj: &object) -> String {
	match obj.what_is.what_is() {
		Scroll | Potion => format!("{}{}{}", get_quantity(obj), name_of(obj), get_id_real(obj)),
		Ring => {
			let more_info = if (wizard || obj.identified) && (obj.which_kind == DEXTERITY || obj.which_kind == ADD_STRENGTH) {
				format!("{}{} ", if obj.class > 0 { "+" } else { "" }, obj.class)
			} else {
				"".to_string()
			};
			format!("{}{}{}{}", get_quantity(obj), more_info, name_of(obj), get_id_real(obj))
		}
		Wand => {
			let more_info = if wizard || obj.identified { format!("[{}]", obj.class) } else { "".to_string() };
			format!("{}{}{}{}", get_quantity(obj), name_of(obj), get_id_real(obj), more_info)
		}
		Armor => format!("{}{} {}[{}]", if obj.d_enchant >= 0 { "+" } else { "" }, obj.d_enchant, get_title(obj), get_armor_class(obj)),
		Weapon => format!("{}{}{},{}{} {}",
		                  get_quantity(obj),
		                  if obj.hit_enchant >= 0 { "+" } else { "" }, obj.hit_enchant,
		                  if obj.d_enchant >= 0 { "+" } else { "" }, obj.d_enchant,
		                  name_of(obj)
		),
		_ => panic!("invalid identified object")
	}
}

unsafe fn get_called(obj: &object) -> String {
	match obj.what_is.what_is() {
		Scroll | Potion | Wand | Ring => format!("{}{}called {}", get_quantity(obj), name_of(obj), get_title(obj)),
		_ => panic!("invalid called object"),
	}
}

unsafe fn get_unidentified(obj: &object) -> String {
	let what = obj.what_is.what_is();
	match what {
		Scroll => format!("{}{}entitled: {}", get_quantity(obj), name_of(obj), get_title(obj)),
		Potion => format!("{}{}{}", get_quantity(obj), get_title(obj), name_of(obj)),
		Wand | Ring => if obj.identified || get_id_status(obj) == IdStatus::Identified {
			get_identified(obj)
		} else if get_id_status(obj) == IdStatus::Called {
			get_called(obj)
		} else {
			format!("{}{}{}", get_quantity(obj), get_title(obj), name_of(obj))
		},
		Armor => if obj.identified {
			get_identified(obj)
		} else {
			get_title(obj).to_string()
		},
		Weapon => if obj.identified {
			get_identified(obj)
		} else {
			name_of(obj)
		},
		_ => panic!("invalid unidentified object")
	}
}

pub unsafe fn get_desc(obj: &object) -> String {
	let what_is = obj.what_is.what_is();
	if what_is == Amulet {
		return "the amulet of Yendor ".to_string();
	}
	if what_is == Gold {
		return format!("{} pieces of gold", obj.quantity);
	}

	let desc = if what_is == Food {
		let quantity = if obj.which_kind == RATION {
			if obj.quantity > 1 {
				format!("{} rations of ", obj.quantity)
			} else {
				"some ".to_string()
			}
		} else {
			"a ".to_string()
		};
		format!("{}{}", quantity, name_of(obj))
	} else {
		if wizard {
			get_identified(obj)
		} else {
			match what_is {
				Weapon | Armor | Wand | Ring => get_unidentified(obj),
				_ => match get_id_status(obj) {
					IdStatus::Unidentified => get_unidentified(obj),
					IdStatus::Identified => get_identified(obj),
					IdStatus::Called => get_called(obj),
				}
			}
		}
	};
	let desc = if desc.starts_with("a ") && is_vowel(desc.chars().nth(2).expect("char at 2")) {
		format!("an {}", &desc[2..])
	} else {
		desc
	};
	format!("{}{}", desc, get_in_use_description(obj))
}

fn get_in_use_description(obj: &object) -> &'static str {
	if obj.in_use_flags & BEING_WIELDED != 0 {
		"in hand"
	} else if obj.in_use_flags & BEING_WORN != 0 {
		"being worn"
	} else if obj.in_use_flags & ON_LEFT_HAND != 0 {
		"on left hand"
	} else if obj.in_use_flags & ON_RIGHT_HAND != 0 {
		"on right hand"
	} else {
		""
	}
}


pub unsafe fn get_wand_and_ring_materials() {
	{
		let mut used = [false; WAND_MATERIALS];
		for i in 0..WANDS {
			let j = take_unused(&mut used);
			id_wands[i].title = wand_materials[j].to_string();
			is_wood[i] = j > MAX_METAL;
		}
	}
	{
		let mut used = [false; GEMS];
		for i in 0..RINGS {
			let j = take_unused(&mut used);
			id_rings[i].title = gems[j].to_string();
		}
	}
}

fn take_unused<const N: usize>(used: &mut [bool; N]) -> usize {
	let mut j = 0;
	loop {
		j = get_rand(0, used.len() - 1);
		if !used[j] {
			break;
		}
	}
	used[j] = true;
	j
}

pub unsafe fn single_inv(mut ichar: libc::c_short) {
	let mut ch: libc::c_short = 0;
	let mut desc: [libc::c_char; 80] = [0; 80];
	let mut obj: *mut object = 0 as *mut object;
	ch = (if ichar as i64 != 0 {
		ichar as i64
	} else {
		pack_letter(
			"inventory what?",
			0o777 as i64 as libc::c_ushort as i64,
		)
	}) as libc::c_short;
	if ch as i64 == '\u{1b}' as i32 {
		return;
	}
	obj = get_letter_object(ch as i64);
	if obj.is_null() {
		message("no such item.", 0);
		return;
	}
	desc[0 as i64 as usize] = ch as libc::c_char;
	desc[1
		as usize] = (if (*obj).what_is as i64
		& 0o1 as libc::c_ushort as i64 != 0
		&& (*obj).is_protected as i64 != 0
	{
		'}' as i32
	} else {
		')' as i32
	}) as libc::c_char;
	desc[2 as libc::c_int as usize] = ' ' as i32 as libc::c_char;
	desc[3 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
	get_desc(obj, desc.as_mut_ptr().offset(3 as libc::c_int as isize));
	message(desc.as_mut_ptr(), 0 as libc::c_int);
	panic!("Reached end of non-void function without returning");
}

pub unsafe fn get_id_table(obj: &object) -> Vec<&'static id> {
	match obj.what_is {
		WhatIsOrDisguise::WhatIs(what_is) => {
			match what_is {
				Scroll => &id_scrolls,
				Potion => &id_potions,
				Wand => &id_wands,
				Ring => &id_rings,
				Weapon => &id_weapons,
				Armor => &id_armors,
				_ => &[],
			}
		}
		WhatIsOrDisguise::Disguise(_) => &[],
	}.iter().map(|it| it).collect()
}

#[no_mangle]
pub unsafe extern "C" fn inv_armor_weapon(mut is_weapon: libc::c_char) -> libc::c_int {
	if is_weapon != 0 {
		if !(rogue.weapon).is_null() {
			single_inv((*rogue.weapon).ichar as libc::c_int);
		} else {
			message(
				b"not wielding anything\0" as *const u8 as *const libc::c_char,
				0 as libc::c_int,
			);
		}
	} else if !(rogue.armor).is_null() {
		single_inv((*rogue.armor).ichar as libc::c_int);
	} else {
		message(
			b"not wearing anything\0" as *const u8 as *const libc::c_char,
			0 as libc::c_int,
		);
	}
	panic!("Reached end of non-void function without returning");
}
