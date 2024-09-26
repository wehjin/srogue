use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::objects::note_tables::NoteTables;
use crate::objects::{get_armor_class, name_of, NoteStatus, Object, ObjectId, ObjectPack};
use crate::pack::{pack_letter, wait_for_ack};
use crate::player::Player;
use crate::potions::kind::PotionKind;
use crate::prelude::food_kind::RATION;
use crate::prelude::item_usage::{BEING_WIELDED, BEING_WORN, ON_LEFT_HAND, ON_RIGHT_HAND};
use crate::prelude::object_what::ObjectWhat::{Amulet, Armor, Food, Gold, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter;
use crate::prelude::object_what::PackFilter::AllObjects;
use crate::render_system::backend;
use crate::resources::dungeon::stats::DungeonStats;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::ring::ring_kind::RingKind;
use crate::score::is_vowel;
use crate::scrolls::ScrollKind;
use crate::zap::wand_kind::WandKind;

pub fn inventory(pack: &ObjectPack, filter: PackFilter, fruit: &str, notes: &NoteTables, wizard: bool) -> Vec<String> {
	if pack.is_empty() {
		vec!["your pack is empty".to_string()]
	} else {
		let mut item_lines = Vec::new();
		for obj in pack.objects() {
			let what = obj.what_is;
			if filter.includes(what) {
				let close_char = if what == Armor && obj.is_protected != 0 { '}' } else { ')' };
				let obj_ichar = obj.ichar;
				let obj_desc = obj.to_description(fruit, notes, wizard);
				let line = format!("{obj_ichar}{close_char} {obj_desc}");
				item_lines.push(line);
			}
		}
		item_lines.push("--press space to continue--".to_string());
		item_lines
	}
}

pub enum ObjectSource {
	Player,
	Ground,
}

impl ObjectPack {
	fn to_pickup_line_maybe(&self, obj_id: ObjectId, stats: &DungeonStats) -> String {
		let obj = self.as_object(obj_id);
		let obj_ichar = obj.ichar;
		let obj_desc = obj.to_description(&stats.fruit, &stats.notes, stats.wizard);
		format!("{}({})", obj_desc, obj_ichar)
	}
}

impl Object {
	fn to_description(&self, fruit: &str, notes: &NoteTables, wizard: bool) -> String {
		let what = self.what_is;
		if what == Amulet {
			return "the amulet of Yendor ".to_string();
		}
		if what == Gold {
			return format!("{} pieces of gold", self.quantity);
		}
		let desc = if what == Food {
			let quantity = if self.which_kind == RATION {
				if self.quantity > 1 {
					format!("{} rations of ", self.quantity)
				} else {
					"some ".to_string()
				}
			} else {
				"a ".to_string()
			};
			format!("{}{}", quantity, name_of(self, fruit.to_string(), notes))
		} else {
			if wizard {
				get_identified(self, fruit.to_string(), notes, wizard)
			} else {
				match what {
					Weapon | Armor | Wand | Ring => get_unidentified(self, fruit.to_string(), notes, wizard),
					_ => match notes.status(what, self.which_kind as usize) {
						NoteStatus::Unidentified => get_unidentified(self, fruit.to_string(), notes, wizard),
						NoteStatus::Identified => get_identified(self, fruit.to_string(), notes, wizard),
						NoteStatus::Called => get_called(self, fruit.to_string(), notes),
					},
				}
			}
		};
		let desc = if desc.starts_with("a ")
			&& is_vowel(desc.chars().nth(2).expect("char at 2")) {
			format!("an {}", &desc[2..])
		} else {
			desc
		};
		format!("{}{}", desc, get_in_use_description(self))
	}
}

pub fn inventory_legacy(filter: PackFilter, source: ObjectSource, game: &mut GameState) {
	let pack = match source {
		ObjectSource::Player => game.player.pack(),
		ObjectSource::Ground => &game.ground,
	};
	if pack.is_empty() {
		game.diary.add_entry("your pack is empty");
		return;
	}
	let item_lines = inventory(pack, filter, game.player.settings.fruit.as_str(), &game.player.notes, game.player.wizard);
	let item_lines_max_len = item_lines.iter().map(|it| it.chars().count()).max().expect("max length");

	let mut old_lines = Vec::new();
	let start_col = DCOLS - (item_lines_max_len + 2);
	let max_row = item_lines.len().min(DROWS);
	for row in 0..max_row {
		if row > 0 {
			let mut old_line: Vec<u8> = Vec::new();
			for col in start_col..DCOLS {
				let ch = backend::get_char((row, col).into());
				old_line.push(ch as u8)
			}
			old_lines.push(String::from_utf8(old_line).expect("utf8"));
		}
		backend::set_str(&item_lines[row], (row, start_col).into());
		backend::clear_to_eol();
	}
	backend::push_screen();
	wait_for_ack();

	backend::move_cursor((0, 0).into());
	backend::clear_to_eol();
	for row in 1..max_row {
		backend::set_str(&old_lines[row - 1], (row, start_col).into());
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

fn get_identified(obj: &Object, fruit: String, notes: &NoteTables, wizard: bool) -> String {
	let what = obj.what_is;
	match what {
		Scroll | Potion => {
			let quantity = get_quantity(obj);
			let name = name_of(obj, fruit, notes);
			let real_name = get_id_real(obj);
			format!("{}{}{}", quantity, name, real_name)
		}
		Ring => {
			let more_info =
				if (wizard || obj.identified)
					&& (obj.which_kind == RingKind::Dexterity.to_index() as u16 || obj.which_kind == RingKind::AddStrength.to_index() as u16) {
					format!("{}{} ", if obj.class > 0 { "+" } else { "" }, obj.class)
				} else {
					"".to_string()
				};
			let name = name_of(obj, fruit, notes);
			let real_name = get_id_real(obj);
			format!("{}{}{}{}", get_quantity(obj), more_info, name, real_name)
		}
		Wand => format!("{}{}{}{}",
		                get_quantity(obj),
		                name_of(obj, fruit, notes),
		                get_id_real(obj),
		                if wizard || obj.identified {
			                format!("[{}]", obj.class)
		                } else {
			                "".to_string()
		                }),
		Armor => {
			let armor_class = get_armor_class(Some(obj));
			let enchantment = obj.d_enchant;
			let plus_or_none = if enchantment >= 0 { "+" } else { "" };
			let title = notes.title(what, obj.which_kind as usize);
			format!("{}{} {}[{}] ", plus_or_none, enchantment, title.as_str(), armor_class)
		}
		Weapon => format!("{}{}{},{}{} {}",
		                  get_quantity(obj),
		                  if obj.hit_enchant >= 0 { "+" } else { "" }, obj.hit_enchant,
		                  if obj.d_enchant >= 0 { "+" } else { "" }, obj.d_enchant,
		                  name_of(obj, fruit, notes)
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

fn get_unidentified(obj: &Object, fruit: String, notes: &NoteTables, wizard: bool) -> String {
	let what = obj.what_is;
	let kind = obj.which_kind as usize;
	match what {
		Scroll => {
			let title = notes.title(what, kind);
			format!("{}{}entitled: {}", get_quantity(obj), name_of(obj, fruit, notes), title.as_str())
		}
		Potion => {
			let title = notes.title(what, kind);
			format!("{}{}{}", get_quantity(obj), title.as_str(), name_of(obj, fruit, notes))
		}
		Wand | Ring => {
			if obj.identified || notes.status(what, kind) == NoteStatus::Identified {
				get_identified(obj, fruit, notes, wizard)
			} else if notes.status(what, kind) == NoteStatus::Called {
				get_called(obj, fruit, notes)
			} else {
				let title = notes.title(what, kind);
				format!("{}{}{}", get_quantity(obj), title.as_str(), name_of(obj, fruit, notes))
			}
		}
		Armor => if obj.identified {
			get_identified(obj, fruit, notes, wizard)
		} else {
			notes.title(what, kind).to_string()
		},
		Weapon => if obj.identified {
			get_identified(obj, fruit, notes, wizard)
		} else {
			name_of(obj, fruit, notes)
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
			get_identified(obj, fruit, &player.notes, player.wizard)
		} else {
			match what {
				Weapon | Armor | Wand | Ring => {
					get_unidentified(obj, fruit, &player.notes, player.wizard)
				}
				_ => {
					match player.notes.status(what, obj.which_kind as usize) {
						NoteStatus::Unidentified => get_unidentified(obj, fruit, &player.notes, player.wizard),
						NoteStatus::Identified => get_identified(obj, fruit, &player.notes, player.wizard),
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
	if ch == CANCEL_CHAR {
		return;
	}
	if let Some(obj) = game.player.object_with_letter(ch) {
		let separator = if obj.what_is == Armor && obj.is_protected != 0 { '}' } else { ')' };
		let obj_desc = get_obj_desc(obj, game.player.settings.fruit.to_string(), &game.player);
		let msg = format!("{}{} {}", ch, separator, obj_desc);
		game.diary.add_entry(&msg);
	} else {
		game.diary.add_entry("no such item.");
	}
}
