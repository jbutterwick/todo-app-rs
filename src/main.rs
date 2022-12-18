extern crate core;

mod item;
mod response;
mod todo;

use crate::todo::Todo;
use crossterm::{ExecutableCommand, terminal};

fn main() {
	let mut stdout = std::io::stdout();
	stdout
		.execute(terminal::Clear(terminal::ClearType::All))
		.unwrap();
	match std::fs::read_to_string("TODO.md") {
		Ok(existing_list) => {
			let mut todo = Todo::from_existing(existing_list);
			let mut string = String::new();
			for (index, item) in todo.item_vec.iter().enumerate() {
				string.push_str(&String::from(item.to_line(index)));
				string.push_str("\n");
			}
			println!("found existing list!");
			println!("{}", string);
			Todo::todo_loop(&mut todo)
		}
		Err(_) => {
			let mut todo = Todo::new();
			Todo::todo_loop(&mut todo)
		}
	};
}
