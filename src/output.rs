pub enum Color {
	Red,
	Green,
	Yellow,
	Blue,
	Default,
}

pub struct Output<T>
where
	T: Outputtable,
{
	pub kind: String,
	pub value: T,
}

pub struct ColoredString {
	pub color: Color,
	pub string: String,
}

impl ColoredString {
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

pub(crate) trait Outputtable {
	fn show(&self) -> String;
}

impl Outputtable for ColoredString {
	fn show(&self) -> String {
		let mut string = String::new();
		string.push_str("\u{001B}[");
		string.push_str(&*self.color_to_code().to_string());
		string.push('m');
		string.push_str(&*self.string);
		string.push_str("\u{001B}[0m");
		string
	}
}

impl Outputtable for String {
	fn show(&self) -> String {
		String::from(self)
	}
}
