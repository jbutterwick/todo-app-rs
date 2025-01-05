use crate::response::Error;
use chrono::NaiveDate;
use crossterm::style::Stylize;
use std::cmp::Ordering;
use std::fmt::Display;
use std::fs;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Status {
	Open,
	Ongoing,
	Checked,
	Obsolete,
	InQuestion,
}

impl Status {
	pub const fn as_str(&self) -> &'static str {
		match self {
			Self::Open => "[ ] ",
			Self::Ongoing => "[@] ",
			Self::Checked => "[x] ",
			Self::Obsolete => "[~] ",
			Self::InQuestion => "[?] ",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"[ ]" => Some(Self::Open),
			"[@]" => Some(Self::Ongoing),
			"[x]" => Some(Self::Checked),
			"[~]" => Some(Self::Obsolete),
			"[?]" => Some(Self::InQuestion),
			_ => None,
		}
	}
}

pub struct Item {
	pub description: String,
	pub state: Status,
	pub due_date: Option<NaiveDate>,
	pub priority: i8,
}

pub struct Line {
	string: String,
}

impl Item {
	pub const fn new(
		description: String,
		state: Status,
		due_date: Option<NaiveDate>,
		priority: i8,
	) -> Self {
		Self {
			description,
			state,
			due_date,
			priority,
		}
	}

	fn parse_dates(date_string: &str) -> Option<NaiveDate> {
		for format in [
			"%Y-%m-%d", "%Y/%m/%d", "%Y-%m", "%Y/%m", "%Y", "%Y-W%U", "%Y/W%U", "%Y-Q%q", "%Y/Q%q",
		] {
			let string_to_parse = match format {
				"%Y-%m-%d" | "%Y/%m/%d" => date_string.split_at(9).0,
				"%Y-W%U" | "%Y/W%U" => date_string.split_at(7).0,
				"%Y-%m" | "%Y/%m" | "%Y-Q%q" | "%Y/Q%q" => date_string.split_at(6).0,
				"%Y" => date_string.split_at(3).0,
				_ => unreachable!(),
			};

			if let Ok(date) = NaiveDate::parse_from_str(string_to_parse, format) {
				return Some(date);
			}
		}
		None
	}

	pub fn parse_line(string: &str) -> Self {
		let (status, description) = string.split_at(3);

		let due_date = description.find("->").and_then(|index| {
			let (_, tail) = description.split_at(index + 2);
			let (maybe_date, _) = tail.split_at(12);
			Self::parse_dates(maybe_date)
		});

		let maybe_priority = description.trim().split_once(' ');

		let priority: i8 = if maybe_priority.is_some() {
			maybe_priority.unwrap().0.chars().fold(0, |mut acc: i8, c| {
				if c == '!' {
					acc += 1;
				}
				acc
			})
		} else {
			0
		};

		match Status::from_str(status) {
			Some(Status::Open) => Self::new(description.parse().unwrap(), Status::Open, due_date, priority),
			Some(Status::Ongoing) => Self::new(description.parse().unwrap(), Status::Ongoing, due_date, priority),
			Some(Status::Checked) => Self::new(description.parse().unwrap(), Status::Checked, due_date, priority),
			Some(Status::Obsolete) => Self::new(description.parse().unwrap(), Status::Obsolete, due_date, priority),
			Some(Status::InQuestion) => Self::new(description.parse().unwrap(), Status::InQuestion, due_date, priority),
			None => panic!("Invalid formatting at line: {status} {description}\nItems should start with one of the following valid states: [ ], [@], [x], [~], [?]")
		}
	}

	pub fn get_file_string(&self) -> String {
		let prefix = Status::as_str(&self.state);

		let priority = if self.priority > 0 {
			stringify!(" {} ", &*"!".repeat(self.priority as usize))
		} else {
			" "
		};

		let date_string = if self.due_date.is_some() {
			stringify!(" -> {}", self.due_date.unwrap().format("%Y-%m-%d"))
		} else {
			""
		};

		format!("{}{}{}{}", prefix, priority, &self.description, date_string)
	}
}

impl Display for Item {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let prefix = Status::as_str(&self.state);

		let priority = if self.priority > 0 {
			stringify!(" {} ", &*"!".repeat(self.priority as usize))
		} else {
			" "
		};

		let date_string = if self.due_date.is_some() {
			stringify!(" -> {}", self.due_date.unwrap().format("%Y-%m-%d"))
		} else {
			""
		};

		match self.state {
			Status::Open => write!(
				f,
				"{}{}{}{}",
				prefix.blue(),
				priority,
				&self.description,
				date_string
			),
			Status::InQuestion => {
				write!(
					f,
					"{}{}{}{}",
					prefix.yellow(),
					priority,
					&self.description,
					date_string
				)
			}
			Status::Checked => {
				write!(
					f,
					"{}{}{}{}",
					prefix.green(),
					priority,
					&self.description.clone().grey(),
					date_string
				)
			}
			Status::Ongoing => {
				write!(
					f,
					"{}{}{}{}",
					prefix.magenta(),
					priority,
					&self.description,
					date_string
				)
			}
			Status::Obsolete => {
				write!(
					f,
					"{}{}{}{}",
					prefix.grey(),
					priority,
					&self.description.clone().grey(),
					date_string
				)
			}
		}
	}
}

impl From<Line> for String {
	fn from(line: Line) -> Self {
		line.string
	}
}

impl From<&str> for Item {
	fn from(string: &str) -> Self {
		Self::new(String::from(string), Status::Open, None, 0)
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
		Some(self.cmp(other))
	}
}

impl Ord for Item {
	fn cmp(&self, other: &Self) -> Ordering {
		String::cmp(&self.description, &other.description)
	}
}
