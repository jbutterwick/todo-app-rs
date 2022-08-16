use crate::item::*;
use crate::response::*;
use std::fmt::{Display, Formatter};

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
	pub kind: ResponseType,
	pub value: T,
}

impl Display for Output<String> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.value)
	}
}

pub struct ColoredString {
	pub color: Color,
	pub string: String,
}

impl From<&Color> for i32 {
	fn from(color: &Color) -> Self {
		match color {
			Color::Red => 31,
			Color::Green => 32,
			Color::Yellow => 33,
			Color::Blue => 34,
			Color::Default => 0,
		}
	}
}

impl From<State> for Color {
	fn from(state: State) -> Self {
		match state {
			State::Todo => Color::Blue,
			State::Done => Color::Green,
		}
	}
}

impl From<String> for ColoredString {
	fn from(str: String) -> Self {
		ColoredString {
			color: Color::Blue,
			string: str,
		}
	}
}

impl From<Item> for ColoredString {
	fn from(item: Item) -> Self {
		ColoredString {
			color: Color::from(item.state),
			string: item.description,
		}
	}
}

impl From<ColoredString> for String {
	fn from(c_str: ColoredString) -> Self {
		c_str.show()
	}
}

impl From<ItemList> for String {
	fn from(item_list: ItemList) -> Self {
		let mut string = String::new();
		for x in item_list.items {
			string.push_str(&*ColoredString::from(x).show());
			string.push("\n".parse().unwrap())
		}
		string
	}
}

pub trait Outputtable {
	fn show(&self) -> String;
}

impl Outputtable for ColoredString {
	fn show(&self) -> String {
		let mut string = String::new();
		string.push_str("\u{001B}[");
		string.push_str(&i32::from(&self.color).to_string());
		string.push('m');
		string.push_str(&*self.string);
		string.push_str("\u{001B}[0m");
		string
	}
}

impl Outputtable for Vec<ColoredString> {
	fn show(&self) -> String {
		let mut string = String::new();
		for x in self {
			string.push_str(&x.show())
		}
		string
	}
}

impl Outputtable for String {
	fn show(&self) -> String {
		String::from(self)
	}
}
