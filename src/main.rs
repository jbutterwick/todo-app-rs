mod app;
mod item;
mod todo;
mod tui;
mod ui;

use app::App;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use todo::Todo;

fn main() -> std::io::Result<()> {
	let file_path = std::env::args()
		.nth(1)
		.unwrap_or_else(|| String::from("todo.xit"));

	let todo = match std::fs::read_to_string(&file_path) {
		Ok(existing) => Todo::from_existing(&existing, file_path),
		Err(_) => Todo::new(file_path),
	};

	let mut terminal = tui::init()?;
	let result = run(App::new(todo), &mut terminal);
	tui::restore()?; // always restore the terminal, even if the loop errored
	result
}

fn run(mut app: App, terminal: &mut tui::Tui) -> std::io::Result<()> {
	while !app.should_quit {
		terminal.draw(|frame| ui::ui(frame, &mut app))?;
		if let Event::Key(key) = event::read()? {
			if key.kind == KeyEventKind::Press {
				app.handle_key(key);
			}
		}
	}
	Ok(())
}
