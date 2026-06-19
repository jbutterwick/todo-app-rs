use crate::item::{Item, Status};
use crate::theme::{load_themes, Theme};
use crate::todo::Todo;
use chrono::{Days, Local, Months, NaiveDate};
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
 c              calendar      t       theme
 h / F1         this help     q / Esc quit (saves)";

pub const CALENDAR_HINT: &str =
	"←→ day · ↑↓ week · [ ] month · j/k item · a/e/x/o/@/~/i/>/</D/r edit · c/Esc back";

#[derive(Clone, Copy)]
pub enum Mode {
	Normal,
	Add,
	Edit,
	DatePicker,
	Help,
	ConfirmRemove,
	Calendar,
	Theme,
}

pub struct App {
	pub todo: Todo,
	pub mode: Mode,
	pub list_state: ListState,
	pub input: String,
	pub filter: Option<Status>,
	pub status_msg: String,
	pub should_quit: bool,
	pub cursor: NaiveDate,    // calendar day cursor
	pub due_selected: usize,  // selection within the calendar's due-items list
	pub themes: Vec<Theme>,   // available themes (>= 1)
	pub theme_idx: usize,     // currently applied theme
	pub pick_cursor: NaiveDate, // date-picker calendar cursor
	pub pick_text_focus: bool, // date picker: text field focused (vs calendar)
	popup_return: Mode,       // mode to restore when a popup closes
	target: Option<usize>,    // real item index an Edit/DatePicker popup acts on
	add_due: Option<NaiveDate>, // due date stamped onto a newly added item
	theme_saved: usize,       // theme to revert to if the picker is cancelled
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
			cursor: Local::now().date_naive(),
			due_selected: 0,
			themes: load_themes(),
			theme_idx: 0,
			pick_cursor: Local::now().date_naive(),
			pick_text_focus: true,
			popup_return: Mode::Normal,
			target: None,
			add_due: None,
			theme_saved: 0,
		}
	}

	pub fn theme(&self) -> &Theme {
		&self.themes[self.theme_idx]
	}

	/// True when the calendar is the active view behind any open popup, so a
	/// popup opened from the calendar keeps the calendar (not the list) behind it.
	pub fn background_is_calendar(&self) -> bool {
		let view = if self.is_overlay() {
			self.popup_return
		} else {
			self.mode
		};
		matches!(view, Mode::Calendar)
	}

	fn is_overlay(&self) -> bool {
		matches!(
			self.mode,
			Mode::Add
				| Mode::Edit
				| Mode::DatePicker
				| Mode::Help
				| Mode::ConfirmRemove
				| Mode::Theme
		)
	}

	pub fn items_due_on(&self, date: NaiveDate) -> Vec<usize> {
		self.todo
			.item_vec
			.iter()
			.enumerate()
			.filter(|(_, it)| it.due_date == Some(date))
			.map(|(i, _)| i)
			.collect()
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
			Mode::Add | Mode::Edit => self.handle_text_input(key),
			Mode::DatePicker => self.handle_datepicker(key),
			Mode::Help => self.mode = self.popup_return, // any key closes, back to origin
			Mode::ConfirmRemove => self.handle_confirm(key),
			Mode::Calendar => self.handle_calendar(key),
			Mode::Theme => self.handle_theme(key),
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
			KeyCode::Char('c') => {
				self.mode = Mode::Calendar;
				self.due_selected = 0;
				self.status_msg = String::from(CALENDAR_HINT);
			}
			KeyCode::Char('t') => {
				self.theme_saved = self.theme_idx;
				self.mode = Mode::Theme;
				self.status_msg = String::from("j/k preview · Enter keep · Esc cancel");
			}
			KeyCode::Char('a') | KeyCode::Char('+') => self.open_add(None),
			KeyCode::Char('e') => {
				if let Some(i) = self.selected_real() {
					self.open_edit(i);
				}
			}
			KeyCode::Char('D') => {
				if let Some(i) = self.selected_real() {
					self.open_due_date(i);
				}
			}
			KeyCode::Char('r') | KeyCode::Char('-') => {
				if let Some(i) = self.selected_real() {
					self.open_confirm_remove(i);
				}
			}
			KeyCode::Char('h') | KeyCode::F(1) => self.open_help(),
			_ => {}
		}
	}

	fn handle_text_input(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Esc => self.close_popup(),
			KeyCode::Backspace => {
				self.input.pop();
			}
			KeyCode::Char(c) => self.input.push(c),
			KeyCode::Enter => self.commit_input(),
			_ => {}
		}
	}

	fn handle_confirm(&mut self, key: KeyEvent) {
		if matches!(key.code, KeyCode::Char('y')) {
			if let Some(i) = self.target {
				self.todo.remove(i);
				self.clamp_selection();
				self.clamp_due();
				self.save();
			}
		}
		self.target = None;
		self.mode = self.popup_return;
	}

	fn handle_calendar(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Char('q') => self.quit(),
			KeyCode::Esc | KeyCode::Char('c') => {
				self.mode = Mode::Normal;
				self.status_msg = String::from("`h` for help");
			}
			// arrow keys move the calendar day cursor
			KeyCode::Left => self.shift_days(-1),
			KeyCode::Right => self.shift_days(1),
			KeyCode::Up => self.shift_days(-7),
			KeyCode::Down => self.shift_days(7),
			KeyCode::Char('[') => self.shift_months(false),
			KeyCode::Char(']') => self.shift_months(true),
			// j/k move the selection within the due-items list
			KeyCode::Char('j') => self.move_due(1),
			KeyCode::Char('k') => self.move_due(-1),
			// add a todo due on the cursor date
			KeyCode::Char('a') | KeyCode::Char('+') => self.open_add(Some(self.cursor)),
			// everything below acts on the selected due item — same as the list view
			KeyCode::Char('e') => {
				if let Some(i) = self.due_real() {
					self.open_edit(i);
				}
			}
			KeyCode::Char('D') => {
				if let Some(i) = self.due_real() {
					self.open_due_date(i);
				}
			}
			KeyCode::Char('r') | KeyCode::Char('-') => {
				if let Some(i) = self.due_real() {
					self.open_confirm_remove(i);
				}
			}
			KeyCode::Char('x') | KeyCode::Char(' ') => self.mutate_due(|t, i| t.toggle_done(i)),
			KeyCode::Char('o') => self.mutate_due(|t, i| t.set_status(i, Status::Open)),
			KeyCode::Char('@') => self.mutate_due(|t, i| t.set_status(i, Status::Ongoing)),
			KeyCode::Char('~') => self.mutate_due(|t, i| t.set_status(i, Status::Obsolete)),
			KeyCode::Char('i') => self.mutate_due(|t, i| t.set_status(i, Status::InQuestion)),
			KeyCode::Char('>') => self.mutate_due(|t, i| t.adjust_priority(i, 1)),
			KeyCode::Char('<') => self.mutate_due(|t, i| t.adjust_priority(i, -1)),
			KeyCode::Char('s') => {
				self.todo.sort();
				self.clamp_due();
				self.save();
			}
			KeyCode::Char('h') | KeyCode::F(1) => self.open_help(),
			_ => {}
		}
	}

	fn due_real(&self) -> Option<usize> {
		self.items_due_on(self.cursor).get(self.due_selected).copied()
	}

	/// Mutate the selected due item, then clamp the due selection + save.
	fn mutate_due(&mut self, f: impl FnOnce(&mut Todo, usize)) {
		if let Some(i) = self.due_real() {
			f(&mut self.todo, i);
			self.clamp_due();
			self.save();
		}
	}

	fn handle_theme(&mut self, key: KeyEvent) {
		let n = self.themes.len();
		match key.code {
			// scrolling previews live — rendering always reads the current theme
			KeyCode::Char('j') | KeyCode::Down => self.theme_idx = (self.theme_idx + 1) % n,
			KeyCode::Char('k') | KeyCode::Up => self.theme_idx = (self.theme_idx + n - 1) % n,
			KeyCode::Enter => {
				let name = self.theme().name.clone();
				self.status_msg = format!("theme: {name}");
				self.mode = Mode::Normal;
			}
			KeyCode::Esc | KeyCode::Char('q') => {
				self.theme_idx = self.theme_saved; // revert the preview
				self.status_msg = String::from("`h` for help");
				self.mode = Mode::Normal;
			}
			_ => {}
		}
	}

	fn shift_days(&mut self, n: i64) {
		self.cursor = add_days(self.cursor, n);
		self.due_selected = 0; // the due-list changed
	}

	fn shift_months(&mut self, forward: bool) {
		self.cursor = add_months(self.cursor, forward);
		self.due_selected = 0;
	}

	/// Date picker: navigate the calendar grid, type a date, or Tab between them.
	fn handle_datepicker(&mut self, key: KeyEvent) {
		match key.code {
			KeyCode::Esc => self.close_popup(),
			KeyCode::Tab => self.pick_text_focus = !self.pick_text_focus,
			KeyCode::Enter => self.commit_datepicker(),
			// when typing, keys edit the text field
			KeyCode::Backspace if self.pick_text_focus => {
				self.input.pop();
			}
			KeyCode::Char(c) if self.pick_text_focus => self.input.push(c),
			// arrows/brackets drive the calendar cursor (and focus it, so Enter picks it)
			KeyCode::Left => self.pick_to(add_days(self.pick_cursor, -1)),
			KeyCode::Right => self.pick_to(add_days(self.pick_cursor, 1)),
			KeyCode::Up => self.pick_to(add_days(self.pick_cursor, -7)),
			KeyCode::Down => self.pick_to(add_days(self.pick_cursor, 7)),
			KeyCode::Char('[') => self.pick_to(add_months(self.pick_cursor, false)),
			KeyCode::Char(']') => self.pick_to(add_months(self.pick_cursor, true)),
			_ => {}
		}
	}

	/// Move the picker cursor and focus the calendar — navigating the grid means
	/// you intend to pick that day, so Enter commits it rather than the text field.
	fn pick_to(&mut self, date: NaiveDate) {
		self.pick_cursor = date;
		self.pick_text_focus = false;
	}

	fn commit_datepicker(&mut self) {
		let Some(i) = self.target else {
			self.close_popup();
			return;
		};
		if self.pick_text_focus {
			let text = self.input.trim().to_string();
			if text.is_empty() {
				self.todo.set_due_date(i, None); // empty clears the date
			} else if let Some(d) = Item::parse_dates(&text) {
				self.todo.set_due_date(i, Some(d));
			} else {
				self.status_msg = format!("couldn't parse date: {text}");
				return; // stay open so the user can fix it
			}
		} else {
			self.todo.set_due_date(i, Some(self.pick_cursor)); // calendar pick
		}
		self.save();
		self.close_popup();
	}

	fn move_due(&mut self, delta: isize) {
		let len = self.items_due_on(self.cursor).len();
		if len == 0 {
			return;
		}
		let next = (self.due_selected as isize + delta).clamp(0, len as isize - 1);
		self.due_selected = next as usize;
	}

	fn clamp_due(&mut self) {
		let len = self.items_due_on(self.cursor).len();
		self.due_selected = self.due_selected.min(len.saturating_sub(1));
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

	/// Open a popup, remembering the mode to return to when it closes.
	fn open_add(&mut self, due: Option<NaiveDate>) {
		self.add_due = due;
		self.input.clear();
		self.popup_return = self.mode;
		self.mode = Mode::Add;
	}

	fn open_edit(&mut self, target: usize) {
		self.target = Some(target);
		self.input = self.todo.item_vec[target].description.clone();
		self.popup_return = self.mode;
		self.mode = Mode::Edit;
	}

	fn open_due_date(&mut self, target: usize) {
		self.target = Some(target);
		self.input.clear();
		self.pick_cursor = self.todo.item_vec[target].due_date.unwrap_or(self.cursor);
		self.pick_text_focus = true; // start typing; Tab to use the calendar
		self.popup_return = self.mode;
		self.mode = Mode::DatePicker;
	}

	fn open_confirm_remove(&mut self, target: usize) {
		self.target = Some(target);
		self.popup_return = self.mode;
		self.mode = Mode::ConfirmRemove;
	}

	fn open_help(&mut self) {
		self.popup_return = self.mode; // so closing returns to the list or calendar
		self.mode = Mode::Help;
	}

	fn close_popup(&mut self) {
		self.input.clear();
		self.add_due = None;
		self.target = None;
		self.mode = self.popup_return;
		self.clamp_due();
	}

	fn commit_input(&mut self) {
		let text = self.input.trim().to_string();
		match self.mode {
			Mode::Add if !text.is_empty() => {
				self.todo.add(text);
				if let Some(d) = self.add_due {
					self.todo.set_due_date(self.todo.item_vec.len() - 1, Some(d));
				}
				self.clamp_selection();
				self.save();
			}
			Mode::Edit => {
				if let (Some(i), false) = (self.target, text.is_empty()) {
					self.todo.edit(i, text);
					self.save();
				}
			}
			_ => {}
		}
		self.close_popup();
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

fn add_days(date: NaiveDate, n: i64) -> NaiveDate {
	let shifted = if n >= 0 {
		date.checked_add_days(Days::new(n as u64))
	} else {
		date.checked_sub_days(Days::new(n.unsigned_abs()))
	};
	shifted.unwrap_or(date)
}

fn add_months(date: NaiveDate, forward: bool) -> NaiveDate {
	let shifted = if forward {
		date.checked_add_months(Months::new(1))
	} else {
		date.checked_sub_months(Months::new(1))
	};
	shifted.unwrap_or(date)
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

	fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
		NaiveDate::from_ymd_opt(y, m, d).unwrap()
	}

	#[test]
	fn calendar_cursor_navigation() {
		let mut app = empty_app();
		app.cursor = ymd(2026, 6, 19);
		press(&mut app, KeyCode::Char('c'));
		assert!(matches!(app.mode, Mode::Calendar));
		press(&mut app, KeyCode::Right); // +1 day
		assert_eq!(app.cursor, ymd(2026, 6, 20));
		press(&mut app, KeyCode::Down); // +1 week
		assert_eq!(app.cursor, ymd(2026, 6, 27));
		press(&mut app, KeyCode::Up); // -1 week
		assert_eq!(app.cursor, ymd(2026, 6, 20));
		press(&mut app, KeyCode::Char(']')); // next month (day clamps)
		assert_eq!(app.cursor, ymd(2026, 7, 20));
		press(&mut app, KeyCode::Char('[')); // prev month
		assert_eq!(app.cursor, ymd(2026, 6, 20));
		press(&mut app, KeyCode::Esc); // back to list
		assert!(matches!(app.mode, Mode::Normal));
	}

	#[test]
	fn items_due_on_filters_by_date() {
		let mut app = empty_app();
		app.todo.add("has date".into());
		app.todo.set_due_date(0, Some(ymd(2026, 6, 19)));
		app.todo.add("no date".into());
		assert_eq!(app.items_due_on(ymd(2026, 6, 19)), vec![0]);
		assert!(app.items_due_on(ymd(2026, 6, 20)).is_empty());
	}

	#[test]
	fn calendar_add_stamps_cursor_date() {
		let mut app = empty_app();
		app.cursor = ymd(2026, 6, 19);
		press(&mut app, KeyCode::Char('c')); // enter calendar
		press(&mut app, KeyCode::Char('a')); // add, due on cursor
		for c in "dentist".chars() {
			app.handle_key(key(c));
		}
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.todo.item_vec[0].description, "dentist");
		assert_eq!(app.todo.item_vec[0].due_date, Some(ymd(2026, 6, 19)));
		assert!(matches!(app.mode, Mode::Calendar)); // back in the calendar
	}

	#[test]
	fn calendar_edit_targets_selected_due_item() {
		let mut app = empty_app();
		app.cursor = ymd(2026, 6, 19);
		app.todo.add("a".into());
		app.todo.set_due_date(0, Some(ymd(2026, 6, 19)));
		app.todo.add("b".into());
		app.todo.set_due_date(1, Some(ymd(2026, 6, 19)));
		press(&mut app, KeyCode::Char('c'));
		press(&mut app, KeyCode::Char('j')); // select 2nd due item
		assert_eq!(app.due_selected, 1);
		press(&mut app, KeyCode::Char('e'));
		assert_eq!(app.input, "b"); // prefilled
		press(&mut app, KeyCode::Backspace);
		for c in "bravo".chars() {
			app.handle_key(key(c));
		}
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.todo.item_vec[1].description, "bravo");
		assert_eq!(app.todo.item_vec[0].description, "a"); // sibling untouched
		assert!(matches!(app.mode, Mode::Calendar));
	}

	#[test]
	fn help_opens_with_h_from_list_and_calendar() {
		let mut app = empty_app();
		// from the list
		app.handle_key(key('h'));
		assert!(matches!(app.mode, Mode::Help));
		app.handle_key(key('x')); // any key closes
		assert!(matches!(app.mode, Mode::Normal));
		// from the calendar — and it returns to the calendar, not the list
		press(&mut app, KeyCode::Char('c'));
		app.handle_key(key('h'));
		assert!(matches!(app.mode, Mode::Help));
		app.handle_key(key('x'));
		assert!(matches!(app.mode, Mode::Calendar));
	}

	fn two_due_calendar() -> App {
		let mut app = empty_app();
		app.cursor = ymd(2026, 6, 19);
		app.todo.add("a".into());
		app.todo.set_due_date(0, Some(ymd(2026, 6, 19)));
		app.todo.add("b".into());
		app.todo.set_due_date(1, Some(ymd(2026, 6, 19)));
		press(&mut app, KeyCode::Char('c'));
		app
	}

	#[test]
	fn calendar_status_and_priority_edits() {
		let mut app = two_due_calendar();
		app.handle_key(key('@')); // first -> ongoing
		assert_eq!(app.todo.item_vec[0].state, Status::Ongoing);
		app.handle_key(key('>')); // first priority +1
		assert_eq!(app.todo.item_vec[0].priority, 1);
		press(&mut app, KeyCode::Char('j')); // select 2nd
		app.handle_key(key('x')); // toggle done
		assert_eq!(app.todo.item_vec[1].state, Status::Checked);
		assert!(matches!(app.mode, Mode::Calendar)); // never left the calendar
	}

	#[test]
	fn calendar_remove_confirms_and_returns() {
		let mut app = two_due_calendar();
		press(&mut app, KeyCode::Char('r'));
		assert!(matches!(app.mode, Mode::ConfirmRemove));
		app.handle_key(key('y'));
		assert_eq!(app.todo.item_vec.len(), 1);
		assert_eq!(app.todo.item_vec[0].description, "b"); // removed "a"
		assert!(matches!(app.mode, Mode::Calendar)); // back to calendar, not Normal
	}

	#[test]
	fn calendar_due_date_change_moves_item_off_cursor() {
		let mut app = two_due_calendar();
		press(&mut app, KeyCode::Char('D'));
		for c in "2026-07-01".chars() {
			app.handle_key(key(c));
		}
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.todo.item_vec[0].due_date, Some(ymd(2026, 7, 1)));
		assert_eq!(app.items_due_on(app.cursor).len(), 1); // only "b" remains on cursor
		assert_eq!(app.due_selected, 0); // clamped
		assert!(matches!(app.mode, Mode::Calendar));
	}

	#[test]
	fn datepicker_calendar_pick_sets_date() {
		let mut app = App::new(Todo::from_existing("[ ]  task\n", "/tmp/__x.xit".into()));
		app.cursor = ymd(2026, 6, 19);
		// open picker on the selected item
		press(&mut app, KeyCode::Char('D'));
		assert!(matches!(app.mode, Mode::DatePicker));
		assert_eq!(app.pick_cursor, ymd(2026, 6, 19)); // started at the view cursor
		press(&mut app, KeyCode::Tab); // switch to calendar focus
		assert!(!app.pick_text_focus);
		press(&mut app, KeyCode::Right); // +1 day on the grid
		press(&mut app, KeyCode::Down); // +1 week
		press(&mut app, KeyCode::Enter); // commit the cursor date
		assert_eq!(app.todo.item_vec[0].due_date, Some(ymd(2026, 6, 27)));
		assert!(matches!(app.mode, Mode::Normal));
	}

	#[test]
	fn datepicker_scroll_then_enter_saves_without_tab() {
		// reproduces the reported bug: open picker, scroll the grid, Enter -> saves
		let mut app = App::new(Todo::from_existing("[ ]  task\n", "/tmp/__x.xit".into()));
		app.cursor = ymd(2026, 6, 19);
		press(&mut app, KeyCode::Char('D'));
		assert!(app.pick_text_focus); // starts in text mode
		press(&mut app, KeyCode::Right); // scrolling the grid switches focus to it
		assert!(!app.pick_text_focus);
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.todo.item_vec[0].due_date, Some(ymd(2026, 6, 20))); // saved, not cleared
	}

	#[test]
	fn datepicker_text_still_works_and_clears() {
		let mut app = App::new(Todo::from_existing("[ ]  task\n", "/tmp/__x.xit".into()));
		app.todo.set_due_date(0, Some(ymd(2026, 1, 1)));
		// type a new date (text focus is the default)
		press(&mut app, KeyCode::Char('D'));
		for c in "2026-07-01".chars() {
			app.handle_key(key(c));
		}
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.todo.item_vec[0].due_date, Some(ymd(2026, 7, 1)));
		// reopen, submit empty -> clears
		press(&mut app, KeyCode::Char('D'));
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.todo.item_vec[0].due_date, None);
	}

	#[test]
	fn theme_picker_previews_then_reverts_or_keeps() {
		let mut app = empty_app();
		assert!(app.themes.len() >= 2, "need >1 theme to test");
		let start = app.theme_idx;
		// open picker, preview the next theme, cancel -> reverts
		press(&mut app, KeyCode::Char('t'));
		assert!(matches!(app.mode, Mode::Theme));
		press(&mut app, KeyCode::Down);
		assert_eq!(app.theme_idx, start + 1); // live preview applied
		press(&mut app, KeyCode::Esc);
		assert_eq!(app.theme_idx, start); // reverted
		assert!(matches!(app.mode, Mode::Normal));
		// open again, preview, keep with Enter
		press(&mut app, KeyCode::Char('t'));
		press(&mut app, KeyCode::Down);
		press(&mut app, KeyCode::Enter);
		assert_eq!(app.theme_idx, start + 1);
		assert!(matches!(app.mode, Mode::Normal));
	}
}
