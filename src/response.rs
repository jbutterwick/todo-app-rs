use crate::output::{Color, ColoredString, Output, Outputtable};
use crate::ItemList;

pub enum ResponseType {
	Exit,
	Continue,
	Error,
}

pub trait Respond<T>
where
	T: Outputtable,
{
	fn to_output(&self) -> Output<T>;
}

pub struct StringResponse<'a> {
	pub str: &'a str,
}

impl Respond<String> for StringResponse<'_> {
	fn to_output(&self) -> Output<String> {
		Output {
			kind: ResponseType::Continue,
			value: String::from(self.str),
		}
	}
}

pub struct ErrorResponse<'a> {
	pub error_msg: &'a str,
}

impl Respond<String> for ErrorResponse<'_> {
	fn to_output(&self) -> Output<String> {
		Output {
			kind: ResponseType::Error,
			value: ColoredString {
				color: Color::Red,
				string: String::from(self.error_msg),
			}
			.show(),
		}
	}
}

pub struct ExitResponse<'a> {
	pub exit_msg: &'a str,
}

impl Respond<String> for ExitResponse<'_> {
	fn to_output(&self) -> Output<String> {
		Output {
			kind: ResponseType::Exit,
			value: ColoredString {
				color: Color::Blue,
				string: String::from(self.exit_msg),
			}
			.show(),
		}
	}
}

pub struct HelpResponse<'a> {
	pub help_msg: &'a str,
}

impl Respond<String> for HelpResponse<'_> {
	fn to_output(&self) -> Output<String> {
		Output {
			kind: ResponseType::Continue,
			value: ColoredString {
				color: Color::Yellow,
				string: String::from(self.help_msg),
			}
			.show(),
		}
	}
}

pub struct ListResponse<'a> {
	pub list: &'a ItemList,
}

impl Respond<String> for ListResponse<'_> {
	fn to_output(&self) -> Output<String> {
		let mut string = String::new();
		for (index, item) in self.list.items.iter().enumerate() {
			string.push_str(&String::from(item.to_line(index)));
			string.push_str("\n");
		}
		Output {
			kind: ResponseType::Continue,
			value: string,
		}
	}
}
