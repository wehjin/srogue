#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]


use std::{env};
use std::fs::File;
use std::io::{Write};
use ncurses::clear;
use crate::prelude::*;
use crate::save::data::SaveData;
use crate::settings::{login_name, save_file};

pub unsafe fn save_game(game: &GameState) {
	let file_name = get_input_line("file name?", save_file().clone(), Some("game not saved"), false, true);
	if file_name.is_empty() {
		return;
	}
	check_message();
	message(&file_name, 0);
	save_into_file(&file_name, game);
}

mod data;

pub unsafe fn save_into_file(save_path: &str, game: &GameState) {
	let save_path = expand_tilde(&save_path);
	let file = File::create(&save_path);
	let mut file = match file {
		Err(_) => {
			message("problem accessing the save file", 0);
			return;
		}
		Ok(file) => {
			file
		}
	};
	let file_id = md_get_file_id(&save_path);
	if file_id == -1 {
		message("problem accessing the save file", 0);
		return;
	}
	md_ignore_signals();
	let save_data = SaveData::read_from_statics(file_id, game);
	let json = serde_json::to_string_pretty(&save_data).expect("serialize data");
	let write_failed = if let Err(_) = file.write(json.as_bytes()) {
		message("write() failed, don't know why", 0);
		sound_bell();
		true
	} else {
		false
	};
	drop(file);
	if write_failed {
		delete_file(&save_path);
	} else {
		clean_up("");
	}
}

fn expand_tilde(file: &str) -> String {
	if file.starts_with('~') {
		if let Ok(home) = env::var("HOME") {
			format!("{}{}", home, &file[1..])
		} else {
			file.to_string()
		}
	} else {
		file.to_string()
	}
}

pub unsafe fn restore(file_path: &str, game: &mut GameState) {
	let new_file_id = md_get_file_id(file_path);
	if new_file_id == -1 {
		clean_up("cannot open file");
		unreachable!("post clean up")
	}
	if md_link_count(file_path) > 1 {
		clean_up("file has link");
		unreachable!("post clean up")
	}
	let save_data = match data::from_file(file_path) {
		Ok(result) => result,
		Err(e) => {
			clean_up(&format!("read failed: {}", e));
			unreachable!("post clean up")
		}
	};
	if save_data.login_name != login_name() {
		clean_up("you're not the original player");
		unreachable!("post clean up")
	}
	if new_file_id != save_data.file_id {
		clean_up("sorry, saved game is not in the same file");
		unreachable!("post clean up")
	}
	match get_file_modification_time(file_path) {
		Ok(mod_time) => {
			if has_been_touched(&save_data.saved_time, &mod_time) {
				clear();
				clean_up("sorry, file has been touched");
				unreachable!("post clean up");
			}
		}
		Err(_) => {
			clean_up("sorry, no modification time");
			unreachable!("post clean up");
		}
	}
	save_data.write_to_statics();
	game.depth = save_data.depth;
	game.level = save_data.level;

	if !save_data.wizard && !delete_file(file_path) {
		clean_up("cannot delete file");
		unreachable!("post clean up");
	}

	msg_cleared = false;
	ring_stats(false, game.depth.cur, &game.level);
}

fn has_been_touched(saved_time: &RogueTime, mod_time: &RogueTime) -> bool {
	if mod_time.year > (saved_time.year) {
		true
	} else if mod_time.year < saved_time.year {
		false
	} else if mod_time.month > (saved_time.month) {
		true
	} else if mod_time.month < saved_time.month {
		false
	} else if mod_time.day > (saved_time.day) {
		true
	} else if mod_time.day < saved_time.day {
		false
	} else if mod_time.hour > (saved_time.hour) {
		true
	} else if mod_time.hour < saved_time.hour {
		false
	} else if mod_time.minute > (saved_time.minute) {
		true
	} else if mod_time.minute < saved_time.minute {
		false
	} else if mod_time.second > (saved_time.second) {
		true
	} else {
		false
	}
}
