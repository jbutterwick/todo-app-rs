use chrono::NaiveDate;
use std::cmp::Ordering;

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

	pub fn parse_dates(date_string: &str) -> Option<NaiveDate> {
		// Only full y-m-d dates parse to a NaiveDate; take the leading 10 chars so
		// trailing content on a file line is ignored. `get` is panic-safe on short input.
		let head = date_string.get(..10).unwrap_or(date_string);
		NaiveDate::parse_from_str(head, "%Y-%m-%d")
			.or_else(|_| NaiveDate::parse_from_str(head, "%Y/%m/%d"))
			.ok()
	}

	pub fn parse_line(string: &str) -> Self {
		let (status, rest) = string.split_at(3);
		let state = match Status::from_str(status) {
			Some(state) => state,
			None => panic!("Invalid formatting at line: {status} {rest}\nItems should start with one of the following valid states: [ ], [@], [x], [~], [?]"),
		};

		// A trailing "-> <date>" is the due date — but only when it actually parses,
		// so a description that itself contains "->" is left intact.
		let (body, due_date) = match rest.rsplit_once("->") {
			Some((before, after)) => match Self::parse_dates(after.trim()) {
				Some(date) => (before, Some(date)),
				None => (rest, None),
			},
			None => (rest, None),
		};

		// A leading run of '!' is the priority marker; the remainder is the description.
		let body = body.trim();
		let (priority, description) = match body.split_once(char::is_whitespace) {
			Some((head, tail)) if !head.is_empty() && head.bytes().all(|b| b == b'!') => {
				(head.len() as i8, tail.trim_start().to_string())
			}
			_ => (0, body.to_string()),
		};

		Self::new(description, state, due_date, priority)
	}

	pub(crate) fn suffix(&self) -> (String, String) {
		let priority = if self.priority > 0 {
			format!(" {} ", "!".repeat(self.priority as usize))
		} else {
			String::from(" ")
		};

		let date_string = if let Some(date) = self.due_date {
			format!(" -> {}", date.format("%Y-%m-%d"))
		} else {
			String::new()
		};

		(priority, date_string)
	}

	pub fn get_file_string(&self) -> String {
		let prefix = Status::as_str(&self.state);
		let (priority, date_string) = self.suffix();
		format!("{}{}{}{}", prefix, priority, &self.description, date_string)
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
		fn status_rank(s: &Status) -> u8 {
			match s {
				Status::Ongoing => 0,
				Status::Open => 1,
				Status::InQuestion => 2,
				Status::Checked => 3,
				Status::Obsolete => 4,
			}
		}
		status_rank(&self.state)
			.cmp(&status_rank(&other.state))
			.then(other.priority.cmp(&self.priority))
			.then(self.description.cmp(&other.description))
	}
}
