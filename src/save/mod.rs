use std::env;
use std::fs::File;
use std::io::Write;

use crate::init::{clean_up, GameState};
use crate::machdep::{delete_file, get_file_modification_time, md_get_file_id, md_ignore_signals, md_link_count, RogueTime};
use crate::message::sound_bell;
use crate::resources::input_line::get_input_line;
use crate::ring::ring_stats;
use crate::save::data::SaveData;

pub fn save_game(game: &mut GameState) -> bool {
	let save_file = game.player.settings.save_file.clone();
	let cancellation_prompt = Some("game not saved");
	let file_name = get_input_line("file name?", save_file, cancellation_prompt, false, true, &mut game.diary);
	if file_name.is_empty() {
		return false;
	}
	game.diary.add_entry(&file_name);
	save_into_file(&file_name, game)
}

mod data;

fn save_into_file(save_path: &str, game: &mut GameState) -> bool {
	let save_path = expand_tilde(&save_path);
	let file = File::create(&save_path);
	let mut file = match file {
		Err(_) => {
			game.diary.add_entry("problem accessing the save file");
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
			game.diary.add_entry("problem accessing the save file");
			return false;
		}
	};
	md_ignore_signals();
	let save_data = SaveData::read_from_statics(file_id, game);
	let json = serde_json::to_string_pretty(&save_data).expect("serialize data");
	let write_failed = if let Err(_) = file.write(json.as_bytes()) {
		game.diary.add_entry("write() failed, don't know why");
		sound_bell();
		true
	} else {
		false
	};
	drop(file);
	if write_failed {
		delete_file(&save_path);
		false
	} else {
		clean_up("", game);
		true
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

pub fn restore(file_path: &str, game: &mut GameState) -> bool {
	let cur_login_name = game.player.settings.login_name.clone();
	let new_file_id = md_get_file_id(file_path);
	let new_file_id = match new_file_id {
		Ok(id) => id,
		Err(_) => {
			clean_up("cannot open file", game);
			return false;
		}
	};
	match md_link_count(file_path) {
		Ok(count) if count == 1 => (),
		_ => {
			clean_up("file has link", game);
			return false;
		}
	}
	let save_data = match data::from_file(file_path) {
		Ok(result) => result,
		Err(e) => {
			clean_up(&format!("read failed: {}", e), game);
			return false;
		}
	};
	if save_data.player.settings.login_name != cur_login_name {
		clean_up("you're not the original player", game);
		return false;
	}
	if new_file_id != save_data.file_id {
		clean_up("sorry, saved game is not in the same file", game);
		return false;
	}
	match get_file_modification_time(file_path) {
		Ok(mod_time) => {
			if has_been_touched(&save_data.saved_time, &mod_time) {
				clean_up("sorry, file has been touched", game);
				return false;
			}
		}
		Err(_) => {
			clean_up("sorry, no modification time", game);
			return false;
		}
	}
	save_data.write_to_statics(game);

	if !game.player.wizard && !delete_file(file_path) {
		clean_up("cannot delete file", game);
		return false;
	}

	ring_stats(false, game);
	true
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
