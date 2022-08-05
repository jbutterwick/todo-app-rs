use crate::output::*;

pub struct Item {
	description: String,
	state: State, // Rust doesn't let us use string literals as types so we need to define an Enum
}

impl Item {
	fn to_line(&self, index: i32) -> Vec<(i32, ColoredString, Option<&str>)> {
		vec![(
			index + 1,
			ColoredString {
				color: Color::Blue,
				string: &self.description,
			},
			None,
		)]
	}
}

pub enum State {
	Todo,
	Done,
}
