use crate::output::*;
use std::{
	cmp::Ordering,
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
};

#[derive(PartialEq)]
pub enum State {
	Todo,
	Done,
}

pub struct Item {
	pub description: String,
	pub state: State,
	pub hash: u64,
}

pub struct Line {
	index: usize,
	string: String,
	suffix: Option<String>,
}

impl Item {
	pub fn new(description: String) -> Self {
		let mut s = DefaultHasher::new();
		description.hash(&mut s);
		Item {
			description,
			state: State::Todo,
			hash: s.finish(),
		}
	}
	pub fn to_line(&self, index: usize) -> Line {
		let mut string = String::from(&self.description);
		Line {
			index: index + 1,
			string: ColoredString {
				color: if self.state == State::Done {
					Color::Blue
				} else {
					Color::Green
				},
				string: if self.state == State::Done {
					string.push_str(" - (done)");
					string
				} else {
					string
				},
			}
			.show(),
			suffix: None,
		}
	}
	pub fn to_string(&self) -> String {
		match &self.state {
			State::Todo => String::new() + "- [ ] " + &self.description,
			State::Done => String::new() + "- [x] " + &self.description,
		}
	}
}

impl From<Line> for String {
	fn from(line: Line) -> Self {
		let mut string = String::new();
		string.push_str(&line.index.to_string());
		string.push_str(" ");
		string.push_str(&line.string);
		string.push_str(&line.suffix.unwrap_or(String::new()));
		string
	}
}

impl From<&str> for Item {
	fn from(string: &str) -> Self {
		Item::new(String::from(string))
	}
}

impl Eq for Item {}

impl PartialEq<Self> for Item {
	fn eq(&self, other: &Self) -> bool {
		String::eq(&self.description, &other.description)
	}
}

impl PartialOrd<Self> for Item {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		String::partial_cmp(&self.description, &other.description)
	}
}

impl Ord for Item {
	fn cmp(&self, other: &Self) -> Ordering {
		String::cmp(&self.description, &other.description)
	}
}
