use crate::item::*;
use crate::output::*;
use crate::response::*;

pub struct Todo {
	pub(crate) item_list: ItemList,
}

impl Todo {
	pub fn dispatch(mut self, input: &str) -> Output<String> {
		let trimmed_input = input.trim();
		let commands = trimmed_input.split_whitespace().collect::<Vec<&str>>();

		match commands[..] {
			[first] => match first {
				"help" | "Help" => HelpResponse {
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
				"list" | "List" => ListResponse {
					list: &self.item_list,
				}
				.to_output(),
				"quit" | "Quit" => ExitResponse {
					exit_msg: &String::from("buh-bye!"),
				}
				.to_output(),
				"add" | "Add" | "done" | "Done" => ErrorResponse {
					error_msg: "not enough arguments",
				}
				.to_output(),
				_ => ErrorResponse {
					error_msg: stringify!("unknown argument : {}", first),
				}
				.to_output(),
			},
			[first, second] => match first {
				"add" | "Add" => {
					self.item_list.items.push(Item::from(second));
					self.item_list.items.sort();
					ListResponse {
						list: &self.item_list,
					}
				}
				.to_output(),
				"done" | "Done" => {
					let index = self
						.item_list
						.items
						.binary_search(&Item::from(second))
						.expect("done command must have a valid item index");
					let mut item = self
						.item_list
						.items
						.get_mut(index)
						.expect("index out of bounds");
					item.state = State::Done;
					ListResponse {
						list: &self.item_list,
					}
				}
				.to_output(),
				"help" | "Help" | "list" | "List" | "quit" | "Quit" => ErrorResponse {
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
