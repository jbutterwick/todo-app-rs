use crate::item::Item;
use crossterm::style::Stylize;
use std::fs;

use std::fmt::{Display, Formatter};

pub struct Output {
	pub kind: Kind,
	pub value: String,
}

impl Display for Output {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.value)
	}
}

pub enum Kind {
	Exit,
	Continue,
	Error,
}

pub trait Respond {
	fn to_output(&self) -> Output;
}

pub struct Normal<'a> {
	pub str: &'a str,
}

impl Respond for Normal<'_> {
	fn to_output(&self) -> Output {
		Output {
			kind: Kind::Continue,
			value: String::from(self.str),
		}
	}
}

pub struct Error<'a> {
	pub list: &'a Vec<Item>,
	pub error_msg: String,
}

impl Respond for Error<'_> {
	fn to_output(&self) -> Output {
		let mut string = String::new();
		for item in self.list {
			string.push_str(&(item.to_string() + "\n"));
		}
		string.push_str(&String::from(&self.error_msg).red().to_string());
		Output {
			kind: Kind::Error,
			value: string,
		}
	}
}

pub struct Empty;

impl Respond for Empty {
	fn to_output(&self) -> Output {
		Output {
			kind: Kind::Continue,
			value: String::new(),
		}
	}
}

pub struct Exit<'a> {
	pub list: &'a Vec<Item>,
	pub exit_msg: &'a str,
}

impl Respond for Exit<'_> {
	fn to_output(&self) -> Output {
		let mut string = String::new();
		for item in self.list {
			string.push_str(&(item.to_string() + "\n"));
		}
		fs::write("TODO.md", string).unwrap();

		Output {
			kind: Kind::Exit,
			value: String::from(self.exit_msg).blue().to_string(),
		}
	}
}

pub struct Help<'a> {
	pub help_msg: &'a str,
}

impl Respond for Help<'_> {
	fn to_output(&self) -> Output {
		Output {
			kind: Kind::Continue,
			value: String::from(self.help_msg).yellow().to_string(),
		}
	}
}

pub struct List<'a> {
	pub list: &'a Vec<Item>,
}

impl Respond for List<'_> {
	fn to_output(&self) -> Output {
		let mut string = String::new();
		if self.list.is_empty() {
			Output {
				kind: Kind::Continue,
				value: String::from("your todo list has no items, add one with the `add` command"),
			}
		} else {
			for item in self.list {
				string.push_str(&(item.to_string() + "\n"));
			}
			Output {
				kind: Kind::Continue,
				value: string,
			}
		}
	}
}

pub struct Save<'a> {
	pub list: &'a Vec<Item>,
}

impl Respond for Save<'_> {
	fn to_output(&self) -> Output {
		let mut file_string = String::new();
		let mut string = String::new();
		for item in self.list {
			file_string.push_str(&(item.get_file_string() + "\n"));
			string.push_str(&(item.to_string() + "\n"));
		}

		fs::write("todo.xit", file_string).expect("Couldn't write to file for some reason");

		Output {
			kind: Kind::Continue,
			value: (string + "wrote list to todo.xit"),
		}
	}
}
