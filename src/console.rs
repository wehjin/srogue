use std::sync::RwLock;
use crate::machdep::md_control_keybord;

static UP: RwLock<bool> = RwLock::new(false);


pub fn is_up() -> bool { *UP.read().unwrap() }

pub fn up() {
	if !is_up() {
		ncurses::cbreak();
		ncurses::noecho();
		ncurses::nonl();
		md_control_keybord(0);
		*UP.write().unwrap() = true;
	}
}

pub fn down() {
	if is_up() {
		ncurses::endwin();
		md_control_keybord(1);
		*UP.write().unwrap() = false;
	}
}

