use crate::item::{Item, State};
use crate::response::{
	ErrorResponse, ExitResponse, HelpResponse, ListResponse, NoResponse, Output, Respond,
	ResponseType, SaveResponse,
};
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::io::{stdin, stdout, Write};

pub struct Todo {
	pub item_vec: Vec<Item>,
}

impl Todo {
	pub(crate) fn new() -> Self {
		Todo { item_vec: vec![] }
	}

	pub(crate) fn from_existing(existing_list: String) -> Self {
		let item_list = existing_list.split("\n").collect::<Vec<&str>>();
		let mut item_vec = vec![];
		for item in item_list {
			if item != "\n" {
				item_vec.push(Item::parse(item))
			}
		}

		Todo { item_vec }
	}

	pub(crate) fn todo_loop(todo: &mut Todo) -> () {
		let mut stdout = stdout();
		stdout
			.execute(terminal::Clear(terminal::ClearType::All))
			.unwrap();
		loop {
			stdout.queue(cursor::SavePosition).unwrap();
			let mut command = String::new();
			stdin()
				.read_line(&mut command)
				.expect("Failed to read command");
			match todo.dispatch(command) {
				Output {
					kind: ResponseType::Continue,
					value,
				} => {
					if value != String::new() {
						stdout.write_all(format!("{}", &value).as_bytes()).unwrap();
					}
				}
				Output {
					kind: ResponseType::Exit,
					value: _,
				} => {
					stdout.write_all("goodbye!".as_bytes()).unwrap();
					break;
				}
				Output {
					kind: ResponseType::Error,
					value,
				} => {
					stdout.write_all(value.as_bytes()).unwrap();
				}
			}
			stdout.write_all("\n>".as_bytes()).unwrap();
			stdout.flush().unwrap();
			stdout.queue(cursor::RestorePosition).unwrap();
			stdout
				.queue(terminal::Clear(terminal::ClearType::FromCursorUp))
				.unwrap();
		}
	}

	fn dispatch(&mut self, input: String) -> Output {
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
					help_msg: "Available commands:
help    | h                                 Displays this help message
list    | l                                 Display the todo list
add     | a  <todo item description>        Adds the item to the todo list
remove  | rm <item index or description>    Removes the item from the todo list
done    | d  <item index or description>    Marks the item as done
flip    | f  <item index or description>    Flips the items done state
save    | s                                 Saves the entire list to `TODO.md`
quit    | q                                 Exit the program",
				}
				.to_output(),

				"list" | "l" => ListResponse {
					list: &self.item_vec,
				}
				.to_output(),

				"save" | "s" => SaveResponse {
					list: &self.item_vec,
				}
				.to_output(),

				"quit" | "exit" | "q" | "e" => ExitResponse {
					exit_msg: "buh-bye!",
				}
				.to_output(),

				"add" | "a" => {
					let string_tail = tail.join(" ");
					let output = match string_tail.is_empty() {
						true => ErrorResponse {
							list: &self.item_vec,
							error_msg: String::from("Please enter description"),
						}
						.to_output(),
						_ => {
							self.item_vec.push(Item::from(&*string_tail));
							ListResponse {
								list: &self.item_vec,
							}
							.to_output()
						}
					};
					output
				}

				"done" | "d" | "flip" | "f" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => {
								if item.state == State::Todo {
									item.state = State::Done
								} else {
									item.state = State::Todo
								}
							}
							_ => {
								return ErrorResponse {
									list: &self.item_vec,
									error_msg: format!("unable to find item {}", num),
								}
								.to_output()
							}
						},
						_ => match self
							.item_vec
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
							None => {
								return ErrorResponse {
									list: &self.item_vec,
									error_msg: format!("unable to find item {}", string_index),
								}
								.to_output()
							}
						},
					};

					ListResponse {
						list: &self.item_vec,
					}
				}
				.to_output(),
				"rem" | "rm" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => {
							self.item_vec.remove(num - 1);
						}
						_ => {
							self.item_vec
								.retain(|item| !(item.description == string_index));
						}
					}

					ListResponse {
						list: &self.item_vec,
					}
				}
				.to_output(),

				arg => ErrorResponse {
					list: &self.item_vec,
					error_msg: format!("unknown argument: {}", arg),
				}
				.to_output(),
			},
			_ => ErrorResponse {
				list: &self.item_vec,
				error_msg: String::from("no argument made"),
			}
			.to_output(),
		}
	}
}
