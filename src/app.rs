use crate::item::{Item, Status};
use crate::todo::Todo;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

pub const HELP_TEXT: &str = "\
 j / k  ↓ / ↑   move          a / +   add
 g / G          top / bottom  e       edit
 x / Space      toggle done   D       due date
 o              undo (open)   > / <   priority +/-
 @              ongoing       r / -   remove
 ~              obsolete      s       sort
 i              question      f       cycle filter
 h / F1         this help     q / Esc quit (saves)";

pub enum Mode {
	Normal,
	Add,
	Edit,
	DueDate,
	Help,
	ConfirmRemove,
}

pub struct App {
	pub todo: Todo,
	pub mode: Mode,
	pub list_state: ListState,
	pub input: String,
	pub filter: Option<Status>,
	pub status_msg: String,
	pub should_quit: bool,
}

impl App {
	pub fn new(todo: Todo) -> Self {
		let mut list_state = ListState::default();
		if !todo.item_vec.is_empty() {
			list_state.select(Some(0));
		}
		Self {
			todo,
			mode: Mode::Normal,
			list_state,
			input: String::new(),
			filter: None,
			status_msg: String::from("`h` for help"),
			should_quit: false,
		}
	}

	// ---- filter <-> real index mapping (item_vec stays canonical) ----

	pub fn visible_indices(&self) -> Vec<usize> {
		self.todo
			.item_vec
			.iter()
			.enumerate()
			.filter(|(_, it)| self.filter.as_ref().is_none_or(|f| &it.state == f))
			.map(|(i, _)| i)
			.collect()
	}

	fn selected_real(&self) -> Option<usize> {
		let vis = self.visible_indices();
		self.list_state.selected().and_then(|i| vis.get(i).copied())
	}

	fn clamp_selection(&mut self) {
		let len = self.visible_indices().len();
		if len == 0 {
			self.list_state.select(None);
		} else {
			let i = self.list_state.selected().unwrap_or(0).min(len - 1);
			self.list_state.select(Some(i));
		}
	}

	fn save(&mut self) {
		if let Err(e) = self.todo.save_to_file() {
			self.status_msg = e;
		}
	}

	// ---- navigation ----

	fn move_by(&mut self, delta: isize) {
		let len = self.visible_indices().len();
		if len == 0 {
			return;
		}
		let cur = self.list_state.selected().unwrap_or(0) as isize;
		let next = (cur + delta).clamp(0, len as isize - 1);
		self.list_state.select(Some(next as usize));
	}

	fn jump(&mut self, to_end: bool) {
		let len = self.visible_indices().len();
		if len == 0 {
			return;
		}
		self.list_state.select(Some(if to_end { len - 1 } else { 0 }));
	}

	// ---- key routing ----

	pub fn handle_key(&mut self, key: KeyEvent) {
		if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
			self.quit();
			return;
		}
		match self.mode {
			Mode::Normal => self.handle_normal(key),
			Mode::Add | Mode::Edit | Mode::DueDate => self.handle_text_input(key),
			Mode::Help => self.mode = Mode::Normal,
			Mode::ConfirmRemove => self.handle_confirm(key),
		}
	}

	fn handle_normal(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Char('q') | KeyCode::Esc => self.quit(),
			KeyCode::Char('j') | KeyCode::Down => self.move_by(1),
			KeyCode::Char('k') | KeyCode::Up => self.move_by(-1),
			KeyCode::Char('g') | KeyCode::Home => self.jump(false),
			KeyCode::Char('G') | KeyCode::End => self.jump(true),
			KeyCode::Char('x') | KeyCode::Char(' ') => self.mutate(|t, i| t.toggle_done(i)),
			KeyCode::Char('o') => self.mutate(|t, i| t.set_status(i, Status::Open)),
			KeyCode::Char('@') => self.mutate(|t, i| t.set_status(i, Status::Ongoing)),
			KeyCode::Char('~') => self.mutate(|t, i| t.set_status(i, Status::Obsolete)),
			KeyCode::Char('i') => self.mutate(|t, i| t.set_status(i, Status::InQuestion)),
			KeyCode::Char('>') => self.mutate(|t, i| t.adjust_priority(i, 1)),
			KeyCode::Char('<') => self.mutate(|t, i| t.adjust_priority(i, -1)),
			KeyCode::Char('s') => {
				self.todo.sort();
				self.clamp_selection();
				self.save();
			}
			KeyCode::Char('f') => self.cycle_filter(),
			KeyCode::Char('a') | KeyCode::Char('+') => self.open_input(Mode::Add, String::new()),
			KeyCode::Char('e') => {
				if let Some(i) = self.selected_real() {
					let desc = self.todo.item_vec[i].description.clone();
					self.open_input(Mode::Edit, desc);
				}
			}
			KeyCode::Char('D') => self.open_input(Mode::DueDate, String::new()),
			KeyCode::Char('r') | KeyCode::Char('-') if self.selected_real().is_some() => {
				self.mode = Mode::ConfirmRemove;
			}
			KeyCode::Char('h') | KeyCode::F(1) => self.mode = Mode::Help,
			_ => {}
		}
	}

	fn handle_text_input(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Esc => {
				self.input.clear();
				self.mode = Mode::Normal;
			}
			KeyCode::Backspace => {
				self.input.pop();
			}
			KeyCode::Char(c) => self.input.push(c),
			KeyCode::Enter => self.commit_input(),
			_ => {}
		}
	}

	fn handle_confirm(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Char('y') => {
				self.mutate(|t, i| t.remove(i));
				self.mode = Mode::Normal;
			}
			_ => self.mode = Mode::Normal,
		}
	}

	// ---- actions ----

	/// Run an index-based mutation on the selected item, then clamp + save.
	fn mutate(&mut self, f: impl FnOnce(&mut Todo, usize)) {
		if let Some(i) = self.selected_real() {
			f(&mut self.todo, i);
			self.clamp_selection();
			self.save();
		}
	}

	fn open_input(&mut self, mode: Mode, prefill: String) {
		self.input = prefill;
		self.mode = mode;
	}

	fn commit_input(&mut self) {
		let text = self.input.trim().to_string();
		match self.mode {
			Mode::Add if !text.is_empty() => {
				self.todo.add(text);
				self.clamp_selection();
				self.save();
			}
			Mode::Edit => {
				if let (Some(i), false) = (self.selected_real(), text.is_empty()) {
					self.todo.edit(i, text);
					self.save();
				}
			}
			Mode::DueDate => {
				if let Some(i) = self.selected_real() {
					if text.is_empty() {
						self.todo.set_due_date(i, None);
						self.save();
					} else if let Some(d) = Item::parse_dates(&text) {
						self.todo.set_due_date(i, Some(d));
						self.save();
					} else {
						self.status_msg = format!("couldn't parse date: {text}");
					}
				}
			}
			_ => {}
		}
		self.input.clear();
		self.mode = Mode::Normal;
	}

	fn cycle_filter(&mut self) {
		self.filter = match self.filter {
			None => Some(Status::Open),
			Some(Status::Open) => Some(Status::Ongoing),
			Some(Status::Ongoing) => Some(Status::InQuestion),
			Some(Status::InQuestion) => Some(Status::Checked),
			Some(Status::Checked) => Some(Status::Obsolete),
			Some(Status::Obsolete) => None,
		};
		self.clamp_selection();
	}

	fn quit(&mut self) {
		self.save();
		let _ = self.todo.export_to_md();
		self.should_quit = true;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn key(c: char) -> KeyEvent {
		KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
	}

	fn press(app: &mut App, code: KeyCode) {
		app.handle_key(KeyEvent::new(code, KeyModifiers::NONE));
	}

	fn empty_app() -> App {
		// non-existent path so save() is a no-op-ish; we only assert in-memory state
		App::new(Todo::new("/tmp/__todo_test_unused.xit".into()))
	}

	#[test]
	fn add_popup_commits_item() {
		let mut app = empty_app();
		press(&mut app, KeyCode::Char('a')); // open Add
		for c in "buy milk".chars() {
			app.handle_key(key(c));
		}
		press(&mut app, KeyCode::Enter); // commit
		assert_eq!(app.todo.item_vec.len(), 1);
		assert_eq!(app.todo.item_vec[0].description, "buy milk");
		assert!(matches!(app.mode, Mode::Normal));
	}

	#[test]
	fn esc_cancels_add() {
		let mut app = empty_app();
		press(&mut app, KeyCode::Char('a'));
		app.handle_key(key('x'));
		press(&mut app, KeyCode::Esc);
		assert!(app.todo.item_vec.is_empty());
	}

	#[test]
	fn status_keys_and_filter_mapping() {
		let mut app = App::new(Todo::from_existing("[ ]  a\n[ ]  b\n", "/tmp/__x.xit".into()));
		// select second item, mark ongoing
		press(&mut app, KeyCode::Char('j'));
		app.handle_key(key('@'));
		assert_eq!(app.todo.item_vec[1].state, Status::Ongoing);
		// filter to Ongoing -> only item b visible, selection maps to real index 1
		app.handle_key(key('f')); // None -> Open
		assert_eq!(app.filter, Some(Status::Open));
		assert_eq!(app.visible_indices(), vec![0]);
		app.handle_key(key('f')); // Open -> Ongoing
		assert_eq!(app.visible_indices(), vec![1]);
	}

	#[test]
	fn due_date_popup_sets_date() {
		let mut app = App::new(Todo::from_existing("[ ]  a\n", "/tmp/__x.xit".into()));
		press(&mut app, KeyCode::Char('D'));
		for c in "2026-07-01".chars() {
			app.handle_key(key(c));
		}
		press(&mut app, KeyCode::Enter);
		assert_eq!(
			app.todo.item_vec[0].due_date,
			chrono::NaiveDate::from_ymd_opt(2026, 7, 1)
		);
	}
}
