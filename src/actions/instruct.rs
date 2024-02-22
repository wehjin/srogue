use crate::actions::PlayerAction;
use crate::init::GameState;
use crate::render_system::{detect_all_rows, render_all_rows};
use crate::resources::keyboard::rgetchar;

pub struct Instruct;

impl PlayerAction for Instruct {
	fn update(_input_key: char, _game: &mut GameState) {
		let repair_rows = detect_all_rows();
		render_all_rows(instruct_line_for_row);
		rgetchar();
		render_all_rows(|row| {
			if row < repair_rows.len() { repair_rows[row].as_str() } else { "" }
		});
	}
}

fn instruct_line_for_row(row: usize) -> &'static str {
	let lines: Vec<&str> = CONTENTS.split('\n').collect();
	let line = if row < lines.len() {
		lines[row]
	} else {
		""
	};
	line
}

const CONTENTS: &'static str = include_str!("assets/rogue.instr");
