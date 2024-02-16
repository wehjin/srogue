use std::fs::File;
use std::io::Read;

use libc::c_int;
use ncurses::{clrtoeol, mv, mvaddstr, mvinch, refresh};

use crate::init::GameState;
use crate::level::constants::{DCOLS, DROWS};
use crate::message::rgetchar;

static INSTRUCTIONS_FILE: &'static str = "/usr/games/player.rogue.instr";

#[no_mangle]
pub unsafe extern "C" fn Instructions(game: &mut GameState) {
	let mut file = match File::open(INSTRUCTIONS_FILE) {
		Ok(file) => file,
		Err(_) => {
			game.dialog.message("Help file not on line.", 0);
			return;
		}
	};
	let mut rows = Vec::new();
	for row in 0..DROWS {
		// Read the rows in out of the window.
		let mut chars = Vec::new();
		for col in 0..DCOLS {
			let ch = mvinch(row as c_int, col as c_int);
			chars.push(ch as u8);
		}
		// Rewrite the chars into the same r
		let string = String::from_utf8(chars).expect("valid utf8");
		mvaddstr(row as c_int, 0, &string);
		clrtoeol();
		rows.push(string);
	}
	// This is weird...why clear the rows right after we rewrote the chars into each row?
	mv(0, 0);
	for row in 0..DROWS {
		mv(row as i32, 0);
		clrtoeol();
	}
	refresh();
	for row in 0..DROWS {
		let mut buf = [0u8; 256];
		if file.read_exact(&mut buf).is_err() {
			break;
		}
		let first_line = if let Some(index) = buf.iter().position(|x| *x == '\n' as u8) {
			&buf[0..index]
		} else {
			&buf[..]
		}.to_vec();
		let first_line = String::from_utf8(first_line).expect("valid utf8");
		mv(row as i32, 0);
		clrtoeol();
		mvaddstr(row as i32, 0, &first_line);
	}
	refresh();
	rgetchar();
	mv(0, 0);
	clrtoeol();
	for row in 0..DROWS {
		mv(row as i32, 0);
		clrtoeol();
	}
	refresh();
	for row in 0..DROWS {
		mvaddstr(row as i32, 0, &rows[row]);
	}
	refresh();
}
