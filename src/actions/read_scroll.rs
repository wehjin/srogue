use crate::actions::GameUpdater;
use crate::init::{Dungeon, GameState};
use crate::monster::{aggravate, create_monster};
use crate::objects::NoteStatus::Identified;
use crate::pack::pack_letter;
use crate::player::Player;
use crate::prelude::object_what::ObjectWhat::{Armor, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Scrolls};
use crate::random::coin_toss;
use crate::resources::diary;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::room::draw_magic_map;
use crate::scrolls::ScrollKind;
use crate::systems::play_level::LevelResult;
use crate::trap::is_off_screen;

pub struct ReadScroll;

impl GameUpdater for ReadScroll {
	fn update(game: &mut GameState) -> Option<LevelResult> {
		read_scroll(game);
		None
	}
}

fn read_scroll(game: &mut GameState) {
	if game.player.health.blind.is_active() {
		game.diary.add_entry("You can't see to read the scroll.");
		return;
	}

	let ch = pack_letter("read what?", Scrolls, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.diary.add_entry("no such item.");
			return;
		}
		Some(obj_id) => {
			match game.player.expect_object(obj_id).scroll_kind() {
				None => {
					game.diary.add_entry("you can't read that");
					return;
				}
				Some(scroll_kind) => {
					match scroll_kind {
						ScrollKind::ScareMonster => {
							game.diary.add_entry("you hear a maniacal laughter in the distance");
						}
						ScrollKind::HoldMonster => {
							hold_monster(game);
						}
						ScrollKind::EnchWeapon => {
							let glow_color = crate::r#use::get_ench_color(&game.player);
							if let Some(weapon_id) = game.player.weapon_id() {
								let weapon_name = game.player.name_of(weapon_id);
								let weapon = game.player.expect_object_mut(weapon_id);
								let plural_char = if weapon.quantity <= 1 { "s" } else { "" };
								let msg = format!("your {}glow{} {}for a moment", weapon_name, plural_char, glow_color);
								game.diary.add_entry(&msg);
								if coin_toss() {
									weapon.hit_enchant += 1;
								} else {
									weapon.d_enchant += 1;
								}
								weapon.is_cursed = 0;
							} else {
								game.diary.add_entry("your hands tingle");
							}
						}
						ScrollKind::EnchArmor => {
							let glow_color = crate::r#use::get_ench_color(&game.player);
							if let Some(armor) = game.player.armor_mut() {
								let msg = format!("your armor glows {} for a moment", glow_color.trim());
								game.diary.add_entry(&msg);
								armor.raise_armor_enchant(1);
								armor.is_cursed = 0;
								game.as_diary_mut().set_stats_changed(true);
							} else {
								game.diary.add_entry("your skin crawls");
							}
						}
						ScrollKind::Identify => {
							game.diary.add_entry("this is a scroll of identify");
							let obj = game.player.expect_object_mut(obj_id);
							obj.identified = true;
							let note = game.player.notes.note_mut(Scroll, scroll_kind.to_index());
							note.status = Identified;
							idntfy(game);
						}
						ScrollKind::Teleport => {
							crate::r#use::tele(game);
						}
						ScrollKind::Sleep => {
							game.diary.add_entry("you fall asleep");
							crate::r#use::take_a_nap(game);
						}
						ScrollKind::ProtectArmor => {
							if let Some(armor) = game.player.armor_mut() {
								game.diary.add_entry("your armor is covered by a shimmering gold shield");
								armor.is_protected = 1;
								armor.is_cursed = 0;
							} else {
								game.diary.add_entry("your acne seems to have disappeared");
							}
						}
						ScrollKind::RemoveCurse => {
							let msg = if game.player.health.halluc.is_active() {
								"you feel in touch with the universal oneness"
							} else {
								"you feel as though someone is watching over you"
							};
							game.diary.add_entry(msg);
							uncurse_all(&mut game.player);
						}
						ScrollKind::CreateMonster => {
							create_monster(game);
						}
						ScrollKind::AggravateMonster => {
							aggravate(game);
						}
						ScrollKind::MagicMapping => {
							game.diary.add_entry("this scroll seems to have a map on it");
							draw_magic_map(game);
						}
					}
					game.player.notes.identify_if_un_called(Scroll, scroll_kind.to_index());
					crate::r#use::vanish(obj_id, scroll_kind != ScrollKind::Sleep, game);
				}
			}
		}
	}
}

fn idntfy(game: &mut GameState) {
	loop {
		let ch = pack_letter("what would you like to identify?", AllObjects, game);
		if ch == CANCEL_CHAR {
			return;
		}
		match game.player.object_id_with_letter(ch) {
			None => {
				game.diary.add_entry("no such item, try again");
				game.diary.add_entry("");
				diary::show_current_page(&game.diary);
				game.diary.turn_page();
				continue;
			}
			Some(obj_id) => {
				let obj = game.player.expect_object_mut(obj_id);
				obj.identified = true;
				let what = obj.what_is;
				match what {
					Scroll | Potion | Weapon | Armor | Wand | Ring => {
						let kind = obj.which_kind as usize;
						game.player.notes.identify(what, kind);
					}
					_ => {}
				}
				let msg = game.player.get_obj_desc(obj_id);
				game.diary.add_entry(&msg);
			}
		}
	}
}

fn hold_monster(game: &mut GameState) {
	let mut mcount = 0;
	for i in -2..=2 {
		for j in -2..=2 {
			let row = game.player.rogue.row + i;
			let col = game.player.rogue.col + j;
			if is_off_screen(row, col) {
				continue;
			}
			if game.level.dungeon[row as usize][col as usize].has_monster() {
				let monster = game.mash.monster_at_spot_mut(row, col).expect("monster at spot");
				monster.m_flags.asleep = true;
				monster.m_flags.wakens = false;
				mcount += 1;
			}
		}
	}
	if mcount == 0 {
		game.diary.add_entry("you feel a strange sense of loss");
	} else if mcount == 1 {
		game.diary.add_entry("the monster freezes");
	} else {
		game.diary.add_entry("the monsters around you freeze");
	}
}

fn uncurse_all(player: &mut Player) {
	for obj_id in player.object_ids() {
		let obj = player.expect_object_mut(obj_id);
		obj.is_cursed = 0;
	}
}
