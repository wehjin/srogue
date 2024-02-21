use ncurses::{clrtoeol, mv, mvaddstr, mvinch, refresh};

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::resources::keyboard::CANCEL;
use crate::objects::{get_armor_class, name_of, NoteStatus, Object, ObjectId};
use crate::objects::note_tables::NoteTables;
use crate::pack::{pack_letter, wait_for_ack};
use crate::player::Player;
use crate::potions::kind::PotionKind;
use crate::prelude::food_kind::RATION;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::{Amulet, Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::ring::ring_kind::RingKind;
use crate::score::is_vowel;
use crate::scrolls::ScrollKind;
use crate::zap::wand_kind::WandKind;

pub fn inventory(filter: PackFilter, game: &mut GameState) {
	if game.player.pack().is_empty() {
		game.dialog.message("your pack is empty", 0);
		return;
	}
	let item_lines = {
		let mut item_lines = Vec::new();
		for obj in game.player.pack().objects() {
			let what = obj.what_is;
			if filter.includes(what) {
				let close_char = if what == Armor && obj.is_protected != 0 { '}' } else { ')' };
				let obj_ichar = obj.ichar;
				let obj_desc = get_obj_desc(obj, game.player.settings.fruit.to_string(), &game.player);
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

fn get_quantity(obj: &Object) -> String {
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

fn get_id_real(obj: &Object) -> &'static str {
	match obj.what_is {
		Scroll => ScrollKind::from_index(obj.which_kind as usize).real_name(),
		Potion => PotionKind::from_index(obj.which_kind as usize).real_name(),
		Wand => WandKind::from_index(obj.which_kind as usize).real_name(),
		Ring => RingKind::from_index(obj.which_kind as usize).real_name(),
		_ => "",
	}
}

fn get_identified(obj: &Object, fruit: String, player: &Player) -> String {
	let what = obj.what_is;
	match what {
		Scroll | Potion => {
			let quantity = get_quantity(obj);
			let name = name_of(obj, fruit, &player.notes);
			let real_name = get_id_real(obj);
			format!("{}{}{}", quantity, name, real_name)
		}
		Ring => {
			let more_info =
				if (player.wizard || obj.identified)
					&& (obj.which_kind == RingKind::Dexterity.to_index() as u16 || obj.which_kind == RingKind::AddStrength.to_index() as u16) {
					format!("{}{} ", if obj.class > 0 { "+" } else { "" }, obj.class)
				} else {
					"".to_string()
				};
			let name = name_of(obj, fruit, &player.notes);
			let real_name = get_id_real(obj);
			format!("{}{}{}{}", get_quantity(obj), more_info, name, real_name)
		}
		Wand => format!("{}{}{}{}",
		                get_quantity(obj),
		                name_of(obj, fruit, &player.notes),
		                get_id_real(obj),
		                if player.wizard || obj.identified {
			                format!("[{}]", obj.class)
		                } else {
			                "".to_string()
		                }),
		Armor => {
			let armor_class = get_armor_class(Some(obj));
			let enchantment = obj.d_enchant;
			let plus_or_none = if enchantment >= 0 { "+" } else { "" };
			let title = player.notes.title(what, obj.which_kind as usize);
			format!("{}{} {}[{}] ", plus_or_none, enchantment, title.as_str(), armor_class)
		}
		Weapon => format!("{}{}{},{}{} {}",
		                  get_quantity(obj),
		                  if obj.hit_enchant >= 0 { "+" } else { "" }, obj.hit_enchant,
		                  if obj.d_enchant >= 0 { "+" } else { "" }, obj.d_enchant,
		                  name_of(obj, fruit, &player.notes)
		),
		_ => panic!("invalid identified object")
	}
}

fn get_called(obj: &Object, fruit: String, notes: &NoteTables) -> String {
	let what = obj.what_is;
	match what {
		Scroll | Potion | Wand | Ring => {
			let name = name_of(obj, fruit, notes);
			let title = notes.title(what, obj.which_kind as usize);
			format!("{}{}called {}", get_quantity(obj), name, title.as_str())
		}
		_ => panic!("invalid called object"),
	}
}

fn get_unidentified(obj: &Object, fruit: String, player: &Player) -> String {
	let what = obj.what_is;
	let kind = obj.which_kind as usize;
	match what {
		Scroll => {
			let title = player.notes.title(what, kind);
			format!("{}{}entitled: {}", get_quantity(obj), name_of(obj, fruit, &player.notes), title.as_str())
		}
		Potion => {
			let title = player.notes.title(what, kind);
			format!("{}{}{}", get_quantity(obj), title.as_str(), name_of(obj, fruit, &player.notes))
		}
		Wand | Ring => {
			if obj.identified || player.notes.status(what, kind) == NoteStatus::Identified {
				get_identified(obj, fruit, player)
			} else if player.notes.status(what, kind) == NoteStatus::Called {
				get_called(obj, fruit, &player.notes)
			} else {
				let title = player.notes.title(what, kind);
				format!("{}{}{}", get_quantity(obj), title.as_str(), name_of(obj, fruit, &player.notes))
			}
		}
		Armor => if obj.identified {
			get_identified(obj, fruit, player)
		} else {
			player.notes.title(what, kind).to_string()
		},
		Weapon => if obj.identified {
			get_identified(obj, fruit, player)
		} else {
			name_of(obj, fruit, &player.notes)
		},
		_ => panic!("invalid unidentified object")
	}
}

impl Player {
	pub fn get_obj_desc(&self, obj_id: ObjectId) -> String {
		let obj = self.expect_object(obj_id);
		let fruit = self.settings.fruit.to_string();
		let obj_ichar = obj.ichar;
		let obj_desc = get_obj_desc(&obj, fruit, self);
		format!("{}({})", obj_desc, obj_ichar)
	}
}

pub fn get_obj_desc(obj: &Object, fruit: String, player: &Player) -> String {
	let what = obj.what_is;
	if what == Amulet {
		return "the amulet of Yendor ".to_string();
	}
	if what == Gold {
		return format!("{} pieces of gold", obj.quantity);
	}

	let desc = if what == Food {
		let quantity = if obj.which_kind == RATION {
			if obj.quantity > 1 {
				format!("{} rations of ", obj.quantity)
			} else {
				"some ".to_string()
			}
		} else {
			"a ".to_string()
		};
		format!("{}{}", quantity, name_of(obj, fruit, &player.notes))
	} else {
		if player.wizard {
			get_identified(obj, fruit, player)
		} else {
			match what {
				Weapon | Armor | Wand | Ring => {
					get_unidentified(obj, fruit, player)
				}
				_ => {
					match player.notes.status(what, obj.which_kind as usize) {
						NoteStatus::Unidentified => get_unidentified(obj, fruit, player),
						NoteStatus::Identified => get_identified(obj, fruit, player),
						NoteStatus::Called => get_called(obj, fruit, &player.notes),
					}
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

fn get_in_use_description(obj: &Object) -> &'static str {
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

pub fn single_inv(ichar: Option<char>, game: &mut GameState) {
	let ch = if let Some(ichar) = ichar {
		ichar
	} else {
		pack_letter("inventory what?", AllObjects, game)
	};
	if ch == CANCEL {
		return;
	}
	if let Some(obj) = game.player.object_with_letter(ch) {
		let separator = if obj.what_is == Armor && obj.is_protected != 0 { '}' } else { ')' };
		let obj_desc = get_obj_desc(obj, game.player.settings.fruit.to_string(), &game.player);
		let msg = format!("{}{} {}", ch, separator, obj_desc);
		game.dialog.message(&msg, 0);
	} else {
		game.dialog.message("no such item.", 0);
	}
}

pub fn inv_armor_weapon(is_weapon: bool, game: &mut GameState) {
	if is_weapon {
		if let Some(weapon) = game.player.weapon() {
			single_inv(Some(weapon.ichar), game);
		} else {
			game.dialog.message("not wielding anything", 0);
		}
	} else {
		if let Some(armor) = game.player.armor() {
			single_inv(Some(armor.ichar), game);
		} else {
			game.dialog.message("not wearing anything", 0);
		}
	}
}
