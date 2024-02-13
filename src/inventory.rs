#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]

use ncurses::{clrtoeol, mv, mvaddstr, mvinch, refresh};
use crate::level::constants::{DCOLS, DROWS};
use crate::pack::{pack_letter, wait_for_ack};
use crate::player::Player;
use crate::random::get_rand;

use crate::message::{CANCEL, message};
use crate::objects::{get_armor_class, NoteStatus, name_of, obj, object, ObjectId, Title};
use crate::objects::note_tables::NoteTables;
use crate::potions::kind::PotionKind;
use crate::prelude::food_kind::RATION;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::PackFilter;
use crate::prelude::object_what::ObjectWhat::{Amulet, Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::ring::ring_kind::RingKind;
use crate::ring::constants::{ADD_STRENGTH, DEXTERITY};
use crate::score::is_vowel;
use crate::scrolls::ScrollKind;
use crate::zap::wand_kind::WandKind;
use crate::zap::constants::WANDS;
use crate::zap::wizard;

pub static mut IS_WOOD: [bool; WANDS] = [false; WANDS];

pub unsafe fn inventory(filter: PackFilter, player: &Player) {
	if player.pack().is_empty() {
		message("your pack is empty", 0);
		return;
	}
	let item_lines = {
		let mut item_lines = Vec::new();
		for obj in player.pack().objects() {
			let what = obj.what_is;
			if filter.includes(what) {
				let close_char = if what == Armor && obj.is_protected != 0 { '}' } else { ')' };
				let obj_ichar = obj.ichar;
				let obj_desc = get_obj_desc(obj, player.settings.fruit.to_string(), &player.notes);
				let line = format!(" {}{} {}", obj_ichar, close_char, obj_desc);
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

impl obj {
	pub fn title(&self, id_tables: &NoteTables) -> Title {
		id_tables.title(self.what_is, self.what_is as usize)
	}
}

unsafe fn get_id_status(obj: &object, id_tables: &NoteTables) -> NoteStatus {
	id_tables.note(obj.what_is, obj.which_kind as usize).status
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

unsafe fn get_identified(obj: &object, fruit: String, id_tables: &NoteTables) -> String {
	match obj.what_is {
		Scroll | Potion => {
			let quantity = get_quantity(obj);
			let name = name_of(obj, fruit, id_tables);
			let real_name = get_id_real(obj);
			format!("{}{}{}", quantity, name, real_name)
		}
		Ring => {
			let more_info = if (wizard || obj.identified) && (obj.which_kind == DEXTERITY || obj.which_kind == ADD_STRENGTH) {
				format!("{}{} ", if obj.class > 0 { "+" } else { "" }, obj.class)
			} else {
				"".to_string()
			};
			let name = name_of(obj, fruit, id_tables);
			let real_name = get_id_real(obj);
			format!("{}{}{}{}", get_quantity(obj), more_info, name, real_name)
		}
		Wand => format!("{}{}{}{}",
		                get_quantity(obj),
		                name_of(obj, fruit, id_tables),
		                get_id_real(obj),
		                if wizard || obj.identified {
			                format!("[{}]", obj.class)
		                } else {
			                "".to_string()
		                }),
		Armor => format!("{}{} {}[{}] ",
		                 if obj.d_enchant >= 0 { "+" } else { "" },
		                 obj.d_enchant,
		                 obj.title(id_tables).as_str(),
		                 get_armor_class(Some(obj))),
		Weapon => format!("{}{}{},{}{} {}",
		                  get_quantity(obj),
		                  if obj.hit_enchant >= 0 { "+" } else { "" }, obj.hit_enchant,
		                  if obj.d_enchant >= 0 { "+" } else { "" }, obj.d_enchant,
		                  name_of(obj, fruit, id_tables)
		),
		_ => panic!("invalid identified object")
	}
}

unsafe fn get_called(obj: &object, fruit: String, id_tables: &NoteTables) -> String {
	match obj.what_is {
		Scroll | Potion | Wand | Ring => {
			let name = name_of(obj, fruit, id_tables);
			let title = obj.title(id_tables);
			format!("{}{}called {}", get_quantity(obj), name, title.as_str())
		}
		_ => panic!("invalid called object"),
	}
}

unsafe fn get_unidentified(obj: &object, fruit: String, id_tables: &NoteTables) -> String {
	let what = obj.what_is;
	match what {
		Scroll => format!("{}{}entitled: {}", get_quantity(obj), name_of(obj, fruit, id_tables), obj.title(id_tables).as_str()),
		Potion => format!("{}{}{}", get_quantity(obj), obj.title(id_tables).as_str(), name_of(obj, fruit, id_tables)),
		Wand | Ring => if obj.identified || get_id_status(obj, id_tables) == NoteStatus::Identified {
			get_identified(obj, fruit, id_tables)
		} else if get_id_status(obj, id_tables) == NoteStatus::Called {
			get_called(obj, fruit, id_tables)
		} else {
			format!("{}{}{}", get_quantity(obj), obj.title(id_tables).as_str(), name_of(obj, fruit, id_tables))
		},
		Armor => if obj.identified {
			get_identified(obj, fruit, id_tables)
		} else {
			obj.title(id_tables).to_string()
		},
		Weapon => if obj.identified {
			get_identified(obj, fruit, id_tables)
		} else {
			name_of(obj, fruit, id_tables)
		},
		_ => panic!("invalid unidentified object")
	}
}

impl Player {
	pub unsafe fn get_obj_desc(&self, obj_id: ObjectId) -> String {
		let obj = self.expect_object(obj_id);
		let fruit = self.settings.fruit.to_string();
		let obj_ichar = obj.ichar;
		let obj_desc = get_obj_desc(&obj, fruit, &self.notes);
		format!("{}({})", obj_desc, obj_ichar)
	}
}

pub unsafe fn get_obj_desc(obj: &object, fruit: String, id_tables: &NoteTables) -> String {
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
		format!("{}{}", quantity, name_of(obj, fruit, id_tables))
	} else {
		if wizard {
			get_identified(obj, fruit, id_tables)
		} else {
			match what_is {
				Weapon | Armor | Wand | Ring => get_unidentified(obj, fruit, id_tables),
				_ => match get_id_status(obj, id_tables) {
					NoteStatus::Unidentified => get_unidentified(obj, fruit, id_tables),
					NoteStatus::Identified => get_identified(obj, fruit, id_tables),
					NoteStatus::Called => get_called(obj, fruit, id_tables),
				}
			}
		}
	};
	let desc = if desc.starts_with("a ")
		&& is_vowel(desc.chars().nth(2).expect("char at 2")) {
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

fn take_unused<const N: usize>(used: &mut [bool; N]) -> usize {
	let mut j;
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
		let obj_desc = get_obj_desc(obj, player.settings.fruit.to_string(), &player.notes);
		let msg = format!("{}{} {}", ch, separator, obj_desc);
		message(&msg, 0);
	} else {
		message("no such item.", 0);
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
