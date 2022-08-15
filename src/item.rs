use crate::output::*;

#[derive(PartialEq)]
pub enum State {
	Todo,
	Done,
}

pub struct ItemList {
	pub(crate) items: Vec<Item>,
}

pub struct Item {
	pub description: String,
	pub state: State,
}

pub struct Line<'a> {
	index: i32,
	string: &'a String,
	suffix: Option<&'a str>,
}

impl Item {
	fn to_line(&self, index: i32) -> Vec<Line> {
		let mut string = String::from(&self.description);
		vec![Line {
			index: index + 1,
			string: &ColoredString {
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
			}
			.show(),
			suffix: None,
		}]
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
