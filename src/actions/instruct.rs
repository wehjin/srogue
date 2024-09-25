pub fn instruction_lines() -> Vec<String> {
	CONTENTS.split('\n').map(|line| line.to_string()).collect()
}

const CONTENTS: &'static str = include_str!("assets/rogue.instr");
