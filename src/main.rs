mod item;
mod response;
mod todo;

use crate::todo::Todo;
use std::env;

fn main() {
	let file_path = env::args().nth(1).unwrap_or_else(|| String::from("todo.xit"));

	if let Ok(existing_list) = std::fs::read_to_string(&file_path) {
		let mut todo = Todo::from_existing(&existing_list, file_path);
		let mut string = String::new();
		for item in &todo.item_vec {
			string.push_str(item.to_string().as_str());
			string.push('\n');
		}
		println!("{string}");
		Todo::todo_loop(&mut todo);
	} else {
		let mut todo = Todo::new(file_path);
		Todo::todo_loop(&mut todo);
	};
}
