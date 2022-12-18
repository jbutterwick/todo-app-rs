use crossterm::style::Stylize;
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

	pub fn parse(string: &str) -> Self {
		match string.find("- [ ]") {
			Some(_) => {
				let stripped = string.strip_prefix("- [ ] ").unwrap_or("");
				Item::new(String::from(stripped))
			}
			_ => {
				let stripped = string.strip_prefix("- [x] ").unwrap_or("");
				let mut item = Item::new(String::from(stripped));
				item.state = State::Done;
				item
			}
		}
	}
	pub fn to_line(&self, index: usize) -> Line {
		let string = String::from(&self.description);
		Line {
			index: index + 1,
			string: if self.state == State::Done {
				string.blue().to_string()
			} else {
				string.yellow().to_string()
			},
			suffix: if self.state == State::Done {
				Some(String::from("(done)"))
			} else {
				None
			},
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

		match line.suffix {
			Some(suffix) => {
				string.push_str(&*suffix);
				string
			}
			None => string,
		}
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
