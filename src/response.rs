use crate::item::Item;
use crossterm::style::Stylize;
use std::fs;

use std::fmt::{Display, Formatter};

pub struct Output {
	pub kind: ResponseType,
	pub value: String,
}

impl Display for Output {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.value)
	}
}

pub enum ResponseType {
	Exit,
	Continue,
	Error,
}

pub trait Respond {
	fn to_output(&self) -> Output;
}

pub struct StringResponse<'a> {
	pub str: &'a str,
}

impl Respond for StringResponse<'_> {
	fn to_output(&self) -> Output {
		Output {
			kind: ResponseType::Continue,
			value: String::from(self.str),
		}
	}
}

pub struct ErrorResponse<'a> {
	pub list: &'a Vec<Item>,
	pub error_msg: String,
}

impl Respond for ErrorResponse<'_> {
	fn to_output(&self) -> Output {
		let mut string = String::new();
		for (index, item) in self.list.iter().enumerate() {
			string.push_str(&String::from(item.to_line(index)));
			string.push_str("\n");
		}
		string.push_str(&*String::from(&self.error_msg).red().to_string());
		Output {
			kind: ResponseType::Error,
			value: string,
		}
	}
}

pub struct NoResponse;

impl Respond for NoResponse {
	fn to_output(&self) -> Output {
		Output {
			kind: ResponseType::Continue,
			value: String::new(),
		}
	}
}

pub struct ExitResponse<'a> {
	pub exit_msg: &'a str,
}

impl Respond for ExitResponse<'_> {
	fn to_output(&self) -> Output {
		Output {
			kind: ResponseType::Exit,
			value: String::from(self.exit_msg).blue().to_string(),
		}
	}
}

pub struct HelpResponse<'a> {
	pub help_msg: &'a str,
}

impl Respond for HelpResponse<'_> {
	fn to_output(&self) -> Output {
		Output {
			kind: ResponseType::Continue,
			value: String::from(self.help_msg).yellow().to_string(),
		}
	}
}

pub struct ListResponse<'a> {
	pub list: &'a Vec<Item>,
}

impl Respond for ListResponse<'_> {
	fn to_output(&self) -> Output {
		let mut string = String::new();
		for (index, item) in self.list.iter().enumerate() {
			string.push_str(&String::from(item.to_line(index)));
			string.push_str("\n");
		}
		Output {
			kind: ResponseType::Continue,
			value: string,
		}
	}
}

pub struct SaveResponse<'a> {
	pub list: &'a Vec<Item>,
}

impl Respond for SaveResponse<'_> {
	fn to_output(&self) -> Output {
		let mut string = String::new();
		for item in self.list.iter() {
			string.push_str(&String::from(item.to_string() + "\n"));
		}
		fs::write("TODO.md", string).unwrap();

		let mut string = String::new();
		for (index, item) in self.list.iter().enumerate() {
			string.push_str(&String::from(item.to_line(index)));
			string.push_str("\n");
		}
		Output {
			kind: ResponseType::Continue,
			value: String::from(string + "wrote list to TODO.md"),
		}
	}
}
