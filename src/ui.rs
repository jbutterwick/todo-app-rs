use crate::app::{App, DayEntry, Mode, HELP_TEXT};
use crate::item::{Item, Status};
use crate::recur::rule::Freq;
use crate::recur::series::RecurringSeries;
use crate::theme::Theme;
use chrono::NaiveDate;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &mut App) {
	let theme = app.theme().clone(); // owned so we can still borrow app mutably below

	// paint the whole frame in the theme's base colours first
	frame.render_widget(
		Block::default().style(Style::default().bg(theme.bg).fg(theme.fg)),
		frame.area(),
	);

	let [main_area, status_area] =
		Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

	if app.background_is_calendar() {
		render_calendar(frame, app, main_area, &theme);
	} else {
		render_list(frame, app, main_area, &theme);
	}

	frame.render_widget(
		Paragraph::new(app.status_msg.as_str()).style(status_style(&app.status_msg, &theme)),
		status_area,
	);

	// text-entry popups also place the terminal cursor after the input
	let input_area = match app.mode {
		Mode::Add => Some(popup(frame, "Add item — Tab: make repeating", &app.input, &theme)),
		Mode::Edit => Some(popup(frame, "Edit", &app.input, &theme)),
		Mode::DatePicker => {
			render_date_picker(frame, app, &theme);
			None
		}
		Mode::Help => {
			popup(frame, "Help — any key closes", HELP_TEXT, &theme);
			None
		}
		Mode::ConfirmRemove => {
			popup(frame, "Remove?", "y = yes, any other key = no", &theme);
			None
		}
		Mode::Theme => {
			render_theme_menu(frame, app, &theme);
			None
		}
		Mode::Recurrence => {
			render_recur_builder(frame, app, &theme);
			None
		}
		Mode::RecurRemove => {
			popup(
				frame,
				"Remove recurring",
				"s = skip this · f = this & future · d = whole series · Esc cancel",
				&theme,
			);
			None
		}
		Mode::Normal | Mode::Calendar => None,
	};
	if let Some(area) = input_area {
		let cursor_x = (area.x + 1 + app.input.chars().count() as u16)
			.min(area.x + area.width.saturating_sub(2));
		frame.set_cursor_position((cursor_x, area.y + 1));
	}
}

/// A bordered block in the theme's colours.
fn themed_block(theme: &Theme, title: String) -> Block<'static> {
	Block::default()
		.borders(Borders::ALL)
		.border_style(Style::default().fg(theme.muted))
		.title(Span::styled(title, Style::default().fg(theme.accent)))
		.style(Style::default().bg(theme.bg).fg(theme.fg))
}

fn highlight(theme: &Theme) -> Style {
	Style::default()
		.bg(theme.accent)
		.fg(theme.bg)
		.add_modifier(Modifier::BOLD)
}

fn render_list(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
	let rows: Vec<ListItem> = app
		.visible_indices()
		.iter()
		.map(|&i| ListItem::new(item_line(&app.todo.item_vec[i], theme)))
		.collect();

	let title = match &app.filter {
		None => format!(" Undated — {} ", app.todo.file_path),
		Some(s) => format!(" Undated — {} · filter: {} ", app.todo.file_path, s.as_str().trim()),
	};
	let list = List::new(rows)
		.block(themed_block(theme, title))
		.highlight_style(highlight(theme));
	frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_calendar(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
	let [cal_area, due_area] =
		Layout::horizontal([Constraint::Length(24), Constraint::Min(20)]).areas(area);

	// calendar grid (left): month name in the border, weekday header inside
	let block = themed_block(theme, format!(" {} ", app.cursor.format("%B %Y")));
	let inner = block.inner(cal_area);
	frame.render_widget(block, cal_area);

	let mut events = CalendarEventStore::default();
	let has_item = Style::default().fg(theme.marker).add_modifier(Modifier::BOLD);
	for item in &app.todo.item_vec {
		if let Some(td) = item.due_date.and_then(to_time_date) {
			events.add(td, has_item);
		}
	}
	// mark recurring occurrences across the visible month
	let (m_lo, m_hi) = month_bounds(app.cursor);
	for s in &app.store.series {
		for d in s.occurrences_in(m_lo..=m_hi) {
			if let Some(td) = to_time_date(d) {
				events.add(td, has_item);
			}
		}
	}
	// cursor styled last so it wins even on a day that also has items
	if let Some(cur) = to_time_date(app.cursor) {
		events.add(cur, highlight(theme));
		let calendar =
			Monthly::new(cur, &events).show_weekdays_header(Style::default().fg(theme.muted));
		frame.render_widget(calendar, inner);
	}

	// entries on the cursor date (right): concrete items + recurring occurrences
	let entries = app.entries_on(app.cursor);
	let rows: Vec<ListItem> = entries
		.iter()
		.map(|e| ListItem::new(entry_line(app, e, theme)))
		.collect();
	let due_list = List::new(rows)
		.block(themed_block(theme, format!(" Due {} ", app.cursor.format("%Y-%m-%d"))))
		.highlight_style(highlight(theme));
	let mut state = ListState::default();
	if !entries.is_empty() {
		state.select(Some(app.due_selected.min(entries.len() - 1)));
	}
	frame.render_stateful_widget(due_list, due_area, &mut state);
}

fn month_bounds(date: NaiveDate) -> (NaiveDate, NaiveDate) {
	use chrono::Datelike;
	let lo = date.with_day(1).unwrap_or(date);
	let hi = if date.month() == 12 {
		NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
	} else {
		NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
	}
	.and_then(|d| d.pred_opt())
	.unwrap_or(date);
	(lo, hi)
}

/// Render a calendar day row (concrete item or recurring occurrence).
fn entry_line(app: &App, entry: &DayEntry, theme: &Theme) -> Line<'static> {
	match entry {
		DayEntry::Item(i) => item_line(&app.todo.item_vec[*i], theme),
		DayEntry::Recurring { series_id, date } => match app.store.get(*series_id) {
			Some(s) => recurring_line(s, *date, theme),
			None => Line::from(""),
		},
	}
}

/// A recurring occurrence rendered like an item, using the series' effective
/// view for this date (frozen snapshot if completed, else the live series title).
fn recurring_line(series: &RecurringSeries, date: NaiveDate, theme: &Theme) -> Line<'static> {
	let (title, prio, state) = series.view(date);
	let prefix = state.as_str();
	let prio_str = if prio > 0 {
		format!(" {} ", "!".repeat(prio as usize))
	} else {
		String::from(" ")
	};
	let (prefix_color, desc_style) = match state {
		Status::Open => (theme.open, Style::default()),
		Status::InQuestion => (theme.question, Style::default()),
		Status::Ongoing => (theme.ongoing, Style::default()),
		Status::Checked => (
			theme.checked,
			Style::default()
				.fg(theme.muted)
				.add_modifier(Modifier::CROSSED_OUT),
		),
		Status::Obsolete => (theme.obsolete, Style::default().fg(theme.muted)),
	};
	Line::from(vec![
		Span::styled(prefix.to_string(), Style::default().fg(prefix_color)),
		Span::raw(prio_str),
		Span::styled(title, desc_style),
		Span::styled(" ⟳", Style::default().fg(theme.muted)),
	])
}

fn render_theme_menu(frame: &mut Frame, app: &App, theme: &Theme) {
	let area = centered_rect(40, 60, frame.area());
	frame.render_widget(Clear, area);
	let items: Vec<ListItem> = app
		.themes
		.iter()
		.map(|t| ListItem::new(t.name.clone()))
		.collect();
	let list = List::new(items)
		.block(themed_block(theme, " Theme ".into()))
		.highlight_style(highlight(theme));
	let mut state = ListState::default();
	state.select(Some(app.theme_idx));
	frame.render_stateful_widget(list, area, &mut state);
}

/// Recurrence builder: guided fields plus a raw RRULE field, with a live preview.
fn render_recur_builder(frame: &mut Frame, app: &App, theme: &Theme) {
	let Some(b) = app.builder.as_ref() else {
		return;
	};
	let area = centered_fixed(52, 13, frame.area());
	frame.render_widget(Clear, area);
	let title = if b.edit.is_some() {
		" Edit repeat — applies this & future "
	} else {
		" New repeating todo "
	};
	let block = themed_block(theme, title.into());
	let inner = block.inner(area);
	frame.render_widget(block, area);

	let freq = match b.freq {
		Freq::Daily => "daily",
		Freq::Weekly => "weekly",
		Freq::Monthly => "monthly",
		Freq::Yearly => "yearly",
	};
	let field = |i: usize, label: &str, val: String| -> Line<'static> {
		let focused = b.field == i;
		let marker = if focused { "› " } else { "  " };
		let style = if focused {
			Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
		} else {
			Style::default().fg(theme.fg)
		};
		Line::styled(format!("{marker}{label:<10}{val}"), style)
	};
	let preview = b.to_rule().map_or_else(|e| e, |r| r.to_rrule());

	// text fields show their real value when focused (so the caret aligns), else a placeholder
	let until_val = if b.until.is_empty() && b.field != 4 {
		"—".into()
	} else {
		b.until.clone()
	};
	let raw_val = if b.raw.is_empty() && b.field != 5 {
		"(advanced, optional)".into()
	} else {
		b.raw.clone()
	};
	let mut lines = vec![
		field(0, "title:", b.title.clone()),
		field(1, "frequency:", freq.to_string()),
		field(2, "interval:", b.interval.to_string()),
		field(3, "weekday:", format!("{:?} (weekly only)", b.weekday)),
		field(4, "until:", until_val),
		field(5, "rrule:", raw_val),
		Line::raw(""),
		Line::styled(format!("  preview: {preview}"), Style::default().fg(theme.muted)),
		Line::styled(
			"  ↑↓/Tab move · ←→ change · Enter save · Esc cancel",
			Style::default().fg(theme.muted),
		),
	];
	if !b.error.is_empty() {
		lines.push(Line::styled(format!("  {}", b.error), Style::default().fg(theme.error)));
	}
	frame.render_widget(
		Paragraph::new(lines).style(Style::default().bg(theme.bg).fg(theme.fg)),
		inner,
	);

	// place the terminal cursor in the focused text field ("›·" + label padded to 10 = 12 cols)
	let text_len = match b.field {
		0 => Some(b.title.chars().count()),
		4 => Some(b.until.chars().count()),
		5 => Some(b.raw.chars().count()),
		_ => None,
	};
	if let Some(len) = text_len {
		let x = (inner.x + 12 + len as u16).min(inner.x + inner.width.saturating_sub(1));
		frame.set_cursor_position((x, inner.y + b.field as u16));
	}
}

/// Date picker: a calendar grid plus a text field. `Tab` switches focus.
fn render_date_picker(frame: &mut Frame, app: &App, theme: &Theme) {
	let area = centered_fixed(34, 12, frame.area());
	frame.render_widget(Clear, area);
	let block = themed_block(theme, " Due date — Tab: calendar/text · Enter set ".into());
	let inner = block.inner(area);
	frame.render_widget(block, area);

	let [cal_area, text_area] =
		Layout::vertical([Constraint::Min(7), Constraint::Length(1)]).areas(inner);

	// calendar grid, with existing due dates marked and the pick cursor highlighted
	let mut events = CalendarEventStore::default();
	let marked = Style::default().fg(theme.marker);
	for item in &app.todo.item_vec {
		if let Some(td) = item.due_date.and_then(to_time_date) {
			events.add(td, marked);
		}
	}
	if let Some(cur) = to_time_date(app.pick_cursor) {
		let cursor_style = if app.pick_text_focus {
			Style::default().fg(theme.accent).add_modifier(Modifier::UNDERLINED)
		} else {
			highlight(theme)
		};
		events.add(cur, cursor_style);
		let calendar = Monthly::new(cur, &events)
			.show_month_header(Style::default().fg(theme.accent))
			.show_weekdays_header(Style::default().fg(theme.muted));
		frame.render_widget(calendar, cal_area);
	}

	// text field (focused indicator + caret)
	let label = if app.pick_text_focus { "› type: " } else { "  type: " };
	frame.render_widget(
		Paragraph::new(format!("{label}{}", app.input)).style(Style::default().fg(theme.fg).bg(theme.bg)),
		text_area,
	);
	if app.pick_text_focus {
		let x = text_area.x + (label.chars().count() + app.input.chars().count()) as u16;
		frame.set_cursor_position((x.min(text_area.x + text_area.width.saturating_sub(1)), text_area.y));
	}
}

fn centered_fixed(w: u16, h: u16, area: Rect) -> Rect {
	Rect {
		x: area.x + area.width.saturating_sub(w) / 2,
		y: area.y + area.height.saturating_sub(h) / 2,
		width: w.min(area.width),
		height: h.min(area.height),
	}
}

fn to_time_date(d: NaiveDate) -> Option<time::Date> {
	use chrono::Datelike;
	let month = time::Month::try_from(d.month() as u8).ok()?;
	time::Date::from_calendar_date(d.year(), month, d.day() as u8).ok()
}

fn status_style(msg: &str, theme: &Theme) -> Style {
	let fg = if msg.starts_with("failed") || msg.starts_with("couldn't") {
		theme.error
	} else {
		theme.muted
	};
	Style::default().fg(fg).bg(theme.bg)
}

/// Render an item as a styled line, reusing the same prefix/suffix the file uses.
fn item_line(item: &Item, theme: &Theme) -> Line<'static> {
	let prefix = item.state.as_str();
	let (prio, date) = item.suffix();
	let (prefix_color, desc_style) = match item.state {
		Status::Open => (theme.open, Style::default()),
		Status::InQuestion => (theme.question, Style::default()),
		Status::Ongoing => (theme.ongoing, Style::default()),
		Status::Checked => (
			theme.checked,
			Style::default()
				.fg(theme.muted)
				.add_modifier(Modifier::CROSSED_OUT),
		),
		Status::Obsolete => (theme.obsolete, Style::default().fg(theme.muted)),
	};
	Line::from(vec![
		Span::styled(prefix.to_string(), Style::default().fg(prefix_color)),
		Span::raw(prio),
		Span::styled(item.description.clone(), desc_style),
		Span::raw(date),
	])
}

fn popup(frame: &mut Frame, title: &str, body: &str, theme: &Theme) -> Rect {
	let area = centered_rect(60, 30, frame.area());
	frame.render_widget(Clear, area);
	frame.render_widget(
		Paragraph::new(body).block(themed_block(theme, format!(" {title} "))),
		area,
	);
	area
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
	let [_, mid, _] = Layout::vertical([
		Constraint::Percentage((100 - percent_y) / 2),
		Constraint::Percentage(percent_y),
		Constraint::Percentage((100 - percent_y) / 2),
	])
	.areas(area);
	let [_, center, _] = Layout::horizontal([
		Constraint::Percentage((100 - percent_x) / 2),
		Constraint::Percentage(percent_x),
		Constraint::Percentage((100 - percent_x) / 2),
	])
	.areas(mid);
	center
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::app::Mode;
	use crate::todo::Todo;
	use ratatui::backend::TestBackend;
	use ratatui::Terminal;

	fn render(app: &mut App) -> String {
		let mut terminal = Terminal::new(TestBackend::new(50, 12)).unwrap();
		terminal.draw(|f| ui(f, app)).unwrap();
		terminal
			.backend()
			.buffer()
			.content()
			.iter()
			.map(|c| c.symbol())
			.collect()
	}

	#[test]
	fn renders_items() {
		let todo = Todo::from_existing("[ ]  fix dates\n[@] !! ship\n", "t.xit".into());
		let mut app = App::new(todo);
		app.mode = Mode::Normal; // the undated list view (calendar is the default)
		let out = render(&mut app);
		assert!(out.contains("fix dates"));
		assert!(out.contains("ship"));
	}

	#[test]
	fn popup_renders_without_panicking() {
		let todo = Todo::from_existing("[ ]  fix dates\n", "t.xit".into());
		let mut app = App::new(todo);
		app.mode = Mode::Add;
		app.input = "new task".into();
		let out = render(&mut app);
		assert!(out.contains("Add item"));
		assert!(out.contains("new task"));
	}

	#[test]
	fn calendar_renders_month_and_due_items() {
		let todo = Todo::from_existing("[@]  !! ship release -> 2026-06-19\n", "t.xit".into());
		let mut app = App::new(todo);
		app.mode = Mode::Calendar;
		app.cursor = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();
		let out = render(&mut app);
		assert!(out.contains("June 2026"), "month header missing: {out:?}");
		assert!(out.contains("Su"), "weekday header missing");
		assert!(out.contains("ship release"), "due item missing");
	}

	#[test]
	fn calendar_shows_recurring_occurrence() {
		use crate::recur::rule::Recurrence;
		let mut app = App::new(Todo::new("t.xit".into()));
		let anchor = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
		app.store.add(
			"Water plants".into(),
			0,
			Recurrence::from_rrule("FREQ=WEEKLY;BYDAY=MO").unwrap(),
			anchor,
		);
		app.mode = Mode::Calendar;
		app.cursor = NaiveDate::from_ymd_opt(2026, 6, 8).unwrap(); // a Monday
		let out = render(&mut app);
		assert!(out.contains("Water plants"), "recurring occurrence missing: {out:?}");
	}

	#[test]
	fn add_popup_opened_from_calendar_keeps_calendar_behind() {
		use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
		let todo = Todo::from_existing("[@]  !! ship -> 2026-06-19\n", "t.xit".into());
		let mut app = App::new(todo); // calendar is the default view
		app.cursor = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();
		let press = |app: &mut App, c| app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
		press(&mut app, 'a'); // open Add over the calendar
		let out = render(&mut app);
		assert!(out.contains("June 2026"), "calendar should remain behind: {out:?}");
		assert!(out.contains("Add item"), "add popup missing");
	}

	#[test]
	fn date_picker_renders_calendar_and_text() {
		let todo = Todo::from_existing("[ ]  task\n", "t.xit".into());
		let mut app = App::new(todo);
		app.mode = Mode::DatePicker;
		app.pick_cursor = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();
		app.input = "2026-06-".into();
		let out = render(&mut app);
		assert!(out.contains("June 2026"), "calendar grid missing: {out:?}");
		assert!(out.contains("type:"), "text field missing");
		assert!(out.contains("2026-06-"), "typed text missing");
	}

	#[test]
	fn theme_menu_lists_theme_names() {
		let mut app = App::new(Todo::new("t.xit".into()));
		app.mode = Mode::Theme;
		let out = render(&mut app);
		assert!(out.contains("Theme"), "menu title missing");
		assert!(out.contains("Catppuccin Mocha"), "theme names missing: {out:?}");
	}
}
