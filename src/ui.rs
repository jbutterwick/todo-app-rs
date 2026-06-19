use crate::app::{App, Mode, HELP_TEXT};
use crate::item::{Item, Status};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn ui(frame: &mut Frame, app: &mut App) {
	let [list_area, status_area] =
		Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());

	let vis = app.visible_indices();
	let rows: Vec<ListItem> = vis
		.iter()
		.map(|&i| ListItem::new(item_line(&app.todo.item_vec[i])))
		.collect();

	let title = match &app.filter {
		None => format!(" {} ", app.todo.file_path),
		Some(s) => format!(" {} — filter: {} ", app.todo.file_path, s.as_str().trim()),
	};
	let list = List::new(rows)
		.block(Block::default().borders(Borders::ALL).title(title))
		.highlight_style(Style::default().add_modifier(Modifier::REVERSED));
	frame.render_stateful_widget(list, list_area, &mut app.list_state);

	frame.render_widget(
		Paragraph::new(app.status_msg.as_str()).style(status_style(&app.status_msg)),
		status_area,
	);

	match app.mode {
		Mode::Add => popup(frame, "Add item", &app.input),
		Mode::Edit => popup(frame, "Edit", &app.input),
		Mode::DueDate => popup(frame, "Due date (YYYY-MM-DD, empty clears)", &app.input),
		Mode::Help => popup(frame, "Help — any key closes", HELP_TEXT),
		Mode::ConfirmRemove => popup(frame, "Remove?", "y = yes, any other key = no"),
		Mode::Normal => {}
	}
}

fn status_style(msg: &str) -> Style {
	if msg.starts_with("failed") || msg.starts_with("couldn't") {
		Style::default().fg(Color::Red)
	} else {
		Style::default().fg(Color::DarkGray)
	}
}

/// Render an item as a styled line, reusing the same prefix/suffix the file uses.
fn item_line(item: &Item) -> Line<'static> {
	let prefix = item.state.as_str();
	let (prio, date) = item.suffix();
	let (prefix_color, desc_style) = match item.state {
		Status::Open => (Color::Blue, Style::default()),
		Status::InQuestion => (Color::Yellow, Style::default()),
		Status::Ongoing => (Color::Magenta, Style::default()),
		Status::Checked => (
			Color::Green,
			Style::default()
				.fg(Color::Gray)
				.add_modifier(Modifier::CROSSED_OUT),
		),
		Status::Obsolete => (Color::DarkGray, Style::default().fg(Color::DarkGray)),
	};
	Line::from(vec![
		Span::styled(prefix.to_string(), Style::default().fg(prefix_color)),
		Span::raw(prio),
		Span::styled(item.description.clone(), desc_style),
		Span::raw(date),
	])
}

fn popup(frame: &mut Frame, title: &str, body: &str) {
	let area = centered_rect(60, 30, frame.area());
	frame.render_widget(Clear, area);
	frame.render_widget(
		Paragraph::new(body).block(Block::default().borders(Borders::ALL).title(format!(" {title} "))),
		area,
	);
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
}
