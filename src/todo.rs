use crate::item::*;
use crate::output::*;
use crate::response::*;

pub struct Todo {
	pub(crate) item_list: ItemList,
}

impl Todo {
	pub fn dispatch(self, input: &str) -> Output<String> {
		let trimmed_input = input.trim();
		let commands = trimmed_input.split_whitespace().collect::<Vec<&str>>();

		match commands[..] {
			[first] => match first {
				"help" => HelpResponse {
					help_msg: &String::from(
						"
      Available commands:
      help                              Displays this help
      list                              Display the todo list
      add <todo item description>       Adds the item to the todo list
      done <todo item number>           Marks the item as done
      quit                              Exit the program
					",
					),
				}
				.to_output(),
				"list" => ListResponse {
					list: &self.item_list,
				}
				.to_output(),
				"quit" => ExitResponse {
					exit_msg: &String::from("buh-bye!"),
				}
				.to_output(),
				"add" | "done" => ErrorResponse {
					error_msg: "not enough arguments",
				}
				.to_output(),
				_ => ErrorResponse {
					error_msg: stringify!("unknown argument : {}", first),
				}
				.to_output(),
			},
			[first, second] => match first {
				"add" => StringResponse { str: "buh-bye!" }.to_output(),
				"done" => StringResponse { str: "buh-bye!" }.to_output(),
				"help" | "list" | "quit" => ErrorResponse {
					error_msg: stringify!("unexpected argument : {}", second),
				}
				.to_output(),
				_ => ErrorResponse {
					error_msg: stringify!("unknown argument : {}", second),
				}
				.to_output(),
			},
			[_, _, ..] => ErrorResponse {
				error_msg: "too many arguments. type `help`",
			}
			.to_output(),
			[] => ErrorResponse {
				error_msg: "no argument made. type `help`",
			}
			.to_output(),
		}
	}
}
