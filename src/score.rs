#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::cmp::{Ordering};
use std::fs::File;
use std::io::{Read, Seek, Write};
use ncurses::{mv, mvaddch, mvaddstr, mvinch, refresh, standend, standout};
use settings::{score_only, show_skull};
use crate::prelude::*;
use crate::{settings, turn_into_games, turn_into_user};
use crate::level::constants::{DCOLS, DROWS};
use crate::objects::IdStatus::Identified;
use crate::player::Player;
use crate::prelude::armor_kind::ARMORS;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::potion_kind::POTIONS;
use crate::prelude::scroll_kind::SCROLLS;
use crate::prelude::wand_kind::MAX_WAND;
use crate::prelude::weapon_kind::{ARROW, DAGGER, DART, SHURIKEN, WEAPONS};
use crate::settings::{login_name, nick_name};

pub const SCORE_FILE: &'static str = "/usr/games/player.rogue.scores";

pub unsafe fn killed_by(ending: Ending, player: &mut Player) {
	md_ignore_signals();
	if !ending.is_quit() {
		player.rogue.gold = ((player.rogue.gold as f64 * 9.0) / 10.0) as usize;
	}

	let mut how_ended = ending_string(&ending);
	how_ended += &format!(" with {} gold", player.rogue.gold);

	if ending.is_monster() && show_skull() {
		ncurses::clear();
		mvaddstr(4, 32, "__---------__");
		mvaddstr(5, 30, "_~             ~_");
		mvaddstr(6, 29, "/                 \\");
		mvaddstr(7, 28, "~                   ~");
		mvaddstr(8, 27, "/                     \\");
		mvaddstr(9, 27, "|    XXXX     XXXX    |");
		mvaddstr(10, 27, "|    XXXX     XXXX    |");
		mvaddstr(11, 27, "|    XXX       XXX    |");
		mvaddstr(12, 28, "\\         @         /");
		mvaddstr(13, 29, "--\\     @@@     /--");
		mvaddstr(14, 30, "| |    @@@    | |");
		mvaddstr(15, 30, "| |           | |");
		mvaddstr(16, 30, "| vvVvvvvvvvVvv |");
		mvaddstr(17, 30, "|  ^^^^^^^^^^^  |");
		mvaddstr(18, 31, "\\_           _/");
		mvaddstr(19, 33, "~---------~");
		let skull_name = if let Some(nick_name) = nick_name() {
			nick_name
		} else {
			login_name()
		};
		center(21, &skull_name);
		center(22, &how_ended);
	} else {
		message(&how_ended, 0);
	}
	message("", 0);
	put_scores(Some(ending), player);
}

unsafe fn ending_string(ending: &Ending) -> String {
	match ending {
		&Ending::Monster(monster) => {
			let name = monster.name();
			let article = if is_vowel(name.chars().nth(0).unwrap()) { "an" } else { "a" };
			format!("Killed by {} {}", article, name)
		}
		&Ending::Hypothermia => "died of hypothermia".to_string(),
		&Ending::Starvation => "died of starvation".to_string(),
		&Ending::PoisonDart => "killed by a dart".to_string(),
		&Ending::Quit => "quit".to_string(),
		&Ending::Win => "a total winner".to_string()
	}
}

pub unsafe fn win(player: &mut Player, level: &mut Level) {
	unwield(player);          /* disarm and relax */
	unwear(player);
	for hand in PlayerHand::ALL_HANDS {
		un_put_hand(hand, player, level);
	}
	ncurses::clear();
	mvaddstr(10, 11, "@   @  @@@   @   @      @  @  @   @@@   @   @   @");
	mvaddstr(11, 11, " @ @  @   @  @   @      @  @  @  @   @  @@  @   @");
	mvaddstr(12, 11, "  @   @   @  @   @      @  @  @  @   @  @ @ @   @");
	mvaddstr(13, 11, "  @   @   @  @   @      @  @  @  @   @  @  @@");
	mvaddstr(14, 11, "  @    @@@    @@@        @@ @@    @@@   @   @   @");
	mvaddstr(17, 11, "Congratulations,  you have  been admitted  to  the");
	mvaddstr(18, 11, "Fighters' Guild.   You return home,  sell all your");
	mvaddstr(19, 11, "treasures at great profit and retire into comfort.");
	message("", 0);
	message("", 0);
	id_all();
	sell_pack(player);
	put_scores(Some(Ending::Win), player);
}

pub unsafe fn quit(from_intrpt: bool, player: &mut Player) {
	md_ignore_signals();
	let mut orow = 0;
	let mut ocol = 0;
	let mut mc = false;
	let mut buf = [0; 128];
	if from_intrpt {
		orow = player.rogue.row;
		ocol = player.rogue.col;
		mc = msg_cleared;
		for i in 0..DCOLS {
			buf[i] = mvinch(0, i as i32);
		}
	}
	check_message();
	message("really quit?", 1);
	if rgetchar() != 'y' {
		md_heed_signals();
		check_message();
		if from_intrpt {
			for i in 0..DCOLS {
				mvaddch(0, i as i32, buf[i]);
			}
			msg_cleared = mc;
			mv(orow as i32, ocol as i32);
			refresh();
		}
		return;
	}
	if from_intrpt {
		clean_up(BYEBYE_STRING);
	}
	check_message();
	killed_by(Ending::Quit, player);
}

pub unsafe fn put_scores(ending: Option<Ending>, player: &Player) {
	turn_into_games();
	let mut file = File::options().read(true).write(true).open(SCORE_FILE).unwrap_or_else(|_| {
		File::options().write(true).open(SCORE_FILE).unwrap_or_else(|_| {
			message("cannot read/write/create score file", 0);
			sf_error();
			unreachable!("sf_error")
		})
	});
	turn_into_user();

	let mut score_only = score_only();
	let mut scores: Vec<String> = Vec::new();
	let mut n_names: Vec<String> = Vec::new();
	{
		let mut scores_string = String::new();
		if file.read_to_string(&mut scores_string).is_err() {
			sf_error();
			unreachable!("after sf_error");
		};
		for (i, line) in scores_string.lines().enumerate() {
			if i & 1 == 0 {
				scores.push(line.to_string());
			} else {
				n_names.push(line.to_string());
			}
		}
	}
	let mut found_player = None;
	let max_search = scores.len().min(n_names.len()).min(10);
	for i in 0..max_search {
		if !score_only {
			let name_in_score = &scores[i][START_OF_NAME..];
			if name_cmp(name_in_score, &login_name()) == Ordering::Equal {
				if let Some(gold_in_score) = gold_in_score(&scores[i]) {
					if player.rogue.gold < gold_in_score {
						score_only = true;
					} else {
						found_player = Some(i);
					}
				} else {
					sf_error();
					unreachable!("sf error");
				}
			}
		}
	}
	if let Some(found_player) = found_player {
		scores.remove(found_player);
		n_names.remove(found_player);
	}

	let mut rank = 10;
	if !score_only {
		let ne = scores.len().min(n_names.len()).min(10);
		for i in 0..ne {
			if let Some(gold_in_score) = gold_in_score(&scores[i]) {
				if player.rogue.gold >= gold_in_score {
					rank = i;
					break;
				}
			} else {
				sf_error();
				unreachable!("sf error");
			}
		}
		if ne == 0 {
			rank = 0;
		} else if (ne < 10) && (rank == 10) {
			rank = ne;
		}
		if rank < 10 {
			let name = match nick_name() {
				None => "".to_string(),
				Some(name) => name.to_string(),
			};
			insert_score(&mut scores, &mut n_names, &name, rank, ne, ending.expect("ending"), player);
		}
		file.rewind().expect("rewind file");
	}

	ncurses::clear();
	mvaddstr(3, 30, "Top  Ten  Rogueists");
	mvaddstr(8, 0, "Rank   Score   Name");

	md_ignore_signals();

	let ne = scores.len().min(n_names.len()).min(10);
	for i in 0..ne {
		let name = &n_names[i];
		let score = &scores[i];
		let revised_rank = format!("{:2}", i + 1) + &score[2..];
		let revised_name = replace_name_in_score(&revised_rank, name);
		if i == rank {
			standout();
		}
		mvaddstr((i + 10) as i32, 0, &revised_name);
		if i == rank {
			standend();
		}
		if rank < 10 {
			let score_and_name = format!("{}\n{}\n", revised_name, name);
			file.write(score_and_name.as_bytes()).expect("write score and name");
		}
	}
	refresh();
	drop(file);
	message("", 0);
	clean_up("");
}

fn gold_in_score(score: &str) -> Option<usize> {
	let slice = &score[6..12];
	let trimmed = slice.trim();
	trimmed.parse::<usize>().ok()
}

unsafe fn insert_score(
	scores: &mut Vec<String>,
	n_names: &mut Vec<String>,
	n_name: &str,
	rank: usize,
	n: usize,
	ending: Ending,
	player: &Player,
) {
	let mut buf = format!("{:2}    {:6}   {}: ", rank + 1, player.rogue.gold, login_name());
	buf += &ending_string(&ending);
	buf += &format!(" on level {} ", player.max_depth);
	if (!ending.is_win()) && has_amulet(player) {
		buf += "with amulet";
	}
	insert_to_limit(scores, &buf, rank, n);
	insert_to_limit(n_names, n_name, rank, n);
}

fn insert_to_limit(vec: &mut Vec<String>, s: &str, rank: usize, limit: usize) {
	if rank < vec.len() {
		vec.insert(rank, s.to_string());
	} else {
		vec.push(s.to_string());
	}
	if vec.len() > limit {
		vec.pop();
	}
}


pub fn is_vowel(ch: char) -> bool {
	match ch {
		'a' | 'e' | 'i' | 'o' | 'u' => true,
		_ => false
	}
}

pub unsafe fn sell_pack(player: &mut Player)
{
	ncurses::clear();
	mvaddstr(1, 0, "Value      Item");
	let mut row: usize = 2;
	for id in player.object_ids() {
		if player.object_what(id) != ObjectWhat::Food {
			let obj = player.object_mut(id).expect("obj in player");
			obj.identified = true;
			let obj_value = get_value(obj);
			let obj_desc = get_obj_desc(obj);
			player.rogue.gold += obj_value;
			if row < DROWS {
				let msg = format!("{:5}      {}", obj_value, obj_desc);
				mvaddstr(row as i32, 0, &msg);
				row += 1;
			}
		}
	}
	refresh();
	if player.rogue.gold > MAX_GOLD {
		player.rogue.gold = MAX_GOLD;
	}
	message("", 0);
}

unsafe fn get_value(obj: &obj) -> usize {
	let wc = obj.which_kind;
	let mut val = match obj.what_is {
		ObjectWhat::Weapon => {
			let mut val = id_weapons[wc as usize].value;
			if (wc == ARROW) || (wc == DAGGER) || (wc == SHURIKEN) || (wc == DART) {
				val *= obj.quantity;
			}
			val += obj.d_enchant as i16 * 85;
			val += obj.hit_enchant * 85;
			val
		}
		ObjectWhat::Armor => {
			let mut val = id_armors[wc as usize].value;
			val += obj.d_enchant as i16 * 75;
			if obj.is_protected != 0 {
				val += 200;
			}
			val
		}
		ObjectWhat::Wand => id_wands[wc as usize].value * (obj.class as i16 + 1),
		ObjectWhat::Scroll => id_scrolls[wc as usize].value * obj.quantity,
		ObjectWhat::Potion => id_potions[wc as usize].value * obj.quantity,
		ObjectWhat::Amulet => 5000,
		ObjectWhat::Ring => id_rings[wc as usize].value * (obj.class as i16 + 1),
		ObjectWhat::Gold => 0,
		ObjectWhat::Food => 0,
		ObjectWhat::None => 0,
	};
	if val <= 0 {
		val = 10;
	}
	return val as usize;
}


pub unsafe fn id_all()
{
	for i in 0..SCROLLS {
		id_scrolls[i].id_status = Identified;
	}
	for i in 0..WEAPONS {
		id_weapons[i].id_status = Identified;
	}
	for i in 0..ARMORS {
		id_armors[i].id_status = Identified;
	}
	for i in 0..MAX_WAND {
		id_wands[i].id_status = Identified;
	}
	for i in 0..POTIONS {
		id_potions[i].id_status = Identified;
	}
}

pub fn name_cmp(s1: &str, s2: &str) -> Ordering {
	let pre_colon = if let Some(pos) = s1.chars().position(|c| c == ':') {
		&s1[0..pos]
	} else {
		&s1[..]
	};
	pre_colon.cmp(s2)
}

const START_OF_NAME: usize = 15;

fn name_in_score(score: &str) -> String {
	let name_and_more = &score[START_OF_NAME..];
	if let Some(pos) = name_and_more.find(':') {
		&name_and_more[..pos]
	} else {
		name_and_more
	}.to_string()
}

fn replace_name_in_score(score: &str, new_name: &str) -> String {
	if new_name.is_empty() {
		score.to_string()
	} else {
		let left = &score[..START_OF_NAME];
		let middle = new_name;
		let right = &score[(START_OF_NAME + name_in_score(score).chars().count())..];
		format!("{}{}{}", left, middle, right)
	}
}


pub fn center(row: i64, msg: &str) {
	let margin = (DCOLS - msg.len()) / 2;
	mvaddstr(row as i32, margin as i32, msg);
}

pub unsafe fn sf_error() {
	message("", 1);
	clean_up("sorry, score file is out of order");
}
