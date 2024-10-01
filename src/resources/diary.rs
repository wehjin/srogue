use crate::pack::wait_for_ack;
use crate::render_system::backend;

#[derive(Debug, Clone)]
pub struct Diary {
	pub message_line: Option<String>,
	pub next_message_line: Option<String>,
	pub current_page: Vec<String>,
	pub previous_page: Vec<String>,
	pub rewound: bool,
	pub stats_changed: bool,
	pub cleaned_up: Option<String>,
	pub interrupted: bool,
	pub hit_message: Option<String>,
}
impl Default for Diary {
	fn default() -> Self {
		Self {
			message_line: None,
			next_message_line: None,
			current_page: vec![],
			previous_page: vec![],
			rewound: false,
			stats_changed: true,
			cleaned_up: None,
			interrupted: false,
			hit_message: None,
		}
	}
}

impl Diary {
	pub fn has_entries(&self) -> bool {
		self.current_page.len() > 0
	}
	pub fn add_entry(&mut self, entry: impl AsRef<str>) {
		panic!("DIARY ADD ENTRY: {}", entry.as_ref());
		assert!(!self.rewound);
		let text = entry.as_ref();
		self.current_page.push(text.to_string());
	}
	pub fn clear_message_lines(&mut self) {
		self.message_line = None;
		self.next_message_line = None;
	}

	pub fn rewind(&mut self) {
		assert!(self.current_page.is_empty());
		self.rewound = true;
	}
	pub fn turn_page(&mut self) {
		if self.current_page.len() > 0 {
			self.previous_page = self.current_page.iter().filter(|&it| it.len() > 0).cloned().collect::<Vec<_>>();
		}
		self.current_page.clear();
		self.rewound = false;
	}

	pub fn set_stats_changed(&mut self, value: bool) {
		self.stats_changed = value;
	}
}

pub fn show_current_page(diary: &Diary) {
	if diary.rewound {
		if let Some(entry) = diary.previous_page.last() {
			backend::set_full_row(entry, DIALOG_ROW);
			backend::push_screen();
		} else {
			backend::set_full_row("", DIALOG_ROW);
			backend::push_screen();
		}
	} else {
		let count = diary.current_page.len();
		if count == 0 {
			backend::set_full_row("", DIALOG_ROW);
			backend::push_screen();
		} else {
			for (i, entry) in diary.current_page.iter().enumerate() {
				let more_to_show = (i + 1) < count;
				let message = if more_to_show {
					format!("{} -more-", entry)
				} else {
					format!("{}", entry)
				};
				backend::set_full_row(message, DIALOG_ROW);
				backend::push_screen();
				if more_to_show {
					wait_for_ack();
				}
			}
		}
	}
}

pub fn show_prompt(prompt: impl AsRef<str>, diary: &mut Diary) {
	if diary.has_entries() {
		diary.add_entry("");
		show_current_page(diary);
		diary.turn_page();
	}
	backend::set_full_row(format!("{}", prompt.as_ref()), DIALOG_ROW);
	backend::push_screen();
}

pub(crate) const DIALOG_ROW: usize = 0;