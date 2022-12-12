extern crate core;

mod item;
mod output;
mod response;

use response::{NoResponse, SaveResponse};

use crate::item::{Item, State};
use crate::output::Output;
use crate::response::{
	ErrorResponse, ExitResponse, HelpResponse, ListResponse, Respond, ResponseType,
};
use std::io;

pub struct Todo {
	pub item_list: Vec<Item>,
}

impl Todo {
	fn new() -> Self {
		Todo { item_list: vec![] }
	}

	fn todo_loop(todo: &mut Todo) -> () {
		loop {
			println!(">");
			let mut command = String::new();

			io::stdin()
				.read_line(&mut command)
				.expect("Failed to read command");

			println!("You entered: {command}");

			match todo.dispatch(command) {
				Output {
					kind: ResponseType::Continue,
					value,
				} => {
					if value != String::new() {
						println!("{}", &value);
					}
				}
				Output {
					kind: ResponseType::Exit,
					value: _,
				} => {
					println!("goodbye!");
					break;
				}
				Output {
					kind: ResponseType::Error,
					value,
				} => {
					println!("error with message: {}", &value)
				}
			}
		}
	}

	fn dispatch(&mut self, input: String) -> Output<String> {
		if input == String::new() {
			return NoResponse {}.to_output();
		}

		match input
			.trim()
			.to_lowercase()
			.split_whitespace()
			.collect::<Vec<&str>>()
			.split_first()
		{
			Some((first, tail)) => match *first {
				"help" => HelpResponse {
					help_msg: "
							Available commands:
							help    | h                                 Displays this help message
							list    | l                                 Display the todo list
							add     | a  <todo item description>        Adds the item to the todo list
							remove  | rm <item index or description>    Removes the item from the todo list
							done    | d  <item index or description>    Marks the item as done
							flip    | f  <item index or description>    Flips the items done state
							save    | s                                 Saves the entire list to `TODO.md`
							quit    | q                                 Exit the program
							",
				}
				.to_output(),

				"list" | "l" => ListResponse {
					list: &self.item_list,
				}
				.to_output(),

				"save" | "s" => SaveResponse {
					list: &self.item_list,
				}
				.to_output(),

				"quit" | "exit" | "q" | "e" => ExitResponse {
					exit_msg: "buh-bye!",
				}
				.to_output(),

				"add" | "a" => {
					let string_tail = tail.join(" ");
					self.item_list.push(Item::from(&*string_tail));
					ListResponse {
						list: &self.item_list,
					}
				}
				.to_output(),

				"done" | "d" | "flip" | "f" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_list.get_mut(num - 1) {
							Some(item) => {
								if item.state == State::Todo {
									item.state = State::Done
								} else {
									item.state = State::Todo
								}
							}
							_ => {
								return ErrorResponse {
									error_msg: format!("unable to find item at index {}", num),
								}
								.to_output()
							}
						},
						_ => match self
							.item_list
							.iter_mut()
							.find(|item| item.description == string_index)
						{
							Some(item) => {
								if item.state == State::Todo {
									item.state = State::Done
								} else {
									item.state = State::Todo
								}
							}
							None => println!("unable to find item by {}", string_index),
						},
					};

					ListResponse {
						list: &self.item_list,
					}
				}
				.to_output(),
				"rem" | "rm" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => {
							self.item_list.remove(num - 1);
						}
						_ => println!("unable to find item {}", string_index),
					};

					ListResponse {
						list: &self.item_list,
					}
				}
				.to_output(),

				arg => ErrorResponse {
					error_msg: format!("unknown argument: {}", arg),
				}
				.to_output(),
			},
			_ => ErrorResponse {
				error_msg: String::from("no argument made"),
			}
			.to_output(),
		}
	}
}

fn main() {
	println!("Todo List");
	println!("Enter a command. Enter `help` to list available commands: ");

	let mut todo = Todo::new();

	Todo::todo_loop(&mut todo)
}
