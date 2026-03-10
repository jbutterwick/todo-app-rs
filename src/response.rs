use crate::item::Item;
use crossterm::style::Stylize;

pub enum Response {
	Empty,
	Continue(String),
	Error(String),
	List,
	Help(&'static str),
	Save,
	Exit,
}

impl Response {
	pub fn render(&self, items: &[Item]) -> Output {
		match self {
			Self::Empty => Output {
				kind: Kind::Continue,
				value: String::new(),
			},
			Self::Continue(msg) => Output {
				kind: Kind::Continue,
				value: msg.clone(),
			},
			Self::Error(msg) => {
				let mut string = Self::format_list(items);
				string.push_str(&msg.clone().red().to_string());
				Output {
					kind: Kind::Error,
					value: string,
				}
			}
			Self::List => {
				if items.is_empty() {
					Output {
						kind: Kind::Continue,
						value: String::from("your todo list has no items, add one with the `add` command"),
					}
				} else {
					Output {
						kind: Kind::Continue,
						value: Self::format_list(items),
					}
				}
			}
			Self::Help(msg) => Output {
				kind: Kind::Continue,
				value: String::from(*msg).yellow().to_string(),
			},
			Self::Save => Output {
				kind: Kind::Continue,
				value: Self::format_list(items),
			},
			Self::Exit => Output {
				kind: Kind::Exit,
				value: String::from("buh-bye!").blue().to_string(),
			},
		}
	}

	fn format_list(items: &[Item]) -> String {
		let mut string = String::new();
		for item in items {
			string.push_str(&(item.to_string() + "\n"));
		}
		string
	}
}

pub struct Output {
	pub kind: Kind,
	pub value: String,
}

pub enum Kind {
	Exit,
	Continue,
	Error,
}
