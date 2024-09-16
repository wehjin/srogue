use std::cmp::Ordering;
use std::fs::File;
use std::io::{Read, Seek, Write};

use crate::init::{clean_up, GameState, BYEBYE_STRING};
use crate::level::constants::{DCOLS, DROWS};
use crate::machdep::{md_heed_signals, md_ignore_signals};
use crate::pack::{has_amulet, unwear, unwield, wait_for_ack};
use crate::player::Player;
use crate::prelude::ending::Ending;
use crate::prelude::object_what::ObjectWhat;
use crate::prelude::*;
use crate::render_system::backend;
use crate::resources::diary;
use crate::resources::keyboard::rgetchar;
use crate::ring::{un_put_hand, PlayerHand};

mod values;


pub const SCORE_FILE: &'static str = "/usr/games/player.rogue.scores";

pub fn killed_by(ending: Ending, game: &mut GameState) {
	md_ignore_signals();
	if !ending.is_quit() {
		game.player.rogue.gold = ((game.player.rogue.gold as f64 * 9.0) / 10.0) as usize;
	}
	backend::erase_screen();
	let mut how_ended = ending_string(&ending);
	how_ended += &format!(" with {} gold", game.player.rogue.gold);
	if ending.is_monster() && game.player.settings.show_skull {
		game.diary.turn_page();
		backend::set_str("__---------__", (4, 32).into());
		backend::set_str("_~             ~_", (5, 30).into());
		backend::set_str("/                 \\", (6, 29).into());
		backend::set_str("~                   ~", (7, 28).into());
		backend::set_str("/                     \\", (8, 27).into());
		backend::set_str("|    XXXX     XXXX    |", (9, 27).into());
		backend::set_str("|    XXXX     XXXX    |", (10, 27).into());
		backend::set_str("|    XXX       XXX    |", (11, 27).into());
		backend::set_str("\\         @         /", (12, 28).into());
		backend::set_str("--\\     @@@     /--", (13, 29).into());
		backend::set_str("| |    @@@    | |", (14, 30).into());
		backend::set_str("| |           | |", (15, 30).into());
		backend::set_str("| vvVvvvvvvvVvv |", (16, 30).into());
		backend::set_str("|  ^^^^^^^^^^^  |", (17, 30).into());
		backend::set_str("\\_           _/", (18, 31).into());
		backend::set_str("~---------~", (19, 33).into());
		center(21, game.player.settings.player_name().as_str());
		center(22, &how_ended);
	} else {
		diary::show_prompt(&how_ended, &mut game.diary);
	}
	wait_for_ack();

	backend::erase_screen();
	put_scores(Some(ending), game);
}

fn ending_string(ending: &Ending) -> String {
	match ending {
		&Ending::Monster(ref name) => {
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

pub fn win(game: &mut GameState) {
	unwield(&mut game.player);          /* disarm and relax */
	unwear(&mut game.player);
	for hand in PlayerHand::ALL_HANDS {
		un_put_hand(hand, game);
	}
	backend::erase_screen();
	backend::set_str("@   @  @@@   @   @      @  @  @   @@@   @   @   @", (10, 11).into());
	backend::set_str(" @ @  @   @  @   @      @  @  @  @   @  @@  @   @", (11, 11).into());
	backend::set_str("  @   @   @  @   @      @  @  @  @   @  @ @ @   @", (12, 11).into());
	backend::set_str("  @   @   @  @   @      @  @  @  @   @  @  @@", (13, 11).into());
	backend::set_str("  @    @@@    @@@        @@ @@    @@@   @   @   @", (14, 11).into());
	backend::set_str("Congratulations,  you have  been admitted  to  the", (17, 11).into());
	backend::set_str("Fighters' Guild.   You return home,  sell all your", (18, 11).into());
	backend::set_str("treasures at great profit and retire into comfort.", (19, 11).into());
	game.diary.add_entry("");
	game.diary.add_entry("");
	game.player.notes.identify_all();
	sell_pack(game);
	diary::show_current_page(&game.diary);
	game.diary.turn_page();
	put_scores(Some(Ending::Win), game);
}

pub fn ask_quit(from_intrpt: bool, game: &mut GameState) -> bool {
	md_ignore_signals();
	let mut orow = 0;
	let mut ocol = 0;
	let mut buf = ['\x00'; 128];
	if from_intrpt {
		orow = game.player.rogue.row;
		ocol = game.player.rogue.col;
		for i in 0..DCOLS {
			buf[i] = backend::get_char((0, i).into());
		}
	}
	game.player.interrupt_and_slurp();
	diary::show_prompt("really quit?", &mut game.diary);
	if rgetchar() != 'y' {
		md_heed_signals();
		if from_intrpt {
			for i in 0..DCOLS {
				backend::set_char(buf[i], (0, i).into());
			}
			backend::move_cursor((orow, ocol).into());
			backend::push_screen();
		}
		return false;
	}
	if from_intrpt {
		clean_up(BYEBYE_STRING, &mut game.player);
		return true;
	}
	killed_by(Ending::Quit, game);
	true
}

pub fn put_scores(ending: Option<Ending>, game: &mut GameState) {
	// TODO turn_into_games();
	let mut file = match File::options().read(true).write(true).open(SCORE_FILE) {
		Ok(file) => file,
		Err(_) => match File::options().write(true).open(SCORE_FILE) {
			Ok(file) => file,
			Err(_) => {
				game.diary.add_entry("cannot read/write/create score file");
				score_file_error(game);
				return;
			}
		},
	};
	// TODO turn_into_user();

	let mut score_only = game.player.settings.score_only;
	let mut scores: Vec<String> = Vec::new();
	let mut n_names: Vec<String> = Vec::new();
	{
		let mut scores_string = String::new();
		if file.read_to_string(&mut scores_string).is_err() {
			score_file_error(game);
			return;
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
			if name_cmp(name_in_score, &game.player.settings.login_name) == Ordering::Equal {
				if let Some(gold_in_score) = gold_in_score(&scores[i]) {
					if game.player.rogue.gold < gold_in_score {
						score_only = true;
					} else {
						found_player = Some(i);
					}
				} else {
					score_file_error(game);
					return;
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
				if game.player.rogue.gold >= gold_in_score {
					rank = i;
					break;
				}
			} else {
				score_file_error(game);
				return;
			}
		}
		if ne == 0 {
			rank = 0;
		} else if (ne < 10) && (rank == 10) {
			rank = ne;
		}
		if rank < 10 {
			let name = match &game.player.settings.nick_name {
				None => "".to_string(),
				Some(name) => name.to_string(),
			};
			insert_score(&mut scores, &mut n_names, &name, rank, ne, ending.expect("ending"), &game.player);
		}
		file.rewind().expect("rewind file");
	}

	backend::erase_screen();
	backend::set_str("Top  Ten  Rogueists", (3, 30).into());
	backend::set_str("Rank   Score   Name", (8, 0).into());

	md_ignore_signals();

	let ne = scores.len().min(n_names.len()).min(10);
	for i in 0..ne {
		let name = &n_names[i];
		let score = &scores[i];
		let revised_rank = format!("{:2}", i + 1) + &score[2..];
		let revised_name = replace_name_in_score(&revised_rank, name);
		if i == rank {
			backend::stand_out(true);
		}
		backend::set_str(&revised_name, (i + 10, 0).into());
		if i == rank {
			backend::stand_out(false);
		}
		if rank < 10 {
			let score_and_name = format!("{}\n{}\n", revised_name, name);
			file.write(score_and_name.as_bytes()).expect("write score and name");
		}
	}
	backend::push_screen();
	drop(file);
	game.diary.add_entry("");
	clean_up("", &mut game.player);
}

fn gold_in_score(score: &str) -> Option<usize> {
	let slice = &score[6..12];
	let trimmed = slice.trim();
	trimmed.parse::<usize>().ok()
}

fn insert_score(
	scores: &mut Vec<String>,
	n_names: &mut Vec<String>,
	n_name: &str,
	rank: usize,
	n: usize,
	ending: Ending,
	player: &Player,
) {
	let mut buf = format!("{:2}    {:6}   {}: ", rank + 1, player.rogue.gold, player.settings.login_name);
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

pub fn sell_pack(game: &mut GameState)
{
	backend::erase_screen();
	backend::set_str("Value      Item", (1, 0).into());
	let mut row: usize = 2;
	for pack_id in game.player.object_ids() {
		if game.player.object_what(pack_id) != ObjectWhat::Food {
			let obj = game.player.object_mut(pack_id).expect("obj in player");
			obj.identified = true;
			let obj_value = obj.sale_value();
			let obj_desc = game.player.get_obj_desc(pack_id);
			game.player.rogue.gold += obj_value;
			if row < DROWS {
				let msg = format!("{:5}      {}", obj_value, obj_desc);
				backend::set_str(&msg, (row, 0).into());
				row += 1;
			}
		}
	}
	backend::push_screen();
	if game.player.rogue.gold > MAX_GOLD {
		game.player.rogue.gold = MAX_GOLD;
	}
	wait_for_ack();
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
	let margin = (DCOLS - msg.len().max(DCOLS)) / 2;
	backend::set_str(msg, (row, margin as i64).into());
}

pub fn score_file_error(game: &mut GameState) {
	game.player.interrupt_and_slurp();
	clean_up("sorry, score file is out of order", &mut game.player);
}
