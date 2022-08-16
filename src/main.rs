extern crate core;

mod item;
mod output;
mod response;

use crate::item::{Item, ItemList, State};
use crate::output::Output;
use crate::response::{
	ErrorResponse, ExitResponse, HelpResponse, ListResponse, Respond, ResponseType,
};
use std::io;

pub struct Todo {
	pub item_list: ItemList,
}

fn main() {
	println!("Todo List");
	println!("Enter a command. Enter `help` to list available commands: ");

	let mut todo = Todo {
		item_list: ItemList { items: vec![] },
	};

	rep_loop(&mut todo)
}

fn rep_loop(todo: &mut Todo) -> () {
	let mut command = String::new();

	io::stdin()
		.read_line(&mut command)
		.expect("Failed to read command");

	println!("You entered: {command}");

	match dispatch(todo, command) {
		Output {
			kind: ResponseType::Continue,
			value,
		} => {
			println!("{}", &value);
			rep_loop(todo)
		}
		Output {
			kind: ResponseType::Exit,
			value: _,
		} => {
			println!("goodbye!")
		}
		Output {
			kind: ResponseType::Error,
			value,
		} => {
			panic!("error with message: {:?}", &value)
		}
	}
}

fn dispatch(todo: &mut Todo, input: String) -> Output<String> {
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
				list: &todo.item_list,
			}
			.to_output(),
			"quit" | "Quit" | "exit" | "Exit" => ExitResponse {
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
				todo.item_list.items.push(Item::from(second));
				todo.item_list.items.sort();
				ListResponse {
					list: &todo.item_list,
				}
			}
			.to_output(),
			"done" | "Done" => {
				let index = todo
					.item_list
					.items
					.binary_search(&Item::from(second))
					.expect("done command must have a valid item index");
				let mut item = todo
					.item_list
					.items
					.get_mut(index)
					.expect("index out of bounds");
				item.state = State::Done;
				ListResponse {
					list: &todo.item_list,
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
