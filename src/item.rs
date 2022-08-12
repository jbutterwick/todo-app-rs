use crate::output::*;

#[derive(PartialEq)]
pub enum State {
	Todo,
	Done,
}

pub struct Item {
	description: String,
	state: State,
}

impl Item {
	fn to_line(&self, index: i32) -> Vec<(i32, ColoredString, Option<&str>)> {
		let mut string = String::from(&self.description);
		vec![(
			index + 1,
			ColoredString {
				color: if self.state == State::Done {
					Color::Blue
				} else {
					Color::Green
				},
				string: if self.state == State::Done {
					string.push_str("(done)");
					string
				} else {
					string
				},
			},
			None,
		)]
	}
}

impl From<&str> for Item {
	fn from(string: &str) -> Self {
		Item {
			state: State::Todo,
			description: String::from(string),
		}
	}
}
