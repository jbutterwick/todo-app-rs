use crate::item::{Item, Status};
use crate::response::{Kind, Output, Response};
use std::fs;
use std::io::{stdin, stdout, Write};

pub struct Todo {
	pub item_vec: Vec<Item>,
	pub file_path: String,
}

impl Todo {
	pub(crate) fn new(file_path: String) -> Self {
		Self {
			item_vec: vec![],
			file_path,
		}
	}

	pub(crate) fn from_existing(existing_list: &str, file_path: String) -> Self {
		let item_list = existing_list.lines().collect::<Vec<&str>>();
		let mut item_vec = vec![];
		for item in item_list {
			if !item.is_empty() || item.ne("\n") {
				println!("{}", item);
				item_vec.push(Item::parse_line(item));
			}
		}
		Self {
			item_vec,
			file_path,
		}
	}

	fn save_to_file(&self) -> Result<(), String> {
		let mut file_string = String::new();
		for item in &self.item_vec {
			file_string.push_str(&(item.get_file_string() + "\n"));
		}
		fs::write(&self.file_path, file_string)
			.map_err(|e| format!("failed to write {}: {e}", self.file_path))
	}

	fn export_to_md(&self) -> Result<(), String> {
		let mut string = String::new();
		for item in &self.item_vec {
			string.push_str(&(item.to_string() + "\n"));
		}
		fs::write("TODO.md", string)
			.map_err(|e| format!("failed to write TODO.md: {e}"))
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
			let response = todo.dispatch(&command);

			// Handle file I/O for save/exit before rendering
			match &response {
				Response::Save => {
					match todo.save_to_file() {
						Ok(_) => {
							let output = response.render(&todo.item_vec);
							stdout.write_all(output.value.as_bytes()).unwrap();
							stdout
								.write_all(
									format!("wrote list to {}\n", todo.file_path)
										.as_bytes(),
								)
								.unwrap();
							stdout.flush().unwrap();
							continue;
						}
						Err(e) => {
							let output = Response::Error(e).render(&todo.item_vec);
							stdout.write_all(output.value.as_bytes()).unwrap();
							stdout.write_all(b"\n").unwrap();
							stdout.flush().unwrap();
							continue;
						}
					}
				}
				Response::Exit => {
					let _ = todo.save_to_file();
					let _ = todo.export_to_md();
				}
				_ => {}
			}

			let output = response.render(&todo.item_vec);
			match output {
				Output {
					kind: Kind::Continue,
					value,
				} => {
					if !value.is_empty() {
						stdout.write_all(value.as_bytes()).unwrap();
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

	fn dispatch(&mut self, input: &str) -> Response {
		if input == String::new() {
			return Response::Empty;
		}

		match input
			.trim()
			.to_lowercase()
			.split_whitespace()
			.collect::<Vec<&str>>()
			.split_first()
		{
			Some((first, tail)) => match *first {
				"help" | "h" => Response::Help(
					"Available commands:
help     | h                         Displays this help message
list     | l                         Display the todo list
add      | a | + <item description>  Adds the item to the todo list
edit     | e <index> <new desc>       Edits an item's description
remove   | r | - <item>              Removes the item from the todo list
done     | x <item>                  Marks the item as done
undo     | o <item>                  Marks the item as not done
obsolete | ~ <item>                  Marks the item as obsolete
ongoing  | @ <item>                  Marks the item as ongoing
question | ? <item>                  Marks the item as question
duedate  | d <date> <item>           Gives the item a due date
priority | p <-|+> <priority> <item> Adds or subtracts priority from the item
sort                                  Sorts the list by status, priority, name
filter   | f <status>                Filters list by status (open|done|ongoing|obsolete|question)
save     | s                         Saves the list to file
quit     | q                         Exit the program",
				),

				"list" | "l" => Response::List,

				"sort" => {
					self.item_vec.sort();
					Response::List
				}

				"filter" | "f" => {
					if tail.is_empty() {
						return Response::Error(String::from(
							"usage: filter <open|done|ongoing|obsolete|question>",
						));
					}
					let status_filter = match tail[0] {
						"open" => Status::Open,
						"done" | "checked" => Status::Checked,
						"ongoing" => Status::Ongoing,
						"obsolete" => Status::Obsolete,
						"question" => Status::InQuestion,
						other => {
							return Response::Error(format!("unknown status: {other}"))
						}
					};
					let mut filtered = String::new();
					for item in &self.item_vec {
						if item.state == status_filter {
							filtered.push_str(&(item.to_string() + "\n"));
						}
					}
					if filtered.is_empty() {
						Response::Continue(String::from("no items match that filter"))
					} else {
						Response::Continue(filtered)
					}
				}

				"save" | "s" => Response::Save,

				"quit" | "q" => Response::Exit,

				"add" | "a" | "+" => {
					let string_tail = tail.join(" ");
					if string_tail.is_empty() {
						Response::Error(String::from("Please enter description"))
					} else {
						self.item_vec.push(Item::from(&*string_tail));
						Response::List
					}
				}

				"edit" | "e" => {
					if tail.is_empty() {
						return Response::Error(String::from(
							"usage: edit <index> <new description>",
						));
					}
					let index_str = tail[0];
					let new_desc = tail[1..].join(" ");
					if new_desc.is_empty() {
						return Response::Error(String::from(
							"please provide a new description",
						));
					}
					match index_str.parse::<usize>() {
						Ok(num) => match self.item_vec.get_mut(num - 1) {
							Some(item) => item.description = new_desc,
							_ => {
								return Response::Error(format!(
									"unable to find item {num}"
								))
							}
						},
						_ => {
							return Response::Error(String::from(
								"edit requires a numeric index",
							))
						}
					};
					Response::List
				}

				"done" | "x" => {
					let string_index = tail.join(" ");
					self.find_and_update(&string_index, |item| {
						item.state = if item.state == Status::Checked {
							Status::Open
						} else {
							Status::Checked
						};
					})
				}

				"undo" | "o" => {
					let string_index = tail.join(" ");
					self.find_and_update(&string_index, |item| {
						item.state = Status::Open;
					})
				}

				"ongoing" | "@" => {
					let string_index = tail.join(" ");
					self.find_and_update(&string_index, |item| {
						item.state = Status::Ongoing;
					})
				}

				"obsolete" | "~" => {
					let string_index = tail.join(" ");
					self.find_and_update(&string_index, |item| {
						item.state = Status::Obsolete;
					})
				}

				"question" | "?" => {
					let string_index = tail.join(" ");
					self.find_and_update(&string_index, |item| {
						item.state = Status::InQuestion;
					})
				}

				"duedate" | "d" => {
					if tail.is_empty() {
						return Response::Error(String::from(
							"usage: duedate <date> <item>",
						));
					}
					let date_str = tail[0];
					let date = Item::parse_dates(date_str);
					if date.is_none() {
						return Response::Error(format!(
							"unable to parse date: {date_str}"
						));
					}
					let item_index = tail[1..].join(" ");
					if item_index.is_empty() {
						return Response::Error(String::from("please specify an item"));
					}
					self.find_and_update(&item_index, |item| {
						item.due_date = date;
					})
				}

				"priority" | "p" => {
					if tail.is_empty() {
						return Response::Error(String::from(
							"usage: priority <!!...> <item>",
						));
					}
					let mut priority = 0;
					for char in tail[0].chars() {
						match char {
							'.' => continue,
							'!' => priority += 1,
							_ => break,
						}
					}
					let string_index = tail[1..].join(" ");
					if string_index.is_empty() {
						return Response::Error(String::from("please specify an item"));
					}
					self.find_and_update(&string_index, |item| {
						item.priority = priority;
					})
				}

				"remove" | "r" | "-" => {
					let string_index = tail.join(" ");
					match string_index.parse::<usize>() {
						Ok(num) => {
							if num == 0 || num > self.item_vec.len() {
								return Response::Error(format!(
									"unable to find item {num}"
								));
							}
							self.item_vec.remove(num - 1);
						}
						_ => {
							self.item_vec
								.retain(|item| item.description != string_index);
						}
					}
					Response::List
				}

				arg => Response::Error(format!("unknown argument: {arg}")),
			},
			_ => Response::Error(String::from("no argument made")),
		}
	}

	fn find_and_update(
		&mut self,
		index_or_name: &str,
		update: impl FnOnce(&mut Item),
	) -> Response {
		match index_or_name.parse::<usize>() {
			Ok(num) => match self.item_vec.get_mut(num - 1) {
				Some(item) => {
					update(item);
					Response::List
				}
				_ => Response::Error(format!("unable to find item {num}")),
			},
			_ => match self
				.item_vec
				.iter_mut()
				.find(|item| item.description == index_or_name)
			{
				Some(item) => {
					update(item);
					Response::List
				}
				None => {
					Response::Error(format!("unable to find item {index_or_name}"))
				}
			},
		}
	}
}
