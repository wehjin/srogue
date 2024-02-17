#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals)]


use std::env;
use std::fs::File;
use std::io::Write;

use ncurses::clear;

use crate::init::{clean_up, GameState};
use crate::machdep::{delete_file, get_file_modification_time, md_get_file_id, md_ignore_signals, md_link_count, RogueTime};
use crate::message::{get_input_line, sound_bell};
use crate::ring::ring_stats;
use crate::save::data::SaveData;

pub fn save_game(game: &mut GameState) -> bool {
	let save_file = game.player.settings.save_file.clone();
	let cancellation_prompt = Some("game not saved");
	let file_name = get_input_line("file name?", save_file, cancellation_prompt, false, true, game);
	if file_name.is_empty() {
		return false;
	}
	game.dialog.clear_message();
	game.dialog.message(&file_name, 0);
	return save_into_file(&file_name, game);
}

mod data;

fn save_into_file(save_path: &str, game: &mut GameState) -> bool {
	let save_path = expand_tilde(&save_path);
	let file = File::create(&save_path);
	let mut file = match file {
		Err(_) => {
			game.dialog.message("problem accessing the save file", 0);
			return false;
		}
		Ok(file) => {
			file
		}
	};
	let file_id = md_get_file_id(&save_path);
	let file_id = match file_id {
		Ok(id) => id,
		Err(_) => {
			game.dialog.message("problem accessing the save file", 0);
			return false;
		}
	};
	md_ignore_signals();
	let save_data = SaveData::read_from_statics(file_id, game);
	let json = serde_json::to_string_pretty(&save_data).expect("serialize data");
	let write_failed = if let Err(_) = file.write(json.as_bytes()) {
		game.dialog.message("write() failed, don't know why", 0);
		sound_bell();
		true
	} else {
		false
	};
	drop(file);
	return if write_failed {
		delete_file(&save_path);
		false
	} else {
		clean_up("", &mut game.player);
		true
	};
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

pub fn restore(file_path: &str, game: &mut GameState) -> bool {
	let cur_login_name = game.player.settings.login_name.clone();
	let new_file_id = md_get_file_id(file_path);
	let new_file_id = match new_file_id {
		Ok(id) => id,
		Err(_) => {
			clean_up("cannot open file", &mut game.player);
			return false;
		}
	};
	if md_link_count(file_path) > 1 {
		clean_up("file has link", &mut game.player);
		return false;
	}
	let save_data = match data::from_file(file_path) {
		Ok(result) => result,
		Err(e) => {
			clean_up(&format!("read failed: {}", e), &mut game.player);
			return false;
		}
	};
	if save_data.player.settings.login_name != cur_login_name {
		clean_up("you're not the original player", &mut game.player);
		return false;
	}
	if new_file_id != save_data.file_id {
		clean_up("sorry, saved game is not in the same file", &mut game.player);
		return false;
	}
	match get_file_modification_time(file_path) {
		Ok(mod_time) => {
			if has_been_touched(&save_data.saved_time, &mod_time) {
				clear();
				clean_up("sorry, file has been touched", &mut game.player);
				return false;
			}
		}
		Err(_) => {
			clean_up("sorry, no modification time", &mut game.player);
			return false;
		}
	}
	save_data.write_to_statics(game);

	if !game.player.wizard && !delete_file(file_path) {
		clean_up("cannot delete file", &mut game.player);
		return false;
	}

	ring_stats(false, game);
	return true;
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
