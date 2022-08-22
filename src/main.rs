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

	app_loop(&mut todo)
}

fn app_loop(todo: &mut Todo) -> () {
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
			app_loop(todo)
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
	let trimmed = input.trim();
	let lowercase = trimmed.to_lowercase();
	let commands = lowercase.split_whitespace().collect::<Vec<&str>>();

	match &commands[..] {
		&[first] => match first {
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
				list: &todo.item_list,
			}
			.to_output(),
			"quit" | "exit" => ExitResponse {
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
		[first, tail @ ..] => match first {
			&"add" => {
				let string_tail = tail.join(" ");
				todo.item_list.items.push(Item::from(&*string_tail));
				todo.item_list.items.sort();
				ListResponse {
					list: &todo.item_list,
				}
			}
			.to_output(),
			&"done" => {
				let string_tail = tail.join(" ");

				let index = todo
					.item_list
					.items
					.binary_search(&Item::from(&*string_tail))
					.unwrap_or({
						let maybe_index = string_tail
							.parse::<usize>()
							.expect("unable to find index by parse or search");
						maybe_index - 1
					});

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
			&"help" | &"list" | &"quit" => {
				let string_tail = tail.join(" ");
				ErrorResponse {
					error_msg: stringify!("unexpected argument : {}", string_tail),
				}
				.to_output()
			}

			_ => ErrorResponse {
				error_msg: stringify!("unknown argument : {}", second),
			}
			.to_output(),
		},
		[] => ErrorResponse {
			error_msg: "no argument made. type `help`",
		}
		.to_output(),
	}
}
