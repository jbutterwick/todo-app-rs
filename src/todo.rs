use crate::item::{Item, Status};
use crate::response::{Empty, Error, Exit, Help, Kind, List, Output, Respond, Save};
use std::io::{stdin, stdout, Write};

pub struct Todo {
	pub item_vec: Vec<Item>,
}

impl Todo {
	pub(crate) const fn new() -> Self {
		Self { item_vec: vec![] }
	}

	pub(crate) fn from_existing(existing_list: &str) -> Self {
		let item_list = existing_list.lines().collect::<Vec<&str>>();
		let mut item_vec = vec![];
		for item in item_list {
			if !item.is_empty() || item.ne("\n") {
				println!("{}", item);
				item_vec.push(Item::parse_line(item));
			}
		}
		Self { item_vec }
	}

	pub(crate) fn todo_loop(todo: &mut Self) {
		let mut stdout = stdout();
		stdout
			.write_all("welcome to todo_rs! type `help` to see the list of commands\n".as_bytes())
			.unwrap();
		stdout.flush().unwrap();
		loop {
			let mut command = String::new();
			stdin()
				.read_line(&mut command)
				.expect("Failed to read command");
			match todo.dispatch(&command) {
				Output {
					kind: Kind::Continue,
					value,
				} => {
					if !value.is_empty() {
						stdout.write_all(value.to_string().as_bytes()).unwrap();
					}
				}
				Output {
					kind: Kind::Exit,
					value: _,
				} => {
					stdout.write_all(b"goodbye!").unwrap();
					break;
				}
				Output {
					kind: Kind::Error,
					value,
				} => {
					stdout.write_all(value.as_bytes()).unwrap();
				}
			}
			stdout.write_all(b"\n").unwrap();
			stdout.flush().unwrap();
		}
	}

	fn dispatch(&mut self, input: &str) -> Output {
		if input == String::new() {
			return Empty {}.to_output();
		}

		match input
			.trim()
			.to_lowercase()
			.split_whitespace()
			.collect::<Vec<&str>>()
			.split_first()
		{
			Some((first, tail)) => match *first {
				"help" | "h" => Help {
					help_msg: "Available commands:
help     | h                         Displays this help message
list     | l                         Display the todo list
add      | a | + <item description>  Adds the item to the todo list
remove   | r | - <item>              Removes the item from the todo list
done     | x <item>                  Marks the item as done
undo     | o <item>                  Marks the item as not done
obsolete | ~ <item>                  Marks the item as obsolete
ongoing  | @ <item>                  Marks the item as ongoing
question | ? <item>                  Marks the item as question
duedate  | d <date> <item>           Gives the item a due date
priority | p <-|+> <priority> <item> Adds or subtracts priority from the item
save     | s <name=todo.xit>         Saves the entire list to specified filename
quit     | q                         Exit the program",
				}
				.to_output(),

				"list" | "l" => List {
					list: &self.item_vec,
				}
				.to_output(),

				"save" | "s" => Save {
					list: &self.item_vec,
				}
				.to_output(),

				"quit" | "q" => Exit {
					list: &self.item_vec,
					exit_msg: "buh-bye!",
				}
				.to_output(),

				"add" | "a" | "+" => {
					let string_tail = tail.join(" ");
					if string_tail.is_empty() {
						Error {
							list: &self.item_vec,
							error_msg: String::from("Please enter description"),
						}
						.to_output()
					} else {
						self.item_vec.push(Item::from(&*string_tail));
						List {
							list: &self.item_vec,
						}
						.to_output()
					}
				}

				"done" | "x" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => {
								if item.state == Status::Open {
									item.state = Status::Checked;
								} else {
									item.state = Status::Open;
								}
							}
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
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
								if item.state == Status::Open {
									item.state = Status::Checked;
								} else {
									item.state = Status::Open;
								}
							}
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"undo" | "o" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => item.state = Status::Open,
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
								}
								.to_output()
							}
						},
						_ => match self
							.item_vec
							.iter_mut()
							.find(|item| item.description == string_index)
						{
							Some(item) => item.state = Status::Open,
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"ongoing" | "@" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => item.state = Status::Ongoing,
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
								}
								.to_output()
							}
						},
						_ => match self
							.item_vec
							.iter_mut()
							.find(|item| item.description == string_index)
						{
							Some(item) => item.state = Status::Ongoing,
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"obsolete" | "~" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => item.state = Status::Obsolete,
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
								}
								.to_output()
							}
						},
						_ => match self
							.item_vec
							.iter_mut()
							.find(|item| item.description == string_index)
						{
							Some(item) => item.state = Status::Obsolete,
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"question" | "?" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => item.state = Status::InQuestion,
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
								}
								.to_output()
							}
						},
						_ => match self
							.item_vec
							.iter_mut()
							.find(|item| item.description == string_index)
						{
							Some(item) => item.state = Status::InQuestion,
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"duedate" | "d" => {
					// TODO THIS DOES NOTHING USEFUL
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => {}
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
								}
								.to_output()
							}
						},
						_ => match self
							.item_vec
							.iter_mut()
							.find(|item| item.description == string_index)
						{
							Some(item) => {}
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"priority" | "p" => {
					let mut priority = 0;

					for char in tail.iter().next().unwrap().chars() {
						match char {
							'.' => continue,
							'!' => priority += 1,
							_ => break,
						}
					}

					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => {
								item.priority = priority;
							}
							_ => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {num}"),
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
								item.priority = priority;
							}
							None => {
								return Error {
									list: &self.item_vec,
									error_msg: format!("unable to find item {string_index}"),
								}
								.to_output()
							}
						},
					};

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				"remove" | "r" | "-" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => {
							self.item_vec.remove(num - 1);
						}
						_ => {
							self.item_vec
								.retain(|item| item.description != string_index);
						}
					}

					List {
						list: &self.item_vec,
					}
				}
				.to_output(),

				arg => Error {
					list: &self.item_vec,
					error_msg: format!("unknown argument: {arg}"),
				}
				.to_output(),
			},
			_ => Error {
				list: &self.item_vec,
				error_msg: String::from("no argument made"),
			}
			.to_output(),
		}
	}
}
