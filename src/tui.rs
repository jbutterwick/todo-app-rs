use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;
use std::io::{stdout, Stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> std::io::Result<Tui> {
	enable_raw_mode()?;
	execute!(stdout(), EnterAlternateScreen)?;
	set_panic_hook();
	Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> std::io::Result<()> {
	execute!(stdout(), LeaveAlternateScreen)?;
	disable_raw_mode()
}

fn set_panic_hook() {
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |info| {
		let _ = restore(); // best-effort terminal cleanup before the panic prints
		hook(info);
	}));
}
