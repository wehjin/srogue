#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]

use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{RwLock};
use ncurses::{clear, mv, mvaddch, mvaddstr, mvinch, refresh, standend, standout};
use settings::{score_only, show_skull};
use crate::prelude::*;
use crate::{settings, turn_into_games, turn_into_user};
use crate::objects::IdStatus::Identified;
use crate::prelude::armor_kind::ARMORS;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::potion_kind::POTIONS;
use crate::prelude::scroll_kind::SCROLLS;
use crate::prelude::wand_kind::WANDS;
use crate::prelude::weapon_kind::{ARROW, DAGGER, DART, SHURIKEN, WEAPONS};
use crate::settings::{login_name, nick_name};

pub const SCORE_FILE: &'static str = "/usr/games/rogue.scores";

pub unsafe fn killed_by(ending: Ending) {
	md_ignore_signals();
	if !ending.is_quit() {
		rogue.gold = ((rogue.gold as f64 * 9.0) / 10.0) as isize;
	}

	let mut how_ended = ending_string(&ending);
	how_ended += &format!(" with {} gold", rogue.gold);

	if ending.is_monster() && show_skull() {
		clear();
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
		center(21, skull_name);
		center(22, &how_ended);
	} else {
		message(&how_ended, 0);
	}
	message("", 0);
	put_scores(Some(ending));
}

unsafe fn ending_string(ending: &Ending) -> String {
	match ending {
		&Ending::Monster(monster) => {
			let name = mon_real_name(&monster);
			let article = if is_vowel(name.chars().nth(0).unwrap()) { "an" } else { "a" };
			&format!("Killed by {} {}", article, name)
		}
		&Ending::Hypothermia => "died of hypothermia",
		&Ending::Starvation => "died of starvation",
		&Ending::PoisonDart => "killed by a dart",
		&Ending::Quit => "quit",
		&Ending::Win => "a total winner"
	}.to_string()
}

pub unsafe fn win() {
	unwield(rogue.weapon);          /* disarm and relax */
	unwear(rogue.armor);
	un_put_on(rogue.left_ring);
	un_put_on(rogue.right_ring);

	clear();
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
	sell_pack();
	put_scores(Some(Ending::Win));
}

pub unsafe fn quit(from_intrpt: bool) {
	md_ignore_signals();
	let mut orow = 0;
	let mut ocol = 0;
	let mut mc = false;
	let mut buf = [0; 128];
	if from_intrpt {
		orow = rogue.row;
		ocol = rogue.col;
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
	killed_by(Ending::Quit);
}

pub unsafe fn put_scores(ending: Option<Ending>) {
	turn_into_games();
	let mut file = File::options().read(true).write(true).open(SCORE_FILE).unwrap_or_else(|_| {
		match File::options().write(true).open(SCORE_FILE) {
			Ok(file) => file,
			Err(_) => {
				message("cannot read/write/create score file", 0);
				sf_error();
				unreachable!("sf_error")
			}
		}
	});
	turn_into_user();
	xxx(true);

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
			if name_cmp(name_in_score, login_name()) == Ordering::Equal {
				if let Some(gold_in_score) = gold_in_score(&scores[i]) {
					if rogue.gold < gold_in_score {
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
				if rogue.gold >= gold_in_score {
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
			insert_score(&mut scores, &mut n_names, &name, rank, ne, ending.expect("ending"));
		}
		file.rewind().expect("rewind file");
	}

	clear();
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

fn gold_in_score(score: &str) -> Option<isize> {
	let slice = &score[6..12];
	let trimmed = slice.trim();
	trimmed.parse::<isize>().ok()
}

unsafe fn insert_score(scores: &mut Vec<String>, n_names: &mut Vec<String>, n_name: &str, rank: usize, n: usize, ending: Ending) {
	let mut buf = format!("{:2}    {:6}   {}: ", rank + 1, rogue.gold, login_name());
	buf += &ending_string(&ending);
	buf += &format!(" on level {} ", max_level);
	if (!ending.is_win()) && has_amulet() {
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

pub unsafe fn sell_pack()
{
	let mut row: usize = 2;
	let mut obj = rogue.pack.next_object;
	clear();
	mvaddstr(1, 0, "Value      Item");
	while !obj.is_null() {
		if (*obj).what_is != ObjectWhat::Food {
			(*obj).identified = true;
			let val = get_value(&*obj);
			rogue.gold += val;

			if row < DROWS {
				let msg = format!("{:5}      {}", val, get_desc(&*obj));
				mvaddstr(row as i32, 0, &msg);
				row += 1;
			}
		}
		obj = (*obj).next_object;
	}
	refresh();
	if rogue.gold > MAX_GOLD {
		rogue.gold = MAX_GOLD;
	}
	message("", 0);
}

unsafe fn get_value(obj: &obj) -> isize {
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
	return val as isize;
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
	for i in 0..WANDS {
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


pub fn xxxx<const N: usize>(buf: &mut [u8; N], n: usize) {
	for i in 0..n {
		/* It does not matter if accuracy is lost during this assignment */
		let c = xxx(false) as u8;
		buf[i] ^= c;
	}
}

pub fn xxx(reset: bool) -> isize {
	static FS: RwLock<(isize, isize)> = RwLock::new((0, 0));
	if reset {
		let write = FS.write().unwrap();
		*write = (37, 7);
		0
	} else {
		let (f, s) = {
			let read = FS.read().unwrap();
			*read
		};
		let r = (f * s + 9337) % 8887;
		{
			let write = FS.write().unwrap();
			*write = (s, r);
		}
		r
	}
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
