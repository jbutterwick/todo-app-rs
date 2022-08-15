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
			kind: ResponseType::Exit,
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
		for item in &self.list.items {
			string.push_str(&ColoredString::from(item).show())
		}
		Output {
			kind: ResponseType::Continue,
			value: string,
		}
	}
}

// export const emptyListHint = new StringResult(
// new ColoredString('yellow', 'List is empty.  Try adding some items'),
// )
//
// export const doneIndexError: Result = new ErrorResult(
// 'Done command must have a valid item index',
// )
//
// export const unexpectedArg: (cn: string) => Result = (commandName: string) =>
// new ErrorResult(`${commandName} command does not take any arguments`)
//
// export const missingArg: (cn: string) => Result = (commandName: string) =>
// new ErrorResult(`${commandName} command requires an argument`)
//
// export const unknown: Result = new ErrorResult(
// 'I do not understand your command.  ' + 'Enter help to display available commands.',
// )
//
// export const exit = new Exit('exit')
//
// export const next = async (
//     todo: Todo,
//     result: Result,
//     loop: (todo: Todo) => Promise<'done'>,
// stop: () => 'done',
// ): Promise<'done'> => {
// switch (result.kind) {
// case 'continue':
// return await loop(todo)
// case 'exit':
// return stop()
// }
// }
