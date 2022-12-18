extern crate core;

mod item;
mod response;
mod todo;

use crate::todo::Todo;

fn main() {
	match std::fs::read_to_string("TODO.md") {
		Ok(existing_list) => {
			let mut todo = Todo::from_existing(existing_list);
			Todo::todo_loop(&mut todo)
		}
		Err(_) => {
			let mut todo = Todo::new();
			Todo::todo_loop(&mut todo)
		}
	};
}
