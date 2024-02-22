use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::monster::{aggravate, create_monster};
use crate::objects::NoteStatus::Identified;
use crate::pack::pack_letter;
use crate::player::Player;
use crate::prelude::object_what::ObjectWhat::{Armor, Potion, Ring, Scroll, Wand, Weapon};
use crate::prelude::object_what::PackFilter::{AllObjects, Scrolls};
use crate::random::coin_toss;
use crate::resources::keyboard::CANCEL_CHAR;
use crate::room::draw_magic_map;
use crate::scrolls::ScrollKind;
use crate::trap::is_off_screen;

pub struct ReadScroll;

impl PlayerAction for ReadScroll {
	fn update(_input_key: char, game: &mut GameState) {
		read_scroll(game);
	}
}

fn read_scroll(game: &mut GameState) {
	if game.player.blind.is_active() {
		game.dialog.message("You can't see to read the scroll.", 0);
		return;
	}

	let ch = pack_letter("read what?", Scrolls, game);
	if ch == CANCEL_CHAR {
		return;
	}
	match game.player.object_id_with_letter(ch) {
		None => {
			game.dialog.message("no such item.", 0);
			return;
		}
		Some(obj_id) => {
			match game.player.expect_object(obj_id).scroll_kind() {
				None => {
					game.dialog.message("you can't read that", 0);
					return;
				}
				Some(scroll_kind) => {
					match scroll_kind {
						ScrollKind::ScareMonster => {
							game.dialog.message("you hear a maniacal laughter in the distance", 0);
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
								game.dialog.message(&msg, 0);
								if coin_toss() {
									weapon.hit_enchant += 1;
								} else {
									weapon.d_enchant += 1;
								}
								weapon.is_cursed = 0;
							} else {
								game.dialog.message("your hands tingle", 0);
							}
						}
						ScrollKind::EnchArmor => {
							let glow_color = crate::r#use::get_ench_color(&game.player);
							if let Some(armor) = game.player.armor_mut() {
								let msg = format!("your armor glows {} for a moment", glow_color.trim());
								game.dialog.message(&msg, 0);
								armor.raise_armor_enchant(1);
								armor.is_cursed = 0;
								game.stats_changed = true;
							} else {
								game.dialog.message("your skin crawls", 0);
							}
						}
						ScrollKind::Identify => {
							game.dialog.message("this is a scroll of identify", 0);
							let obj = game.player.expect_object_mut(obj_id);
							obj.identified = true;
							game.player.notes.note_mut(Scroll, scroll_kind.to_index()).status = Identified;
							idntfy(game);
						}
						ScrollKind::Teleport => {
							crate::r#use::tele(game);
						}
						ScrollKind::Sleep => {
							game.dialog.message("you fall asleep", 0);
							crate::r#use::take_a_nap(game);
						}
						ScrollKind::ProtectArmor => {
							if let Some(armor) = game.player.armor_mut() {
								game.dialog.message("your armor is covered by a shimmering gold shield", 0);
								armor.is_protected = 1;
								armor.is_cursed = 0;
							} else {
								game.dialog.message("your acne seems to have disappeared", 0);
							}
						}
						ScrollKind::RemoveCurse => {
							let msg = if game.player.halluc.is_active() {
								"you feel in touch with the universal oneness"
							} else {
								"you feel as though someone is watching over you"
							};
							game.dialog.message(msg, 0);
							uncurse_all(&mut game.player);
						}
						ScrollKind::CreateMonster => {
							create_monster(game);
						}
						ScrollKind::AggravateMonster => {
							aggravate(game);
						}
						ScrollKind::MagicMapping => {
							game.dialog.message("this scroll seems to have a map on it", 0);
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
				game.dialog.message("no such item, try again", 0);
				game.dialog.message("", 0);
				game.dialog.clear_message();
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
				game.dialog.message(&msg, 0);
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
		game.dialog.message("you feel a strange sense of loss", 0);
	} else if mcount == 1 {
		game.dialog.message("the monster freezes", 0);
	} else {
		game.dialog.message("the monsters around you freeze", 0);
	}
}

fn uncurse_all(player: &mut Player) {
	for obj_id in player.object_ids() {
		let obj = player.expect_object_mut(obj_id);
		obj.is_cursed = 0;
	}
}
