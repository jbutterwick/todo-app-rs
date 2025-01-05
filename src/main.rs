mod item;
mod response;
mod todo;

use crate::todo::Todo;

fn main() {
	if let Ok(existing_list) = std::fs::read_to_string("todo.xit") {
		let mut todo = Todo::from_existing(&existing_list);
		let mut string = String::new();
		for item in &todo.item_vec {
			string.push_str(item.to_string().as_str());
			string.push('\n');
		}
		println!("{string}");
		Todo::todo_loop(&mut todo);
	} else {
		let mut todo = Todo::new();
		Todo::todo_loop(&mut todo);
	};
}
