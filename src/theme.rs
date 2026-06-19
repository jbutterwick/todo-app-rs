use ratatui::style::Color;
use serde::Deserialize;

/// Colour roles used across the UI. Deserialized from themes.toml.
#[derive(Clone, Deserialize)]
pub struct Theme {
	pub name: String,
	pub bg: Color,
	pub fg: Color,
	pub open: Color,
	pub ongoing: Color,
	pub checked: Color,
	pub obsolete: Color,
	pub question: Color,
	pub accent: Color,
	pub error: Color,
	pub muted: Color,
	pub marker: Color,
}

#[derive(Deserialize)]
struct ThemeFile {
	theme: Vec<Theme>,
}

impl Default for Theme {
	/// Built-in fallback using the terminal's own palette.
	fn default() -> Self {
		Self {
			name: String::from("Default"),
			bg: Color::Reset,
			fg: Color::Reset,
			open: Color::Blue,
			ongoing: Color::Magenta,
			checked: Color::Green,
			obsolete: Color::DarkGray,
			question: Color::Yellow,
			accent: Color::Cyan,
			error: Color::Red,
			muted: Color::DarkGray,
			marker: Color::Yellow,
		}
	}
}

/// Load themes from ./themes.toml if present (so users can edit live), else the
/// copy embedded at build time. Always returns at least the default theme.
pub fn load_themes() -> Vec<Theme> {
	let text = std::fs::read_to_string("themes.toml")
		.unwrap_or_else(|_| include_str!("../themes.toml").to_string());
	match toml::from_str::<ThemeFile>(&text) {
		Ok(f) if !f.theme.is_empty() => f.theme,
		_ => vec![Theme::default()],
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bundled_themes_parse() {
		let themes = load_themes();
		assert!(themes.len() >= 10, "expected the bootstrap set, got {}", themes.len());
		assert!(themes.iter().any(|t| t.name == "Catppuccin Mocha"));
		assert!(themes.iter().any(|t| t.name == "Monokai"));
	}

	#[test]
	fn hex_colours_decode_to_rgb() {
		let themes = load_themes();
		let mocha = themes.iter().find(|t| t.name == "Catppuccin Mocha").unwrap();
		assert_eq!(mocha.bg, Color::Rgb(0x1e, 0x1e, 0x2e));
		assert_eq!(mocha.open, Color::Rgb(0x89, 0xb4, 0xfa));
	}
}
