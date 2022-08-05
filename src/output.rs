pub enum Color {
	Red,
	Green,
	Yellow,
	Blue,
	Default,
}

pub struct ColoredString<'a> {
	pub color: Color,
	pub string: &'a str,
}

impl ColoredString<'_> {
	fn show(&self) -> String {
		let mut string = String::new();
		string.push_str("\u{001B}[");
		string.push_str(&*self.color_to_code().to_string());
		string.push('m');
		string.push_str(&*self.string);
		string.push_str("\u{001B}[0m");
		string
	}

	fn color_to_code(&self) -> i32 {
		match self.color {
			Color::Red => 31,
			Color::Green => 32,
			Color::Yellow => 33,
			Color::Blue => 34,
			Color::Default => 0,
		}
	}
}
