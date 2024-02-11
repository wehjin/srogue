#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use ncurses::{clrtoeol, mv, mvaddstr, mvinch, refresh};
use crate::level::constants::{DCOLS, DROWS};
use crate::message;
use crate::pack::wait_for_ack;
use crate::player::Player;
use crate::random::get_rand;

use crate::prelude::*;
use crate::prelude::armor_kind::ArmorKind;
use crate::prelude::food_kind::RATION;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::{PackFilter};
use crate::prelude::object_what::ObjectWhat::{Amulet, Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::prelude::potion_kind::{PotionKind, POTIONS};
use crate::prelude::ring_kind::{ADD_STRENGTH, DEXTERITY, RingKind, RINGS};
use crate::prelude::scroll_kind::{ScrollKind, SCROLLS};
use crate::prelude::wand_kind::{WandKind, MAX_WAND};
use crate::prelude::weapon_kind::WeaponKind;

pub static mut IS_WOOD: [bool; MAX_WAND] = [false; MAX_WAND];
const WAND_MATERIALS: [&'static str; MAX_WAND_MATERIAL] = [
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

const GEMS: [&'static str; MAX_GEM] = [
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
const SYLLABLES: [&'static str; MAX_SYLLABLE] = [
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
	SYLLABLES[get_rand(1, MAX_SYLLABLE - 1)]
}

pub unsafe fn inventory(pack: &ObjectPack, filter: PackFilter) {
	if pack.is_empty() {
		message("your pack is empty", 0);
		return;
	}
	let item_lines = {
		let mut item_lines = Vec::new();
		for obj in pack.objects() {
			let what = obj.what_is;
			if filter.includes(what) {
				let close_char = if what == Armor && obj.is_protected != 0 { '}' } else { ')' };
				let line = format!(" {}{} {}", obj.ichar, close_char, get_obj_desc(obj));
				item_lines.push(line);
			}
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
	for i in 0..POTIONS {
		if id_potions[i].title.is_none() {
			id_potions[i].title = Some(PotionKind::from_index(i).to_index().to_string());
		}
	}
	for _ in 0..=32 {
		let j = get_rand(0, POTIONS - 1);
		let k = get_rand(0, POTIONS - 1);
		let t = id_potions[j].title.clone();
		id_potions[j].title = id_potions[k].title.clone();
		id_potions[k].title = t;
	}
}

pub unsafe fn make_scroll_titles() {
	for i in 0..SCROLLS {
		let syllables = (0..get_rand(2, 5)).map(|_| random_syllable().to_string()).collect::<Vec<_>>();
		let title = format!("'{}' ", syllables.join("")).trim().to_string();
		id_scrolls[i].title = Some(title);
	}
}

fn get_quantity(obj: &object) -> String {
	match obj.what_is {
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

pub unsafe fn get_title(obj: &object) -> &str {
	let id_table = get_id_table(obj);
	let id = &id_table[obj.which_kind as usize];
	if let Some(title) = &id.title {
		title
	} else {
		match obj.what_is {
			Armor => ArmorKind::from_index(obj.which_kind as usize).title(),
			Weapon => WeaponKind::from(obj.which_kind).title(),
			Potion => PotionKind::from_index(obj.which_kind as usize).title(),
			_ => "",
		}
	}
}

unsafe fn get_id_status(obj: &object) -> IdStatus {
	get_id_table(obj)[obj.which_kind as usize].id_status
}

fn get_id_real(obj: &object) -> &'static str {
	match obj.what_is {
		Scroll => ScrollKind::from_index(obj.which_kind as usize).real_name(),
		Potion => PotionKind::from_index(obj.which_kind as usize).real_name(),
		Wand => WandKind::from_index(obj.which_kind as usize).real_name(),
		Ring => RingKind::from_index(obj.which_kind as usize).real_name(),
		_ => "",
	}
}

unsafe fn get_identified(obj: &object) -> String {
	match obj.what_is {
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
		Armor => format!("{}{} {}[{}]", if obj.d_enchant >= 0 { "+" } else { "" }, obj.d_enchant, get_title(obj), get_armor_class(Some(obj))),
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
	match obj.what_is {
		Scroll | Potion | Wand | Ring => format!("{}{}called {}", get_quantity(obj), name_of(obj), get_title(obj)),
		_ => panic!("invalid called object"),
	}
}

unsafe fn get_unidentified(obj: &object) -> String {
	let what = obj.what_is;
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

pub unsafe fn get_inv_obj_desc(obj: &obj) -> String {
	let obj_desc = get_obj_desc(&obj);
	format!("{}({})", obj_desc, obj.ichar)
}

pub unsafe fn get_obj_desc(obj: &object) -> String {
	let what_is = obj.what_is;
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
		let mut used = [false; MAX_WAND_MATERIAL];
		for i in 0..MAX_WAND {
			let j = take_unused(&mut used);
			id_wands[i].title = Some(WAND_MATERIALS[j].to_string());
			IS_WOOD[i] = j > MAX_METAL;
		}
	}
	{
		let mut used = [false; MAX_GEM];
		for i in 0..RINGS {
			let j = take_unused(&mut used);
			id_rings[i].title = Some(GEMS[j].to_string());
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

pub unsafe fn single_inv(ichar: Option<char>, player: &mut Player) {
	let ch = if let Some(ichar) = ichar {
		ichar
	} else {
		pack_letter("inventory what?", AllObjects, player)
	};
	if ch == CANCEL {
		return;
	}
	if let Some(obj) = player.object_with_letter(ch) {
		let separator = if obj.what_is == Armor && obj.is_protected != 0 { '}' } else { ')' };
		let msg = format!("{}{} {}", ch, separator, get_obj_desc(obj));
		message(&msg, 0);
	} else {
		message("no such item.", 0);
	}
}

pub unsafe fn get_id_table(obj: &object) -> &'static mut [id] {
	match obj.what_is {
		Scroll => &mut id_scrolls[..],
		Potion => &mut id_potions[..],
		Wand => &mut id_wands[..],
		Ring => &mut id_rings[..],
		Weapon => &mut id_weapons[..],
		Armor => &mut id_armors[..],
		_ => panic!("no id table"),
	}
}

pub unsafe fn inv_armor_weapon(is_weapon: bool, player: &mut Player) {
	if is_weapon {
		if let Some(weapon) = player.weapon() {
			single_inv(Some(weapon.ichar), player);
		} else {
			message("not wielding anything", 0);
		}
	} else {
		if let Some(armor) = player.armor() {
			single_inv(Some(armor.ichar), player);
		} else {
			message("not wearing anything", 0);
		}
	}
}
